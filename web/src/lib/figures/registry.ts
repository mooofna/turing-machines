// The interactive figures of the chapters, by the id used in the chapter text
// (`<<figure:fig1.1 | ...>>`). Each entry names the component that draws the figure and the
// aspect ratio (width / height) of its frame on a wide screen, so that the page does not jump
// while a figure loads. Until a figure is built, its entry points to `Placeholder.svelte`.

import { figuresOneToThree } from './chapters-1-3.ts';
import { figuresFourToSix } from './chapters-4-6.ts';
import { placeholder, type FigureEntry } from './types.ts';

export type { FigureEntry, FigureProps } from './types.ts';

export const figures: Readonly<Record<string, FigureEntry>> = {
  ...figuresOneToThree,
  ...figuresFourToSix
};

/** The entry for `id`; unknown ids get a placeholder, and the content tests reject them. */
export function figureEntry(id: string): FigureEntry {
  return figures[id] ?? placeholder(16 / 9, 1, 'unknown figure');
}
