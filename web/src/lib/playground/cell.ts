// The text of one cell of the transition table: `q2, X, R` in the course notation, or `1RB` in
// the compact notation of busy beaver machines.

import type { DirDto, TransitionDto } from '#lib/engine/pkg/turing_wasm.js';

export interface CellContext {
  states: string[];
  gamma: string[];
  blank: string;
  /** The symbol of the cell's column, which `*` stands for. */
  read: string;
  compact: boolean;
}

export type CellResult =
  | { kind: 'empty' }
  | {
      kind: 'ok';
      to: string;
      write: string;
      dir: DirDto;
      /** A state or symbol the cell names that does not exist yet. */
      missing: { state?: string; symbol?: string };
    }
  | { kind: 'error'; message: string };

const DIRS = new Set(['L', 'R', 'N']);

/** How a glyph is shown: the blank as ␣ even if the machine writes it as _. */
export function glyph(g: string): string {
  return g === '_' ? '␣' : g;
}

export function showCell(t: TransitionDto | undefined, compact: boolean): string {
  if (!t) return '';
  const write = glyph(t.write[0] ?? '');
  const dir = t.dir[0] ?? '';
  return compact ? `${write}${dir}${t.to}` : `${t.to}, ${write}, ${dir}`;
}

function symbol(token: string, ctx: CellContext): string {
  if (token === '*') return ctx.read;
  if ((token === '_' || token === '␣') && !ctx.gamma.includes(token)) return ctx.blank;
  return token;
}

function finish(to: string, write: string, dirText: string, ctx: CellContext): CellResult {
  const dir = dirText.toUpperCase();
  if (!DIRS.has(dir)) return { kind: 'error', message: `${dirText} is not a move. Use L, R or N.` };
  if (Array.from(write).length !== 1)
    return { kind: 'error', message: `${write} is not a single symbol.` };
  const missing: { state?: string; symbol?: string } = {};
  if (!ctx.states.includes(to)) missing.state = to;
  if (!ctx.gamma.includes(write)) missing.symbol = write;
  return { kind: 'ok', to, write, dir: dir as DirDto, missing };
}

export function parseCell(text: string, ctx: CellContext): CellResult {
  let s = text.trim();
  if (s === '' || s === '---' || s === '-') return { kind: 'empty' };
  if (s.startsWith('(') && s.endsWith(')')) s = s.slice(1, -1).trim();
  const parts = s.split(/\s*,\s*|\s+/u).filter((p) => p !== '');
  if (parts.length === 3) {
    const [to, write, dir] = parts as [string, string, string];
    return finish(to, symbol(write, ctx), dir, ctx);
  }
  if (parts.length === 1) {
    // Compact notation: write symbol, move, next state, as in 1RB.
    const m = /^(.)([LRNlrn])(.+)$/u.exec(s);
    if (m) return finish(m[3]!, symbol(m[1]!, ctx), m[2]!, ctx);
  }
  const example = ctx.compact ? '1RB' : `${ctx.states[0] ?? 'q0'}, ${ctx.read}, R`;
  if (parts.length === 2)
    return {
      kind: 'error',
      message: `Give the next state, the symbol to write and a move, as in ${example}.`
    };
  return { kind: 'error', message: `Write the next state, a symbol and a move, as in ${example}.` };
}
