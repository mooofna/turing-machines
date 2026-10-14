// The bibliography as the components see it.
//
// The entries live in `#lib/content/sources.ts` (`export const sources: Source[]`). They are
// loaded through a glob so that the components build before that file exists; without it the
// list is empty. A reference is numbered by its position in the list, on every page.

import type { Source } from '#lib/content/types.ts';

const modules = import.meta.glob<{ sources?: readonly Source[] }>('/src/lib/content/sources.ts', {
  eager: true
});

/** Every source, in the order of the bibliography. */
export const sources: readonly Source[] = Object.values(modules)[0]?.sources ?? [];

const numbers = new Map(sources.map((s, i) => [s.id, i + 1]));

/** The source with `id`, if any. */
export function sourceById(id: string): Source | undefined {
  return sources.find((s) => s.id === id);
}

/** The reference number of `id` (from 1), or undefined for an unknown id. */
export function sourceNumber(id: string): number | undefined {
  return numbers.get(id);
}

/** Short form for tooltips and labels, for example `Turing 1936`. */
export function shortCitation(source: Source): string {
  const first = source.authors.split(/,| and /)[0]?.trim() ?? source.authors;
  const family = first.startsWith('The ') ? first : (first.split(/\s+/).at(-1) ?? first);
  const more = /,| and /.test(source.authors) ? ' et al.' : '';
  return `${family}${more} ${source.year}`;
}

/** The anchor of a source in the bibliography on the About page. */
export const sourceAnchor = (id: string) => `source-${id}`;
