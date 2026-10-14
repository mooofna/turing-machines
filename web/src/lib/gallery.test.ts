// The gallery is data written by hand; these checks keep it consistent.
import { describe, expect, it } from 'vitest';
import { groups, machines } from './gallery';

describe('gallery', () => {
  it('has unique ids', () => {
    const ids = machines.map((m) => m.id);
    expect(new Set(ids).size).toBe(ids.length);
    expect(new Set(groups.map((g) => g.id)).size).toBe(groups.length);
  });

  it('puts every machine into a known group and every group has machines', () => {
    const known = new Set(groups.map((g) => g.id));
    for (const m of machines) expect(known.has(m.group), m.id).toBe(true);
    for (const g of groups)
      expect(
        machines.some((m) => m.group === g.id),
        g.id
      ).toBe(true);
  });

  it('gives every machine exactly one description and the plain fields', () => {
    for (const m of machines) {
      expect(Boolean(m.file) !== Boolean(m.standard), m.id).toBe(true);
      expect(m.text.trim().length, m.id).toBeGreaterThan(0);
      expect(m.title.trim().length, m.id).toBeGreaterThan(0);
      expect(m.provenance.trim().length, m.id).toBeGreaterThan(0);
      expect(m.description.trim().length, m.id).toBeGreaterThan(0);
      expect(m.id, m.id).toMatch(/^[a-z0-9][a-z0-9-]*$/);
    }
  });

  it('writes standard strings in the bbchallenge format', () => {
    for (const m of machines.filter((m) => m.standard)) {
      expect(m.standard, m.id).toMatch(/^([0-9][LR][A-Z]|---)+(_([0-9][LR][A-Z]|---)+)*$/);
    }
  });
});
