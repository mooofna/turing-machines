// The evidence behind the cycler and translated cycler verdicts of Fig. 5.2, found in the
// configurations of a run: the conditions of chapter 5, checked step by step.

/** A configuration: state, head position and the non-blank cells. */
export interface Frame {
  state: string;
  head: number;
  tape: ReadonlyMap<number, string>;
}

const same = (a: ReadonlyMap<number, string>, b: ReadonlyMap<number, string>) =>
  a.size === b.size && [...a].every(([k, v]) => b.get(k) === v);

/** The first repetition of a whole configuration: frame `from` equals frame `to`. */
export function cycle(frames: readonly Frame[]): { from: number; to: number } | null {
  for (let to = 1; to < frames.length; to++) {
    const b = frames[to]!;
    for (let from = to - 1; from >= 0; from--) {
      const a = frames[from]!;
      if (a.state === b.state && a.head === b.head && same(a.tape, b.tape)) return { from, to };
    }
  }
  return null;
}

export interface Translation {
  from: number;
  to: number;
  /** Cells the pattern moves per period: positive to the right. */
  shift: number;
  /** Number of cells behind the head that agree, the head's cell included. */
  cells: number;
  state: string;
}

/**
 * The first pair of times at which the head stands on a new cell at the same end of the visited
 * region, in the same state, and the tape behind it agrees over the largest distance the head
 * moves back in between (the criterion of chapter 5, and its mirror image).
 */
export function translation(frames: readonly Frame[]): Translation | null {
  const n = frames.length;
  // Whether the head is on a cell to the right (left) of every cell visited before.
  const right: boolean[] = [];
  const left: boolean[] = [];
  let hi = -Infinity;
  let lo = Infinity;
  for (const f of frames) {
    right.push(f.head > hi);
    left.push(f.head < lo);
    hi = Math.max(hi, f.head);
    lo = Math.min(lo, f.head);
  }
  const cell = (f: Frame, x: number) => f.tape.get(x) ?? '';
  for (let to = 2; to < n; to++) {
    const b = frames[to]!;
    for (let from = to - 1; from >= 1; from--) {
      const a = frames[from]!;
      if (a.state !== b.state) continue;
      for (const dir of [1, -1] as const) {
        if (!(dir === 1 ? right[from] && right[to] : left[from] && left[to])) continue;
        // How far the head moves back behind its position at `from` before `to`.
        let back = 0;
        for (let s = from; s <= to; s++) back = Math.max(back, dir * (a.head - frames[s]!.head));
        let agrees = true;
        for (let k = 0; k <= back && agrees; k++) {
          agrees = cell(a, a.head - dir * k) === cell(b, b.head - dir * k);
        }
        if (agrees) {
          return { from, to, shift: b.head - a.head, cells: back + 1, state: a.state };
        }
      }
    }
  }
  return null;
}
