// Plain English descriptions of a run, shared by the playground and the chapters.

import type { KindDto, SnapshotDto } from '#lib/engine/pkg/turing_wasm.js';

const integer = new Intl.NumberFormat('en-US');

/** 1234567 as 1,234,567. */
export function count(n: number): string {
  return integer.format(n);
}

/** A rate such as 12.3 million steps per second. */
export function rate(stepsPerSecond: number): string {
  if (!Number.isFinite(stepsPerSecond) || stepsPerSecond <= 0) return '';
  if (stepsPerSecond >= 1e9) return `${(stepsPerSecond / 1e9).toFixed(1)} billion steps/s`;
  if (stepsPerSecond >= 1e6) return `${(stepsPerSecond / 1e6).toFixed(1)} million steps/s`;
  return `${count(Math.round(stepsPerSecond))} steps/s`;
}

export function plural(n: number, one: string, many = `${one}s`): string {
  return `${count(n)} ${n === 1 ? one : many}`;
}

function clip(word: string, max = 32): string {
  const chars = Array.from(word);
  return chars.length > max ? `${chars.slice(0, max).join('')}...` : word;
}

export type StopKind = 'budget' | 'halted' | 'breakpoint' | 'never-halts' | 'paused' | null;

/** The result part of the status line, for example "Halted, accepted". */
export function outcome(
  snapshot: SnapshotDto,
  kind: KindDto,
  options: { running?: boolean; stop?: StopKind; rate?: number; next?: string } = {}
): string {
  if (snapshot.status === 'never-halts') {
    return snapshot.reason ? `Never halts: ${snapshot.reason}` : 'Never halts';
  }
  if (snapshot.status === 'halted') {
    if (kind === 'acceptor' && snapshot.verdict) return `Halted, ${snapshot.verdict}`;
    if (kind === 'transducer' && snapshot.output) {
      const { word, value } = snapshot.output;
      const shown = word === '' ? 'the empty word' : clip(word);
      return value !== undefined && value !== word
        ? `Halted, output ${shown} (${value})`
        : `Halted, output ${shown}`;
    }
    if (kind === 'enumerator')
      return `Halted after printing ${plural(snapshot.emittedCount, 'word')}`;
    if (kind === 'beaver') return `Halted with ${plural(snapshot.ones, 'one')}`;
    return 'Halted';
  }
  if (options.running) {
    const r = rate(options.rate ?? 0);
    return r ? `Running, ${r}` : 'Running';
  }
  const next = options.next ? `next ${options.next}` : '';
  if (options.stop === 'breakpoint')
    return next ? `Stopped at a breakpoint, ${next}` : 'Stopped at a breakpoint';
  if (options.stop === 'budget') return 'Paused at the run budget';
  if (next) return `${next.charAt(0).toUpperCase()}${next.slice(1)}`;
  return snapshot.step === 0 ? 'Ready' : 'Paused';
}

/** δ(q, a) = (q', b, D) in the course notation, with tuples for several tapes. */
export function transition(t: {
  from: string;
  read: string[];
  to: string;
  write: string[];
  dir: string[];
}): string {
  const tuple = (xs: string[]) => (xs.length === 1 ? xs[0]! : `(${xs.join(', ')})`);
  return `δ(${t.from}, ${tuple(t.read)}) = (${t.to}, ${tuple(t.write)}, ${tuple(t.dir)})`;
}

/**
 * A configuration `u q v` shortened around the state: at most `keep` symbols of u and of v,
 * with an ellipsis for the rest. Each line is one tape.
 */
export function clipConfiguration(configuration: string, state: string, keep = 40): string {
  return configuration
    .split('\n')
    .map((line) => {
      if (line.length <= 2 * keep + state.length + 2) return line;
      const at = line.startsWith(`${state} `) ? 0 : line.indexOf(` ${state} `) + 1;
      if (at <= 0 && !line.startsWith(`${state} `)) return `${line.slice(0, 2 * keep)}…`;
      const u = line.slice(0, Math.max(0, at - 1));
      const v = line.slice(at + state.length + 1);
      const left = u.length > keep ? `…${u.slice(-keep)}` : u;
      const right = v.length > keep ? `${v.slice(0, keep)}…` : v;
      return [left, state, right].filter((x) => x !== '').join(' ');
    })
    .join('\n');
}
