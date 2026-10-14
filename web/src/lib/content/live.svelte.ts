// Numbers in the chapter text that are computed in the browser while the reader reads.
//
// The prerendered text contains `<span data-live="ID">` placeholders (see render.server.ts).
// A figure that computes such a value publishes it with `publish(id, value)`; the attachment
// `fillLive`, placed on the element that holds the text, writes every published value into its
// placeholders and keeps them up to date. Until a value exists the placeholder shows an ellipsis,
// and screen readers hear the description below.

import type { Attachment } from 'svelte/attachments';

/** Every live value the text may use, with a description for screen readers and tooltips. */
export const liveIds = {
  'hero-steps': 'number of steps until the five-state champion halts',
  'hero-ones': 'number of ones the five-state champion leaves on the tape',
  'hero-time': 'time the simulation took in this browser',
  'tower-4': 'value of 2\u2191\u21914',
  'tower-5-digits': 'number of decimal digits of 2\u2191\u21915',
  'antihydra-checked': 'number of rule applications confirmed by simulating the machine',
  'antihydra-iterations': 'number of iterations of the Antihydra rule',
  'antihydra-balance': 'value of the counter a after the last iteration'
} as const;

export type LiveId = keyof typeof liveIds;

const values = $state<Partial<Record<LiveId, string>>>({});

const integer = new Intl.NumberFormat('en-US', { maximumFractionDigits: 0 });

/** Publishes a value; numbers are grouped with commas, strings are shown as given. */
export function publish(id: LiveId, value: number | bigint | string): void {
  values[id] =
    typeof value === 'string'
      ? value
      : typeof value === 'bigint'
        ? value.toLocaleString('en-US')
        : integer.format(value);
}

/** Removes a value, for example while it is being recomputed. */
export function retract(id: LiveId): void {
  delete values[id];
}

/** The published value, if any. */
export function liveValue(id: LiveId): string | undefined {
  return values[id];
}

/** Fills the live placeholders inside the element with the published values. */
export const fillLive: Attachment<HTMLElement> = (root) => {
  for (const node of root.querySelectorAll<HTMLElement>('[data-live]')) {
    const id = node.dataset.live as LiveId;
    const value = values[id];
    if (value === undefined) {
      if (node.dataset.state === 'ready') {
        node.dataset.state = 'pending';
        node.replaceChildren(pendingMark(), hiddenLabel(node.title));
      }
      continue;
    }
    if (node.dataset.state === 'ready' && node.textContent === value) continue;
    node.dataset.state = 'ready';
    node.textContent = value;
  }
};

function pendingMark() {
  const mark = document.createElement('span');
  mark.className = 'live-pending';
  mark.setAttribute('aria-hidden', 'true');
  mark.textContent = '\u2026';
  return mark;
}

function hiddenLabel(text: string) {
  const label = document.createElement('span');
  label.className = 'visually-hidden';
  label.textContent = text;
  return label;
}
