// Regions that scroll (sideways on narrow screens, or a long block), reachable by keyboard only
// when they do.

import type { Attachment } from 'svelte/attachments';

/**
 * Makes the element a focusable, labelled region while its content is wider or taller than it, so
 * that keyboard users can scroll it; otherwise it stays out of the tab order.
 * Usage: `<div class="scroll" {@attach scrollRegion('Results, scrolls sideways')}>`.
 */
export function scrollRegion(label: string): Attachment<HTMLElement> {
  return (element) => {
    const check = () => {
      const scrolls =
        element.scrollWidth > element.clientWidth + 1 ||
        element.scrollHeight > element.clientHeight + 1;
      if (scrolls) {
        element.tabIndex = 0;
        element.setAttribute('role', 'region');
        element.setAttribute('aria-label', label);
      } else {
        element.removeAttribute('tabindex');
        element.removeAttribute('role');
        element.removeAttribute('aria-label');
      }
    };
    const observer = new ResizeObserver(check);
    observer.observe(element);
    for (const child of element.children) observer.observe(child);
    check();
    return () => observer.disconnect();
  };
}
