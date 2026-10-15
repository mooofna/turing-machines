// The enumeration of Fig. 5.1: all machines of one size in tree normal form, in the batch
// worker, or split into shards over several workers.
//
// The enumerator splits the tree at a fixed depth and deals the subtrees out round robin, so the
// shards together visit every node exactly once. Their counts add up; for the records, a tie is
// resolved in favour of the lower shard, so the machine shown does not depend on how many
// workers ran.

import { batch, Engine } from '#lib/engine/client.ts';
import type { DeciderCountDto, EnumProgressDto, RecordDto } from '#lib/engine/pkg/turing_wasm.js';
import type { Response } from '#lib/engine/protocol.ts';

/** Counts of an enumeration that may still be running. */
export type Progress = EnumProgressDto;

/** Adds up the progress of shards, given in shard order. */
export function merge(parts: readonly Progress[]): Progress {
  const deciders: DeciderCountDto[] = [];
  const better = (best: RecordDto | undefined, next: RecordDto | undefined) =>
    next && (!best || next.value > best.value) ? next : best;
  let maxSteps: RecordDto | undefined;
  let maxOnes: RecordDto | undefined;
  for (const part of parts) {
    for (const d of part.deciders) {
      const known = deciders.find((e) => e.key === d.key);
      if (known) known.nonHalting += d.nonHalting;
      else deciders.push({ ...d });
    }
    maxSteps = better(maxSteps, part.maxSteps);
    maxOnes = better(maxOnes, part.maxOnes);
  }
  return {
    nodes: parts.reduce((n, p) => n + p.nodes, 0),
    halting: parts.reduce((n, p) => n + p.halting, 0),
    holdouts: parts.reduce((n, p) => n + p.holdouts, 0),
    deciders,
    maxSteps,
    maxOnes,
    done: parts.length > 0 && parts.every((p) => p.done)
  };
}

/** The progress of a shard that has not reported yet. */
const empty = (): Progress => ({ nodes: 0, halting: 0, holdouts: 0, deciders: [], done: false });

export interface Outcome {
  progress: Progress;
  /** Wall-clock time from the request to the last result, in this browser. */
  elapsedMs: number;
  /** Number of workers that shared the work. */
  workers: number;
  /** Up to a few unresolved machines. */
  holdouts: string[];
}

/** A pool of extra batch workers for sharded runs; the first worker is always `batch()`. */
export class Pool {
  #extra: Engine[] = [];

  /** `count` workers, starting with the shared batch worker. */
  workers(count: number): Engine[] {
    while (this.#extra.length < count - 1) {
      this.#extra.push(new Engine(`batch-${this.#extra.length + 2}`));
    }
    return [batch(), ...this.#extra.slice(0, count - 1)];
  }

  /** Stops the extra workers. */
  terminate(): void {
    for (const engine of this.#extra) engine.terminate();
    this.#extra = [];
  }
}

export interface Job {
  done: Promise<Outcome>;
  cancel(): void;
}

/**
 * Enumerates all `states`-state machines with two symbols in `shards` shards on `engines`
 * (one shard queue shared by all workers). `onProgress` sees the merged counts as they grow.
 */
export function enumerate(
  states: number,
  engines: readonly Engine[],
  shards: number,
  onProgress: (progress: Progress) => void
): Job {
  const started = performance.now();
  const parts: Progress[] = Array.from({ length: shards }, empty);
  const holdouts: string[][] = Array.from({ length: shards }, () => []);
  const calls: { cancel(): void }[] = [];
  let next = 0;
  let cancelled = false;

  const report = () => onProgress(merge(parts));

  const worker = async (engine: Engine): Promise<void> => {
    while (!cancelled && next < shards) {
      const index = next++;
      const call = engine.call(
        {
          type: 'enumerate',
          states,
          symbols: 2,
          shard: { index, count: shards },
          examples: 4
        },
        (response: Response) => {
          if (response.type !== 'enum' || cancelled) return;
          parts[index] = response.progress;
          if (response.holdouts) holdouts[index] = response.holdouts;
          report();
        }
      );
      calls.push(call);
      const final = await call.done;
      if (final.type !== 'enum') throw new Error(`expected enum, got ${final.type}`);
    }
  };

  const done = Promise.all(engines.map(worker)).then(() => ({
    progress: merge(parts),
    elapsedMs: performance.now() - started,
    workers: engines.length,
    holdouts: holdouts.flat()
  }));

  return {
    done,
    cancel() {
      cancelled = true;
      for (const call of calls) call.cancel();
    }
  };
}
