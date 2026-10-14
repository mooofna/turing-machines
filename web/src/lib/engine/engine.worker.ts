/// <reference lib="webworker" />
// The engine worker: runs sessions, renders space-time diagrams, enumerates machines and
// iterates Antihydra, in chunks of about 16 ms so that cancel requests are seen promptly.

import type {
  EnumerationJob,
  MachineSpecDto,
  Session,
  SpaceTimeJob,
  SpaceTimeProgressDto
} from './pkg/turing_wasm';
import type { Envelope, Request, Response } from './protocol';
import { loadWasm, type Wasm } from './wasm';

const scope = self as unknown as DedicatedWorkerGlobalScope;

const sessions = new Map<string, Session>();
const spacetimes = new Map<number, SpaceTimeJob>();
const cancelled = new Set<number>();

/** Target duration of one chunk of work. */
const CHUNK_MS = 16;
/** Interval between progress messages. */
const REPORT_MS = 100;

function send(id: number, response: Response, final = true, transfer: Transferable[] = []) {
  const message: Envelope<Response> = { ...response, id, final };
  scope.postMessage(message, transfer);
}

const yieldToEvents = () => new Promise<void>((resolve) => setTimeout(resolve, 0));

/** Grows or shrinks a chunk size so that one chunk takes about CHUNK_MS. */
function adapt(size: number, elapsed: number, min: number, max: number): number {
  if (elapsed <= 0) return Math.min(max, size * 2);
  const scaled = size * (CHUNK_MS / elapsed);
  return Math.max(min, Math.min(max, Math.round(Math.min(scaled, size * 4))));
}

function openSession(w: Wasm, spec: MachineSpecDto, input: string): Session {
  return new w.Session(spec, input);
}

function session(name: string): Session {
  const found = sessions.get(name);
  if (!found) throw new Error(`no session ${name}`);
  return found;
}

function pixels(job: SpaceTimeJob, palette: Parameters<SpaceTimeJob['rgba']>[0]) {
  return new Uint8ClampedArray(job.rgba(palette));
}

