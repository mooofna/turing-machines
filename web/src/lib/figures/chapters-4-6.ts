// Figures of chapters 4 to 6.

import { placeholder, type FigureEntry } from './types.ts';

export const figuresFourToSix: Readonly<Record<string, FigureEntry>> = {
  'fig4.1': placeholder(16 / 9, 0.8, 'champions for one to five states as space-time diagrams'),
  'fig4.2': placeholder(3, 0.9, 'milestones of the busy beaver functions'),
  'fig5.1': placeholder(16 / 10, 0.75, 'enumeration in tree normal form with the decider funnel'),
  'fig5.2': placeholder(16 / 9, 0.8, 'one example machine per decider with its evidence'),
  'fig6.1': placeholder(3, 1.2, 'towers of twos computed exactly'),
  'fig6.2': placeholder(16 / 9, 0.8, 'Antihydra checked against its rule, then iterated')
};
