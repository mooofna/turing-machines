// The six chapters, in reading order. The landing page, the header, the chapter list and the
// pager at the bottom of each chapter are all built from this list.

import type { Chapter } from './types';

export const chapters: readonly Chapter[] = [
  {
    n: 1,
    slug: 'machine',
    title: 'The machine',
    summary:
      'Turing’s definition of 1936 as a finite table of states and symbols, with machines from the course that you can run step by step.'
  },
  {
    n: 2,
    slug: 'programs-as-data',
    title: 'Programs as data',
    summary:
      'How a machine is written down as a string, how one machine can simulate every other, and how many machines of each size there are.'
  },
  {
    n: 3,
    slug: 'limits',
    title: 'What cannot be computed',
    summary:
      'The halting problem by diagonalisation and Rice’s theorem, with machines that are run under a step budget.'
  },
  {
    n: 4,
    slug: 'busy-beaver',
    title: 'The busy beaver',
    summary:
      'Radó’s functions S(n) and Σ(n), why no program computes them, and the champions with one to five states.'
  },
  {
    n: 5,
    slug: 'proving-bb5',
    title: 'Proving BB(5)',
    summary:
      'Enumerating machines in tree normal form, deciding which of them never halt, and the formal proof that settled the five-state case.'
  },
  {
    n: 6,
    slug: 'frontier',
    title: 'The frontier',
    summary:
      'Six-state machines, cryptids such as Antihydra whose halting depends on open problems in number theory, and why ZF set theory cannot determine every value of BB(n).'
  }
];

/** The chapter at `slug`, if any. */
export function chapterBySlug(slug: string): Chapter | undefined {
  return chapters.find((c) => c.slug === slug);
}

/** The chapter whose route id (for example `/machine`) is `routeId`, if any. */
export function chapterByRoute(routeId: string | null | undefined): Chapter | undefined {
  if (!routeId) return undefined;
  return chapters.find((c) => routeId === `/${c.slug}` || routeId.startsWith(`/${c.slug}/`));
}