async function handle(w: Wasm, msg: Envelope<Request>): Promise<void> {
  const { id } = msg;
  switch (msg.type) {
    case 'open': {
      sessions.get(msg.session)?.free();
      sessions.delete(msg.session);
      const s = openSession(w, msg.spec, msg.input);
      sessions.set(msg.session, s);
      send(id, { type: 'snapshot', snapshot: s.snapshot(msg.radius) });
      return;
    }
    case 'snapshot':
      send(id, { type: 'snapshot', snapshot: session(msg.session).snapshot(msg.radius) });
      return;
    case 'step':
      send(id, { type: 'snapshot', snapshot: session(msg.session).step(msg.count, msg.radius) });
      return;
    case 'back':
      send(id, { type: 'snapshot', snapshot: session(msg.session).back(msg.count, msg.radius) });
      return;
    case 'seek':
      send(id, { type: 'snapshot', snapshot: session(msg.session).seek(msg.step, msg.radius) });
      return;
    case 'close':
      sessions.get(msg.session)?.free();
      sessions.delete(msg.session);
      send(id, { type: 'ok' });
      return;
    case 'run': {
      const s = session(msg.session);
      const started = performance.now();
      let lastReport = started;
      let chunk = 10_000;
      let done = 0;
      for (;;) {
        const t0 = performance.now();
        const budget = Math.min(chunk, msg.maxSteps - done);
        const result = s.run(budget, msg.breakpoints, msg.radius);
        done += result.stepsDone;
        const now = performance.now();
        chunk = adapt(chunk, now - t0, 1_000, 50_000_000);
        const finished = result.reason !== 'budget' || done >= msg.maxSteps;
        if (finished || cancelled.has(id)) {
          cancelled.delete(id);
          send(id, { type: 'stopped', result, elapsedMs: now - started });
          return;
        }
        if (now - lastReport >= REPORT_MS) {
          const rate = (done / (now - started)) * 1000;
          send(id, { type: 'progress', snapshot: result.snapshot, stepsPerSecond: rate }, false);
          lastReport = now;
        }
        await yieldToEvents();
      }
    }
    case 'spacetime': {
      const job = new w.SpaceTimeJob(msg.spec, msg.input, msg.request);
      spacetimes.set(id, job);
      const started = performance.now();
      let lastReport = started;
      let chunk = 100_000;
      let progress: SpaceTimeProgressDto;
      for (;;) {
        const t0 = performance.now();
        progress = job.advance(chunk);
        const now = performance.now();
        chunk = adapt(chunk, now - t0, 1_000, 1e12);
        if (progress.done || cancelled.has(id)) break;
        if (now - lastReport >= REPORT_MS) {
          const px = pixels(job, msg.palette);
          send(
            id,
            {
              type: 'spacetime',
              job: id,
              width: job.width,
              height: job.height,
              pixels: px,
              progress,
              elapsedMs: now - started
            },
            false,
            [px.buffer]
          );
          lastReport = now;
        }
        await yieldToEvents();
      }
      cancelled.delete(id);
      const px = pixels(job, msg.palette);
      send(
        id,
        {
          type: 'spacetime',
          job: id,
          width: job.width,
          height: job.height,
          pixels: px,
          progress,
          elapsedMs: performance.now() - started
        },
        true,
        [px.buffer]
      );
      return;
    }
    case 'recolor': {
      const job = spacetimes.get(msg.job);
      if (!job) throw new Error(`no space-time job ${msg.job}`);
      const px = pixels(job, msg.palette);
      send(
        id,
        {
          type: 'spacetime',
          job: msg.job,
          width: job.width,
          height: job.height,
          pixels: px,
          progress: { rowsDone: job.height, steps: 0, done: true, halted: false },
          elapsedMs: 0
        },
        true,
        [px.buffer]
      );
      return;
    }
    case 'locate': {
      const job = spacetimes.get(msg.job);
      send(id, { type: 'cell', cell: job?.locate(msg.x, msg.y) ?? null });
      return;
    }
    case 'release':
      spacetimes.get(msg.job)?.free();
      spacetimes.delete(msg.job);
      send(id, { type: 'ok' });
      return;
    case 'enumerate': {
      const shard = msg.shard ?? { index: 0, count: 1 };
      const job: EnumerationJob = new w.EnumerationJob(
        msg.states,
        msg.symbols,
        shard.index,
        shard.count
      );
      const started = performance.now();
      let lastReport = started;
      let chunk = 1_000;
      try {
        for (;;) {
          const t0 = performance.now();
          const progress = job.advance(chunk);
          const now = performance.now();
          chunk = adapt(chunk, now - t0, 50, 1_000_000);
          if (progress.done || cancelled.has(id)) {
            const limit = msg.examples ?? 8;
            const examples: Record<string, string[]> = {};
            for (const d of progress.deciders) {
              examples[d.key] = job.examples(d.key, limit).machines;
            }
            const holdouts = job.holdouts(limit).machines;
            cancelled.delete(id);
            send(id, {
              type: 'enum',
              progress,
              elapsedMs: now - started,
              examples,
              holdouts
            });
            return;
          }
          if (now - lastReport >= REPORT_MS) {
            send(id, { type: 'enum', progress, elapsedMs: now - started }, false);
            lastReport = now;
          }
          await yieldToEvents();
        }
      } finally {
        job.free();
      }
    }
    case 'decide':
      send(id, { type: 'verdict', verdict: w.decide(msg.spec as MachineSpecDto) });
      return;
    case 'antihydra-check': {
      const job = new w.AntihydraCheckJob();
      let rows = 0;
      try {
        for (;;) {
          const batch = job.advance(10_000);
          rows += batch.rows.length;
          const finished = batch.done || rows >= msg.maxRows || cancelled.has(id);
          send(id, { type: 'antihydra-rows', rows: batch.rows, done: finished }, finished);
          if (finished) {
            cancelled.delete(id);
            return;
          }
          await yieldToEvents();
        }
      } finally {
        job.free();
      }
    }
    case 'antihydra-walk': {
      const job = new w.AntihydraWalkJob();
      const started = performance.now();
      let chunk = 1_000;
      let done = 0;
      try {
        for (;;) {
          const t0 = performance.now();
          const step = Math.min(chunk, msg.iterations - done);
          const summary = job.advance(step, msg.samples);
          done += step;
          const now = performance.now();
          chunk = adapt(chunk, now - t0, 10, 1_000_000);
          const finished = done >= msg.iterations || summary.halted || cancelled.has(id);
          send(
            id,
            { type: 'antihydra-walk', summary, done: finished, elapsedMs: now - started },
            finished
          );
          if (finished) {
            cancelled.delete(id);
            return;
          }
          await yieldToEvents();
        }
      } finally {
        job.free();
      }
    }
    case 'cancel':
      return;
  }
}

scope.onmessage = (event: MessageEvent<Envelope<Request>>) => {
  const msg = event.data;
  if (msg.type === 'cancel') {
    cancelled.add(msg.target);
    return;
  }
  void loadWasm()
    .then((w) => handle(w, msg))
    .catch((error: unknown) => {
      send(msg.id, {
        type: 'error',
        message: error instanceof Error ? error.message : String(error)
      });
    });
};
