// The colours of space-time diagrams, read from the CSS tokens of the current theme.
//
// The tokens are written as light-dark(...), which getComputedStyle does not resolve on custom
// properties. A hidden probe element resolves them as `color`, and a 1 x 1 canvas turns any CSS
// colour syntax the browser returns (rgb(), color(srgb ...), oklch(), ...) into #rrggbb.

import type { PaletteDto } from './pkg/turing_wasm';

/** The light theme, for tests and prerendering, where no document is available. */
export const LIGHT: PaletteDto = { background: '#ffffff', ink: '#111111', accent: '#215caf' };

let probe: HTMLSpanElement | null = null;
let pixel: CanvasRenderingContext2D | null = null;

const hex = (n: number) => n.toString(16).padStart(2, '0');

/** Converts any CSS colour to `#rrggbb`, compositing transparent colours over white. */
export function toHex(color: string): string {
  const short = /^#([0-9a-f]{3})$/i.exec(color.trim());
  if (short) {
    const [r, g, b] = short[1]!.split('');
    return `#${r}${r}${g}${g}${b}${b}`.toLowerCase();
  }
  if (/^#[0-9a-f]{6}$/i.test(color.trim())) return color.trim().toLowerCase();
  const rgb = /^rgba?\(\s*(\d+)[\s,]+(\d+)[\s,]+(\d+)\s*(?:[,/]\s*1(?:\.0+)?\s*)?\)$/i.exec(
    color.trim()
  );
  if (rgb) return `#${hex(+rgb[1]!)}${hex(+rgb[2]!)}${hex(+rgb[3]!)}`;
  pixel ??= document
    .createElement('canvas')
    .getContext('2d', { willReadFrequently: true }) as CanvasRenderingContext2D | null;
  if (!pixel) return '#000000';
  pixel.canvas.width = 1;
  pixel.canvas.height = 1;
  pixel.clearRect(0, 0, 1, 1);
  pixel.fillStyle = '#ffffff';
  pixel.fillRect(0, 0, 1, 1);
  pixel.fillStyle = color;
  pixel.fillRect(0, 0, 1, 1);
  const [r = 0, g = 0, b = 0] = pixel.getImageData(0, 0, 1, 1).data;
  return `#${hex(r)}${hex(g)}${hex(b)}`;
}

/** The computed value of the colour token `name` (for example `--bg`) in the current theme. */
export function token(name: string): string {
  if (!probe || !probe.isConnected) {
    probe = document.createElement('span');
    probe.setAttribute('aria-hidden', 'true');
    probe.style.cssText = 'position:absolute;width:0;height:0;overflow:hidden;visibility:hidden';
    document.body.append(probe);
  }
  probe.style.color = `var(${name})`;
  return toHex(getComputedStyle(probe).color);
}

/** The palette of the current theme: background, ink and the accent for the head. */
export function readPalette(): PaletteDto {
  if (typeof document === 'undefined') return LIGHT;
  return { background: token('--bg'), ink: token('--text'), accent: token('--accent') };
}

/**
 * Calls `callback` with the new palette whenever the theme changes: through the toggle (the
 * `themechange` event or the `data-theme` attribute) or through the system setting. Returns a
 * function that stops listening.
 */
export function onThemeChange(callback: (palette: PaletteDto) => void): () => void {
  let last = JSON.stringify(readPalette());
  let queued = false;
  const check = () => {
    if (queued) return;
    queued = true;
    requestAnimationFrame(() => {
      queued = false;
      const palette = readPalette();
      const key = JSON.stringify(palette);
      if (key === last) return;
      last = key;
      callback(palette);
    });
  };
  const media = window.matchMedia('(prefers-color-scheme: dark)');
  const observer = new MutationObserver(check);
  observer.observe(document.documentElement, { attributes: true, attributeFilter: ['data-theme'] });
  window.addEventListener('themechange', check);
  media.addEventListener('change', check);
  return () => {
    observer.disconnect();
    window.removeEventListener('themechange', check);
    media.removeEventListener('change', check);
  };
}
