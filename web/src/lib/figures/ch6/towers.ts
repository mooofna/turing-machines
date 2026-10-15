// Formatting for Fig. 6.1: digits of 2↑↑k in lines of groups, and what can be said about the
// size of 2↑↑6 from the leading digits of 2↑↑5.

/** Splits `digits` into lines of `groups` groups of `size` digits; each line knows its offset. */
export function digitLines(
  digits: string,
  groups: number,
  size = 10
): { first: number; groups: string[] }[] {
  const perLine = Math.max(1, groups) * size;
  const lines = [];
  for (let at = 0; at < digits.length; at += perLine) {
    const line = digits.slice(at, at + perLine);
    const parts = [];
    for (let g = 0; g < line.length; g += size) parts.push(line.slice(g, g + size));
    lines.push({ first: at + 1, groups: parts });
  }
  return lines;
}

/**
 * The number of decimal digits of 2↑↑(k + 1) = 2^N, given the decimal digits of N = 2↑↑k:
 * floor(N log10 2) + 1, as a mantissa and a power of ten (N log10 2 is not an integer, so
 * adding 1 changes no leading digit). The mantissa is exact to the shown precision as long as
 * N has more than 17 digits.
 */
export function digitsOfPowerOfTwo(n: string): { mantissa: number; exponent: number } {
  const lead = Number(`${n.slice(0, 1)}.${n.slice(1, 17)}`);
  let mantissa = lead * Math.log10(2);
  let exponent = n.length - 1;
  while (mantissa < 1) {
    mantissa *= 10;
    exponent -= 1;
  }
  return { mantissa, exponent };
}

/** `2 ↑↑ k` as MathML content. */
const arrows = (k: string) => `<mn>2</mn><mo lspace="0.17em" rspace="0.17em">↑↑</mo><mn>${k}</mn>`;

/** A tower of `height` twos as nested powers. */
const tower = (height: number): string =>
  height <= 1 ? '<mn>2</mn>' : `<msup><mn>2</mn><mrow>${tower(height - 1)}</mrow></msup>`;

/**
 * The formula for 2↑↑k as a MathML element: the tower itself up to k = 4, then the power of two
 * with the tower below as its exponent (and, for k = 5, the exponent's value `four`).
 */
export function towerFormula(k: number, four?: string): string {
  let right: string;
  if (k <= 4) right = tower(k);
  else {
    right = `<msup><mn>2</mn><mrow>${arrows(String(k - 1))}</mrow></msup>`;
    if (k === 5 && four) right += `<mo>=</mo><msup><mn>2</mn><mn>${four}</mn></msup>`;
  }
  return `<math>${arrows(String(k))}<mo>=</mo>${right}</math>`;
}

/** `m × 10^e` as a MathML element, with the exponent grouped by commas. */
export function scientific(mantissa: string, exponent: string): string {
  return `<math><mn>${mantissa}</mn><mo>×</mo><msup><mn>10</mn><mn>${exponent}</mn></msup></math>`;
}

/** `2 ↑↑ k` as a MathML element. */
export const arrowsMath = (k: number) => `<math>${arrows(String(k))}</math>`;

/** The number of digits of 2↑↑6, ⌊2↑↑5 log₁₀ 2⌋ + 1, as a MathML element. */
export const digitsOfSix =
  '<math><mo>⌊</mo>' +
  arrows('5') +
  '<mspace width="0.2em"></mspace><msub><mi>log</mi><mn>10</mn></msub><mspace width="0.17em"></mspace><mn>2</mn><mo>⌋</mo><mo>+</mo><mn>1</mn></math>';
