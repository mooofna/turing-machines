// Helpers shared by the figures of chapters 1 to 3: starting work when a figure scrolls into
// view, reduced motion, machines from the gallery and small formatting functions.

import type { Attachment } from 'svelte/attachments';
import type { MachineSpecDto } from '#lib/engine/pkg/turing_wasm.js';
import { galleryMachine } from '#lib/gallery.ts';
import { services } from '#lib/machine/services.ts';

/** True if the reader asked for reduced motion; false during prerendering. */
export function reducedMotion(): boolean {
  return (
    typeof window !== 'undefined' && window.matchMedia('(prefers-reduced-motion: reduce)').matches
  );
}

/**
 * Calls `onvisible` once, when the element comes within 200 px of the viewport. Figures start
 * the engine only then, so that a chapter does not load WebAssembly for figures nobody sees.
 */
export function inView(onvisible: () => void): Attachment<HTMLElement> {
  return (element) => {
    if (typeof IntersectionObserver === 'undefined') {
      onvisible();
      return;
    }
    let done = false;
    const observer = new IntersectionObserver(
      (entries) => {
        if (done || !entries.some((entry) => entry.isIntersecting)) return;
        done = true;
        observer.disconnect();
        onvisible();
      },
      { rootMargin: '200px 0px' }
    );
    observer.observe(element);
    return () => observer.disconnect();
  };
}

/** The machine of a gallery entry, parsed by the engine; throws if it has errors. */
export async function gallerySpec(id: string): Promise<MachineSpecDto> {
  const machine = galleryMachine(id);
  if (!machine) throw new Error(`no gallery machine ${id}`);
  const svc = await services();
  const parsed =
    machine.format === 'standard' ? svc.fromStandard(machine.text) : svc.parseSource(machine.text);
  if (!parsed.spec) throw new Error(`gallery machine ${id} has errors`);
  return parsed.spec;
}

/** A description in the course notation, parsed by the engine; throws if it has errors. */
export async function sourceSpec(source: string): Promise<MachineSpecDto> {
  const svc = await services();
  const parsed = svc.parseSource(source);
  if (!parsed.spec) {
    throw new Error(parsed.diagnostics.map((d) => d.message).join('; ') || 'invalid machine');
  }
  return parsed.spec;
}

const SUPERSCRIPT = '⁰¹²³⁴⁵⁶⁷⁸⁹';

/** 144 as ¹⁴⁴, for exponents in plain text. */
export function superscript(n: number): string {
  return String(n)
    .split('')
    .map((d) => SUPERSCRIPT[Number(d)] ?? d)
    .join('');
}

/** The message of an error, for a status line. */
export function errorText(error: unknown): string {
  return error instanceof Error ? error.message : String(error);
}

/**
 * Makes a region that scrolls sideways reachable by keyboard, as the chapter does for wide
 * formulas: while its content overflows, it is a focusable, labelled region.
 */
export function scrollRegion(label: string): Attachment<HTMLElement> {
  return (element) => {
    const check = () => {
      if (element.scrollWidth > element.clientWidth + 1) {
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
    if (element.firstElementChild) observer.observe(element.firstElementChild);
    return () => observer.disconnect();
  };
}
