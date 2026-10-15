import { describe, expect, it } from 'vitest';
import { digitLines, digitsOfPowerOfTwo } from './towers.ts';

describe('digitLines', () => {
  it('cuts digits into lines of groups and numbers the first digit of each line', () => {
    const lines = digitLines('1234567890123', 1, 5);
    expect(lines).toEqual([
      { first: 1, groups: ['12345'] },
      { first: 6, groups: ['67890'] },
      { first: 11, groups: ['123'] }
    ]);
    expect(digitLines('1234567890', 2, 3)).toEqual([
      { first: 1, groups: ['123', '456'] },
      { first: 7, groups: ['789', '0'] }
    ]);
  });
});

describe('digitsOfPowerOfTwo', () => {
  it('gives N log10 2 as a mantissa and a power of ten', () => {
    // 2^65536 has 19,729 digits: 65536 log10 2 = 19,728.3.
    const small = digitsOfPowerOfTwo('65536');
    expect(small.exponent).toBe(4);
    expect(small.mantissa).toBeCloseTo(1.97283, 4);
    // 2^(2^65536): about 6.031 times 10^19727 digits.
    const big = digitsOfPowerOfTwo(`2003529930406846464979072351560255750447${'0'.repeat(19_689)}`);
    expect(big.exponent).toBe(19_727);
    expect(big.mantissa.toFixed(3)).toBe('6.031');
  });
});
