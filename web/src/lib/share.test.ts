import { describe, expect, it } from 'vitest';
import { buildHash, parseHash } from './share';

describe('share links', () => {
  it('reads the three machine kinds', () => {
    expect(parseHash('#g=bb4')).toEqual({ machine: { kind: 'gallery', id: 'bb4' } });
    expect(parseHash('#tm=1RB1LB_1LA1RZ')).toEqual({
      machine: { kind: 'standard', text: '1RB1LB_1LA1RZ' }
    });
    expect(parseHash('#src=1.abc-_')).toEqual({ machine: { kind: 'source', payload: '1.abc-_' } });
  });

  it('reads the input, in any order', () => {
    expect(parseHash('#in=%23xx%23x%23&g=mul')).toEqual({
      machine: { kind: 'gallery', id: 'mul' },
      input: '#xx#x#'
    });
    expect(parseHash('#g=mul&in=')).toEqual({ machine: { kind: 'gallery', id: 'mul' }, input: '' });
  });

  it('ignores hashes without a machine', () => {
    expect(parseHash('')).toBeNull();
    expect(parseHash('#')).toBeNull();
    expect(parseHash('#main')).toBeNull();
    expect(parseHash('#in=101')).toBeNull();
    expect(parseHash('#g=')).toBeNull();
    expect(parseHash('#g=%E0%A4%A')).toBeNull();
  });

  it('round-trips through buildHash', () => {
    const links = [
      { machine: { kind: 'gallery', id: 'binary-increment' } },
      { machine: { kind: 'standard', text: '1RB1LB_1LA1RZ' }, input: '' },
      { machine: { kind: 'source', payload: '1.eJxLy' }, input: '#x x&y=␣#' }
    ] as const;
    for (const link of links) expect(parseHash(buildHash(link))).toEqual(link);
  });

  it('keeps hashes readable', () => {
    expect(buildHash({ machine: { kind: 'gallery', id: 'bb4' }, input: '1,0' })).toBe(
      '#g=bb4&in=1,0'
    );
  });
});
