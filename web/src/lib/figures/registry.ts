// The interactive figures of the chapters, by the id used in the chapter text
// (`<<figure:fig1.1 | ...>>`). Each entry names the component that draws the figure and the
// aspect ratio (width / height) of its frame on a wide screen, so that the page does not jump
// while a figure loads. Until a figure is built, its entry points to `Placeholder.svelte`.

import type { Component } from 'svelte';
import Placeholder from './Placeholder.svelte';

/** Props every figure component receives. */
export interface FigureProps {
  /** Registry id, for example `fig1.1`. */
  id: string;
  /** Width / height of the frame on wide screens. */
  aspect: number;
  /** Width / height of the frame on phones. */
  aspectPhone: number;
}

export interface FigureEntry {
  component: Component<FigureProps>;
  aspect: number;
  aspectPhone: number;
  /** What the figure shows, for the reviewer of the registry. */
  about: string;
}

const placeholder = (aspect: number, aspectPhone: number, about: string): FigureEntry => ({
  component: Placeholder,
  aspect,
  aspectPhone,
  about
});

export const figures: Readonly<Record<string, FigureEntry>> = {
  'fig1.1': placeholder(3, 1.2, 'binary increment machine, stepped one transition at a time'),
  'fig1.2': placeholder(2.4, 1, 'A_mul with decimal input and output'),
  'fig1.3': placeholder(16 / 9, 0.8, 'M*2 on one and on two tapes, steps for n = 0 to 20'),
  'fig2.1': placeholder(16 / 9, 0.7, 'state diagram, transition table and text format, linked'),
  'fig2.2': placeholder(3, 1.2, 'number of machines with n states, written out'),
  'fig3.1': placeholder(16 / 10, 0.8, 'machines times inputs under a step budget'),
  'fig4.1': placeholder(16 / 9, 0.8, 'champions for one to five states as space-time diagrams'),
  'fig4.2': placeholder(3, 0.9, 'milestones of the busy beaver functions'),
  'fig5.1': placeholder(16 / 10, 0.75, 'enumeration in tree normal form with the decider funnel'),
  'fig5.2': placeholder(16 / 9, 0.8, 'one example machine per decider with its evidence'),
  'fig6.1': placeholder(3, 1.2, 'towers of twos computed exactly'),
  'fig6.2': placeholder(16 / 9, 0.8, 'Antihydra checked against its rule, then iterated')
};

/** The entry for `id`; unknown ids get a placeholder, and the content tests reject them. */
export function figureEntry(id: string): FigureEntry {
  return figures[id] ?? placeholder(16 / 9, 1, 'unknown figure');
}
