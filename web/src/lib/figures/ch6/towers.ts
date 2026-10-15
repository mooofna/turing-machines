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
