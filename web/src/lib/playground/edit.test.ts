import { describe, expect, it } from 'vitest';
import type { DirDto, MachineSpecDto } from '#lib/engine/pkg/turing_wasm.js';
import * as edit from './edit';

function t(
  from: string,
  read: string,
  to: string,
  write: string,
  dir: DirDto,
  leading: string[] = []
) {
  return { from, read: [read], to, write: [write], dir: [dir], leading };
}

/** Binary increment, as in the playground's first example. */
function machine(): MachineSpecDto {
  return {
    ...edit.emptyMachine(),
    name: 'Binary increment',
    kind: 'transducer',
    inputAlphabet: ['0', '1'],
    tapeAlphabet: ['0', '1', '␣'],
    states: [
      { name: 'q0', isFinal: false },
      { name: 'q1', isFinal: false },
      { name: 'q2', isFinal: true }
    ],
    transitions: [
      t('q0', '0', 'q0', '0', 'R', ['Walk to the end of the number.']),
      t('q0', '1', 'q0', '1', 'R'),
      t('q0', '␣', 'q1', '␣', 'L'),
      t('q1', '1', 'q1', '0', 'L', ['Add one, carrying to the left.']),
      t('q1', '0', 'q2', '1', 'L'),
      t('q1', '␣', 'q2', '1', 'L')
    ]
  };
}

describe('structural edits', () => {
  it('defines, replaces and removes a transition, grouped by state', () => {
    let spec = edit.setTransition(machine(), 'q2', ['0'], { to: 'q2', write: ['0'], dir: ['R'] });
    expect(spec.transitions.at(-1)).toMatchObject({ from: 'q2', read: ['0'] });
    spec = edit.setTransition(spec, 'q0', ['␣'], { to: 'q2', write: ['1'], dir: ['L'] });
    const i = edit.findTransition(spec, 'q0', ['␣']);
    expect(i).toBe(2);
    expect(spec.transitions[i]).toMatchObject({ to: 'q2', write: ['1'], dir: ['L'] });
    // A new transition of q1 goes after the other transitions of q1.
    spec = edit.setTransition(spec, 'q0', ['␣'], null);
    expect(edit.findTransition(spec, 'q0', ['␣'])).toBe(-1);
  });

  it('keeps comments when a transition changes', () => {
    const spec = machine();
    const i = edit.findTransition(spec, 'q1', ['1']);
    const next = edit.setTransition(spec, 'q1', ['1'], { to: 'q2', write: ['1'], dir: ['R'] });
    expect(next.transitions[i]!.leading).toEqual(spec.transitions[i]!.leading);
  });

  it('adds missing states and symbols', () => {
    const spec = edit.setTransition(machine(), 'q2', ['0'], { to: 'q9', write: ['X'], dir: ['N'] });
    expect(edit.stateNames(spec)).toContain('q9');
    expect(spec.tapeAlphabet).toContain('X');
  });

  it('renames a state everywhere', () => {
    const spec = edit.renameState(machine(), 'q0', 'scan');
    expect(spec.start).toBe('scan');
    expect(spec.transitions.filter((t) => t.from === 'scan')).toHaveLength(3);
    expect(spec.transitions.some((t) => t.from === 'q0' || t.to === 'q0')).toBe(false);
  });

  it('removes a state with its transitions', () => {
    const spec = edit.removeState(machine(), 'q1');
    expect(edit.stateNames(spec)).toEqual(['q0', 'q2']);
    expect(spec.transitions.every((t) => t.from !== 'q1' && t.to !== 'q1')).toBe(true);
    expect(edit.removeState(edit.emptyMachine(), 'q0').states).toHaveLength(1);
  });

  it('suggests names in the style of the machine', () => {
    expect(edit.nextStateName(machine())).toBe('q3');
    const beaver = {
      ...machine(),
      states: ['A', 'B', 'C', 'D', 'Z'].map((name) => ({ name, isFinal: false }))
    };
    expect(edit.nextStateName(beaver)).toBe('E');
  });

  it('explains invalid names and symbols', () => {
    const spec = machine();
    expect(edit.stateNameProblem(spec, 'q1')).toBe('There is already a state q1.');
    expect(edit.stateNameProblem(spec, 'q1', 'q1')).toBeNull();
    expect(edit.stateNameProblem(spec, 'a b')).toMatch(/spaces/);
    expect(edit.symbolProblem(spec, 'ab')).toMatch(/single character/);
    expect(edit.symbolProblem(spec, '1')).toMatch(/already/);
    expect(edit.symbolProblem(spec, '_')).toMatch(/blank/);
    expect(edit.symbolProblem(spec, 'X')).toBeNull();
  });

  it('removes a symbol with the transitions that use it', () => {
    const spec = edit.removeSymbol(machine(), '0');
    expect(spec.tapeAlphabet).not.toContain('0');
    expect(spec.inputAlphabet).toEqual(['1']);
    expect(spec.transitions.some((t) => t.read.includes('0') || t.write.includes('0'))).toBe(false);
  });

  it('compares machines without positions and comments', () => {
    const a = machine();
    const b = edit.withoutSpans(a);
    b.transitions[0]!.comment = 'changed';
    expect(edit.sameMachine(a, b)).toBe(true);
    expect(edit.behaviourKey(a)).toBe(edit.behaviourKey(b));
    const c = edit.toggleFinal(a, 'q1');
    expect(edit.sameMachine(a, c)).toBe(false);
  });
});
