// One simulation session in the engine worker, seen from the page.
//
// Requests are sent one after another, so a step never overtakes the run before it. Opening the
// session again (a new machine or input) starts a new epoch: queued requests of the old epoch are
// not sent, and answers that arrive for it resolve to null so the caller can ignore them.

import { engine, type Call } from '#lib/engine/client.ts';
import type {
  BreakpointsDto,
  MachineSpecDto,
  RunResultDto,
  SnapshotDto
} from '#lib/engine/pkg/turing_wasm.js';

let sessions = 0;

export interface RunProgress {
  snapshot: SnapshotDto;
  stepsPerSecond: number;
}

export interface RunOutcome {
  result: RunResultDto;
  elapsedMs: number;
}

export class SessionClient {
  readonly name = `session-${++sessions}`;
  #epoch = 0;
  #tail: Promise<unknown> = Promise.resolve();
  #run: Call | null = null;

  /** Queues `work` behind every earlier request of this session. */
  #enqueue<T>(work: () => Promise<T>): Promise<T | null> {
    const epoch = this.#epoch;
    const next = this.#tail.then(async () => {
      if (epoch !== this.#epoch) return null;
      const result = await work();
      return epoch === this.#epoch ? result : null;
    });
    this.#tail = next.catch(() => undefined);
    return next;
  }

  #snapshot(request: Parameters<ReturnType<typeof engine>['request']>[0]) {
    return this.#enqueue(async () => (await engine().request(request, 'snapshot')).snapshot);
  }

  /** Opens the session at step 0; earlier requests that have not been answered are dropped. */
  open(spec: MachineSpecDto, input: string, radius: number): Promise<SnapshotDto | null> {
    this.#epoch++;
    this.#run?.cancel();
    return this.#snapshot({ type: 'open', session: this.name, spec, input, radius });
  }

  snapshot(radius: number) {
    return this.#snapshot({ type: 'snapshot', session: this.name, radius });
  }

  step(count: number, radius: number) {
    return this.#snapshot({ type: 'step', session: this.name, count, radius });
  }

  back(count: number, radius: number) {
    return this.#snapshot({ type: 'back', session: this.name, count, radius });
  }

  seek(step: number, radius: number) {
    return this.#snapshot({ type: 'seek', session: this.name, step, radius });
  }

  /**
   * Runs up to `maxSteps` steps in the worker. `onProgress` sees a snapshot about ten times a
   * second; the result is null if the session was opened again meanwhile.
   */
  run(
    maxSteps: number,
    breakpoints: BreakpointsDto,
    radius: number,
    onProgress?: (progress: RunProgress) => void
  ): Promise<RunOutcome | null> {
    const epoch = this.#epoch;
    return this.#enqueue(async () => {
      const call = engine().call(
        { type: 'run', session: this.name, maxSteps, breakpoints, radius },
        (response) => {
          if (response.type === 'progress' && epoch === this.#epoch) {
            onProgress?.({
              snapshot: response.snapshot,
              stepsPerSecond: response.stepsPerSecond
            });
          }
        }
      );
      this.#run = call;
      try {
        const response = await call.done;
        if (response.type !== 'stopped') throw new Error(`expected stopped, got ${response.type}`);
        return { result: response.result, elapsedMs: response.elapsedMs };
      } finally {
        if (this.#run === call) this.#run = null;
      }
    });
  }

  /** Asks a running `run` to stop; it then resolves with the configuration reached. */
  stop(): void {
    this.#run?.cancel();
  }

  close(): void {
    this.#epoch++;
    this.#run?.cancel();
    void this.#enqueue(() => engine().request({ type: 'close', session: this.name }, 'ok')).catch(
      () => undefined
    );
  }
}
