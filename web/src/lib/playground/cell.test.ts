import { describe, expect, it } from 'vitest';
import { parseCell, showCell, type CellContext } from './cell';

const ctx: CellContext = {
  states: ['q0', 'q1'],
  gamma: ['0', '1', '␣'],
  blank: '␣',
  read: '1',
  compact: false
};

describe('table cells', () => {
  it('reads the course notation in several spellings', () => {
    for (const text of ['q1, 0, R', 'q1,0,R', 'q1 0 r', '(q1, 0, R)', '  q1 ,  0 ,R ']) {
      expect(parseCell(text, ctx)).toEqual({
        kind: 'ok',
        to: 'q1',
        write: '0',
        dir: 'R',
        missing: {}
      });
    }
  });

  it('reads the compact notation', () => {
    expect(parseCell('0Lq1', { ...ctx, compact: true })).toMatchObject({
      kind: 'ok',
      to: 'q1',
      write: '0',
      dir: 'L'
    });
  });

  it('understands the blank and the keep marker', () => {
    expect(parseCell('q0, _, L', ctx)).toMatchObject({ write: '␣' });
    expect(parseCell('q0, *, L', ctx)).toMatchObject({ write: '1' });
  });

  it('names what is missing', () => {
    expect(parseCell('q5, X, N', ctx)).toMatchObject({
      kind: 'ok',
      missing: { state: 'q5', symbol: 'X' }
    });
  });

  it('treats an empty cell as undefined', () => {
    expect(parseCell('', ctx)).toEqual({ kind: 'empty' });
    expect(parseCell(' --- ', ctx)).toEqual({ kind: 'empty' });
  });

  it('explains mistakes', () => {
    expect(parseCell('q1, 0', ctx)).toMatchObject({ kind: 'error' });
    expect(parseCell('q1, 0, Up', ctx)).toMatchObject({
      kind: 'error',
      message: 'Up is not a move. Use L, R or N.'
    });
    expect(parseCell('q1, 00, R', ctx)).toMatchObject({ kind: 'error' });
  });

  it('shows transitions in both notations', () => {
    const t = { from: 'q0', read: ['1'], to: 'q1', write: ['0'], dir: ['R' as const] };
    expect(showCell(t, false)).toBe('q1, 0, R');
    expect(showCell(t, true)).toBe('0Rq1');
    expect(showCell(undefined, false)).toBe('');
  });
});
