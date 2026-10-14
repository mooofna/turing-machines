// Figures of chapters 1 to 3.

import { placeholder, type FigureEntry } from './types.ts';

export const figuresOneToThree: Readonly<Record<string, FigureEntry>> = {
  'fig1.1': placeholder(3, 1.2, 'binary increment machine, stepped one transition at a time'),
  'fig1.2': placeholder(2.4, 1, 'A_mul with decimal input and output'),
  'fig1.3': placeholder(16 / 9, 0.8, 'M*2 on one and on two tapes, steps for n = 0 to 20'),
  'fig2.1': placeholder(16 / 9, 0.7, 'state diagram, transition table and text format, linked'),
  'fig2.2': placeholder(3, 1.2, 'number of machines with n states, written out'),
  'fig3.1': placeholder(16 / 10, 0.8, 'machines times inputs under a step budget')
};
