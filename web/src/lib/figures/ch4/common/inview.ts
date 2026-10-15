// Starts work when an element approaches the viewport.

import type { Attachment } from 'svelte/attachments';

/**
 * Calls `callback` once, when the element comes within `margin` of the viewport (or at once
 * where IntersectionObserver is missing). Usage: `<div {@attach inView(() => (seen = true))}>`.
 */
export function inView(callback: () => void, margin = '200px 0px'): Attachment<HTMLElement> {
  return (element) => {
    if (!('IntersectionObserver' in window)) {
      callback();
      return;
    }
    const observer = new IntersectionObserver(
      (entries) => {
        if (entries.some((e) => e.isIntersecting)) {
          observer.disconnect();
          callback();
        }
      },
      { rootMargin: margin }
    );
    observer.observe(element);
    return () => observer.disconnect();
  };
}
