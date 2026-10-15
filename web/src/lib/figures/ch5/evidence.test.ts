import { describe, expect, it } from 'vitest';
import { cycle, translation, type Frame } from './evidence.ts';

/** Runs a machine in the standard format from the blank tape; for the tests only. */
function run(standard: string, steps: number): Frame[] {
  const table = standard.split('_').map((row) =>
    [0, 1].map((i) => {
      const t = row.slice(3 * i, 3 * i + 3);
      return t === '---' ? null : { write: t[0]!, move: t[1] === 'R' ? 1 : -1, next: t[2]! };
    })
  );
  const tape = new Map<number, string>();
  let head = 0;
  let state = 'A';
  const frames: Frame[] = [];
  for (let t = 0; t <= steps; t++) {
    frames.push({ state, head, tape: new Map([...tape].filter(([, v]) => v !== '0')) });
    const rule = table[state.charCodeAt(0) - 65]![Number(tape.get(head) ?? '0')];
    if (!rule || rule.next === 'Z') break;
    tape.set(head, rule.write);
    head += rule.move;
    state = rule.next;
  }
  return frames;
}

const key = (f: Frame, shift = 0) =>
  `${f.state}|${f.head + shift}|${[...f.tape]
    .map(([k, v]) => `${k + shift}:${v}`)
    .sort()
    .join(',')}`;

describe('cycle', () => {
  it('finds the first repeated configuration of the cycler of Fig. 5.2', () => {
    const frames = run('1RB0RD_0RC---_1LC0LA_1RA0LC', 63);
    expect(cycle(frames)).toEqual({ from: 7, to: 21 });
    // The run repeats from there on.
    for (let t = 7; t + 14 < frames.length; t++) expect(key(frames[t]!)).toBe(key(frames[t + 14]!));
  });

  it('finds nothing in a run that only grows', () => {
    expect(cycle(run('1RB1LB_1LA1RZ', 6))).toBeNull();
  });
});

describe('translation', () => {
  it('finds the period and shift of the translated cycler of Fig. 5.2', () => {
    const frames = run('0RB---_1RD1RA_1RA0LA_1LC1RA', 99);
    const found = translation(frames);
    expect(found).not.toBeNull();
    const { from, to, shift, cells } = found!;
    expect(shift).toBeGreaterThan(0);
    // From `from` on, the machine applies the same transition as a period earlier: the same
    // state and the same symbol under the head, `shift` cells further right; at the start of
    // every period the cells up to the head agree as well.
    const period = to - from;
    const near = (f: Frame, x: number) => f.tape.get(x) ?? '0';
    for (let t = from; t + period < frames.length; t++) {
      const a = frames[t]!;
      const b = frames[t + period]!;
      expect(b.state).toBe(a.state);
      expect(b.head).toBe(a.head + shift);
      expect(near(b, b.head)).toBe(near(a, a.head));
      if ((t - from) % period === 0) {
        for (let k = 0; k < cells; k++) expect(near(b, b.head - k)).toBe(near(a, a.head - k));
      }
    }
  });

  it('does not report a cycler as translated', () => {
    expect(translation(run('1RB0RD_0RC---_1LC0LA_1RA0LC', 63))).toBeNull();
  });
});
