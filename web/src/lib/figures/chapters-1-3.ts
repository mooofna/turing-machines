// Figures of chapters 1 to 3. The aspect ratios are those of the finished figures at 1440 px and
// 375 px; the figures lay themselves out and do not use them.

import BinaryIncrement from './ch1/BinaryIncrement.svelte';
import Doubling from './ch1/Doubling.svelte';
import UnaryMultiplication from './ch1/UnaryMultiplication.svelte';
import LinkedViews from './ch2/LinkedViews.svelte';
import MachineCount from './ch2/MachineCount.svelte';
import HaltingTable from './ch3/HaltingTable.svelte';
import type { FigureEntry } from './types.ts';

export const figuresOneToThree: Readonly<Record<string, FigureEntry>> = {
  'fig1.1': {
    component: BinaryIncrement,
    aspect: 2.1,
    aspectPhone: 0.68,
    about: 'binary increment machine, stepped one transition at a time'
  },
  'fig1.2': {
    component: UnaryMultiplication,
    aspect: 2.9,
    aspectPhone: 0.77,
    about: 'A_mul with decimal input and output'
  },
  'fig1.3': {
    component: Doubling,
    aspect: 1.4,
    aspectPhone: 0.44,
    about: 'M*2 on one and on two tapes, steps for n = 0 to 20'
  },
  'fig2.1': {
    component: LinkedViews,
    aspect: 2.4,
    aspectPhone: 0.62,
    about: 'state diagram, transition table and text format, linked'
  },
  'fig2.2': {
    component: MachineCount,
    aspect: 2.6,
    aspectPhone: 0.94,
    about: 'number of machines with n states, written out'
  },
  'fig3.1': {
    component: HaltingTable,
    aspect: 1.55,
    aspectPhone: 0.42,
    about: 'machines times inputs under a step budget'
  }
};
