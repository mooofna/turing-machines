import { describe, expect, it } from 'vitest';
import { merge, settle, type Progress } from './enumerate.ts';

const part = (over: Partial<Progress>): Progress => ({
  nodes: 0,
  halting: 0,
  holdouts: 0,
  deciders: [],
  done: true,
  ...over
});

describe('merge', () => {
  it('adds up the counts of the shards', () => {
    const merged = merge([
      part({
        nodes: 10,
        halting: 3,
        deciders: [
          { key: 'loops', label: 'Loops', nonHalting: 5 },
          { key: 'repwl', label: 'RepWL', nonHalting: 1 }
        ]
      }),
      part({
        nodes: 7,
        halting: 2,
        holdouts: 1,
        deciders: [{ key: 'loops', label: 'Loops', nonHalting: 4 }]
      })
    ]);
    expect(merged.nodes).toBe(17);
    expect(merged.halting).toBe(5);
    expect(merged.holdouts).toBe(1);
    expect(merged.deciders).toEqual([
      { key: 'loops', label: 'Loops', nonHalting: 9 },
      { key: 'repwl', label: 'RepWL', nonHalting: 1 }
    ]);
    expect(merged.done).toBe(true);
  });

  it('keeps the record of the lowest shard on a tie', () => {
    const merged = merge([
      part({ maxSteps: { value: 96, machine: 'a' }, maxOnes: { value: 13, machine: 'a' } }),
      part({ maxSteps: { value: 107, machine: 'b' }, maxOnes: { value: 13, machine: 'b' } }),
      part({ maxSteps: { value: 107, machine: 'c' } })
    ]);
    expect(merged.maxSteps).toEqual({ value: 107, machine: 'b' });
    expect(merged.maxOnes).toEqual({ value: 13, machine: 'a' });
  });

  it('is not done while a shard runs', () => {
    expect(merge([part({}), part({ done: false })]).done).toBe(false);
    expect(merge([]).done).toBe(false);
  });
});

describe('settle', () => {
  it('breaks ties between shards by the other value, as the engine does', async () => {
    const parts = [
      part({ maxSteps: { value: 6, machine: 'x' }, maxOnes: { value: 13, machine: 'a' } }),
      part({ maxSteps: { value: 6, machine: 'y' }, maxOnes: { value: 13, machine: 'b' } }),
      part({ maxSteps: { value: 5, machine: 'z' }, maxOnes: { value: 12, machine: 'c' } })
    ];
    const measured: Record<string, { steps: number; ones: number }> = {
      a: { steps: 96, ones: 13 },
      b: { steps: 107, ones: 13 },
      x: { steps: 6, ones: 3 },
      y: { steps: 6, ones: 4 }
    };
    const asked: string[] = [];
    const settled = await settle(parts, merge(parts), async (m) => {
      asked.push(m);
      return measured[m]!;
    });
    expect(settled.maxSteps).toEqual({ value: 6, machine: 'y' });
    expect(settled.maxOnes).toEqual({ value: 13, machine: 'b' });
    expect(asked.sort()).toEqual(['a', 'b', 'x', 'y']);
  });

  it('measures nothing without a tie', async () => {
    const parts = [part({ maxSteps: { value: 7, machine: 'x' } }), part({})];
    const settled = await settle(parts, merge(parts), () => {
      throw new Error('no measurement expected');
    });
    expect(settled.maxSteps).toEqual({ value: 7, machine: 'x' });
  });
});
