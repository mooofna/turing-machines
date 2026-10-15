// Figures of chapters 4 to 6. The aspect ratios are those of the built figures after they ran, for
// reference; the figures size themselves.

import type { Component } from 'svelte';
import Champions from './ch4/Champions.svelte';
import History from './ch4/History.svelte';
import Deciders from './ch5/Deciders.svelte';
import Enumeration from './ch5/Enumeration.svelte';
import Antihydra from './ch6/Antihydra.svelte';
import Towers from './ch6/Towers.svelte';
import type { FigureEntry, FigureProps } from './types.ts';

const entry = (
  component: Component<FigureProps>,
  aspect: number,
  aspectPhone: number,
  about: string
): FigureEntry => ({ component, aspect, aspectPhone, about });

export const figuresFourToSix: Readonly<Record<string, FigureEntry>> = {
  'fig4.1': entry(Champions, 1.45, 0.35, 'champions for one to five states as space-time diagrams'),
  'fig4.2': entry(History, 1.4, 0.27, 'milestones of the busy beaver functions'),
  'fig5.1': entry(
    Enumeration,
    1.3,
    0.25,
    'enumeration in tree normal form with the decider funnel'
  ),
  'fig5.2': entry(Deciders, 1.5, 0.35, 'one example machine per decider with its evidence'),
  'fig6.1': entry(Towers, 2, 0.6, 'towers of twos computed exactly'),
  'fig6.2': entry(Antihydra, 0.67, 0.18, 'Antihydra checked against its rule, then iterated')
};
