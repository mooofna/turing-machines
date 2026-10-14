// Checks of the development fallback and of the built-in sample machines run on it.

import { describe, expect, it } from 'vitest';
import { samples } from '#lib/playground/samples.ts';
import { format, fromStandard, parse, toStandard } from './dsl';
import { FallbackSession } from './sim';

function run(source: string, input?: string) {
  const { spec, diagnostics } = parse(source);
  expect(diagnostics).toEqual([]);
  const session = new FallbackSession(spec!, input ?? spec!.input);
  return session.run(1_000_000, { transitions: [], states: [] }, 3);
}

describe('fallback description language', () => {
  it('round-trips every sample through format and parse', () => {
    for (const sample of samples) {
      const first = parse(sample.source).spec!;
      const again = parse(format(first, false)).spec!;
      const strip = (s: typeof first) => ({
        ...s,
        transitions: s.transitions.map(({ span: _span, ...t }) => t)
      });
      expect(strip(again)).toEqual(strip(first));
    }
  });

  it('reports errors with positions', () => {
    const { spec, diagnostics } = parse('δ(q0, a) = (q1, b, X)\nδ(q0, (a) = (q1, b, L)');
    expect(spec).toBeUndefined();
    expect(diagnostics.map((d) => d.line)).toEqual([1, 2]);
    const twice = parse('δ(q0, a) = (q1, b, R)\nδ(q0, a) = (q1, b, L)');
    expect(twice.diagnostics.map((d) => [d.code, d.line])).toEqual([['E004', 2]]);
  });

  it('expands read sets and the keep marker', () => {
    const { spec } = parse('δ(q, {x, X}) = (q, *, R)');
    expect(spec!.transitions.map((t) => t.write[0])).toEqual(['x', 'X']);
  });

  it('reads several tapes', () => {
    const { spec, diagnostics } = parse('tapes = 2\nδ(q, (x, ␣)) = (q, (x, x), (R, R))');
    expect(diagnostics).toEqual([]);
    expect(spec!.transitions[0]!.read).toEqual(['x', '␣']);
  });

  it('converts the standard format both ways', () => {
    const { spec } = fromStandard('1RB1LB_1LA0LC_1RZ1LD_1RD0RA');
    expect(toStandard(spec!)).toBe('1RB1LB_1LA0LC_1RZ1LD_1RD0RA');
  });
});

describe('sample machines', () => {
  it('increments 1011 to 1100', () => {
    const { snapshot, reason } = run(samples[0]!.source);
    expect(reason).toBe('halted');
    expect(snapshot.output?.word).toBe('1100');
  });

  it('multiplies 3 by 2', () => {
    const { snapshot } = run(samples[1]!.source);
    expect(snapshot.output).toEqual({ word: 'xxxxxx', value: '6' });
    for (const [m, n] of [
      [0, 3],
      [2, 0],
      [1, 1],
      [4, 3]
    ] as const) {
      const r = run(samples[1]!.source, `#${'x'.repeat(m)}#${'x'.repeat(n)}#`);
      expect(r.snapshot.output?.value).toBe(String(m * n));
    }
  });

  it('runs the BB(4) champion for 107 steps leaving 13 ones', () => {
    const { snapshot } = run(samples[2]!.source);
    expect(snapshot.step).toBe(107);
    expect(snapshot.ones).toBe(13);
  });

  it('steps back exactly', () => {
    const { spec } = parse(samples[2]!.source);
    const s = new FallbackSession(spec!, '');
    const at50 = s.seek(50, 4);
    s.step(30, 4);
    expect(s.back(30, 4)).toEqual(at50);
  });
});
