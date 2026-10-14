import { describe, expect, it } from 'vitest';
import { LIGHT, readPalette, toHex } from './palette';

describe('toHex', () => {
  it('keeps and normalises hex colours', () => {
    expect(toHex('#215CAF')).toBe('#215caf');
    expect(toHex('#fff')).toBe('#ffffff');
    expect(toHex(' #0e0e0e ')).toBe('#0e0e0e');
  });

  it('converts the rgb() form that getComputedStyle returns', () => {
    expect(toHex('rgb(33, 92, 175)')).toBe('#215caf');
    expect(toHex('rgb(241 241 241)')).toBe('#f1f1f1');
    expect(toHex('rgba(14, 14, 14, 1)')).toBe('#0e0e0e');
  });
});

describe('readPalette', () => {
  it('falls back to the light theme without a document', () => {
    expect(readPalette()).toEqual(LIGHT);
  });
});
