// Shared types and the placeholder entry of the figure registry.

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

export const placeholder = (aspect: number, aspectPhone: number, about: string): FigureEntry => ({
  component: Placeholder,
  aspect,
  aspectPhone,
  about
});
