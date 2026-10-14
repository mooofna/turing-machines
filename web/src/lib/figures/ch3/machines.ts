// The machines of Fig. 3.1. All read a unary input x^n, so that every machine can run on every
// input of the table. M*2 comes from the gallery; the others are written here in the course
// notation and parsed by the engine.

export interface HaltingRow {
  key: string;
  /** Name in the row header, with an optional subscript. */
  name: string;
  sub?: string;
  /** What the machine does, in one sentence; `backticks` mark mono. No claim about halting. */
  note: string;
  /** A gallery id, or a description in the course notation. */
  gallery?: string;
  source?: string;
}

const DIGITS = ['x', '1', '2', '3', '4', '5', '6', '7', '8', '9'];

/** A decimal counter on the n cells of the input, least significant digit first; x is 0. */
function counter(name: string, onOverflow: string): string {
  const increments = DIGITS.slice(0, 9).map((d, i) => `δ(inc, ${d}) = (back, ${DIGITS[i + 1]}, L)`);
  return [
    `machine "${name}"`,
    'Σ = {x}',
    'start = inc',
    ...increments,
    'δ(inc, 9) = (inc, x, R)  // carry',
    onOverflow,
    `δ(back, {${DIGITS.join(', ')}}) = (back, *, L)`,
    'δ(back, ␣) = (inc, ␣, R)'
  ].join('\n');
}

export const ROWS: readonly HaltingRow[] = [
  {
    key: 'double',
    name: 'M',
    sub: '*2',
    note: 'The doubling machine of Fig. 1.3.',
    gallery: 'doubling'
  },
  {
    key: 'parity',
    name: 'Parity',
    note: 'Halts on even n. On odd n it reaches a blank in state odd, where `δ(odd, ␣) = (odd, ␣, R)`.',
    source: [
      'machine "Parity"',
      'Σ = {x}',
      'start = even',
      'δ(even, x) = (odd, x, R)',
      'δ(odd, x) = (even, x, R)',
      'δ(odd, ␣) = (odd, ␣, R)'
    ].join('\n')
  },
  {
    key: 'counter',
    name: 'Counter',
    note: 'Counts in decimal on the n cells of the input and halts when the count overflows.',
    source: counter('Counter', '// no transition for a carry past the last digit: halt')
  },
  {
    key: 'unbounded',
    name: 'Unbounded counter',
    note: 'The same counter, but on overflow it writes a new digit instead of halting.',
    source: counter('Unbounded counter', 'δ(inc, ␣) = (back, 1, L)  // a new digit')
  }
];

/** The inputs x^n, one column each. */
export const INPUTS: readonly string[] = [1, 2, 3, 4, 5].map((n) => 'x'.repeat(n));

/** Budgets the slider offers: 1, 2 and 5 times the powers of ten from 10 to 10^6. */
export const BUDGETS: readonly number[] = [1, 2, 3, 4, 5, 6].flatMap((e) =>
  e === 6 ? [10 ** e] : [1, 2, 5].map((k) => k * 10 ** e)
);
