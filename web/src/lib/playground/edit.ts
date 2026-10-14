// Structural edits of a machine. Every function returns a new spec and leaves its argument
// untouched; validation of names and symbols is separate so the UI can explain problems before
// it edits anything.

import type {
  DirDto,
  KindDto,
  MachineSpecDto,
  TransitionDto
} from '#lib/engine/pkg/turing_wasm.js';

export type Spec = MachineSpecDto;

export interface Action {
  to: string;
  write: string[];
  dir: DirDto[];
}

function copy(spec: Spec): Spec {
  return structuredClone(spec);
}

/** The spec without source positions, which no longer match after an edit. */
export function withoutSpans(spec: Spec): Spec {
  const next = copy(spec);
  for (const t of next.transitions) delete t.span;
  return next;
}

export function emptyMachine(): Spec {
  return {
    name: '',
    kind: 'acceptor',
    tapes: 1,
    blank: '␣',
    inputAlphabet: [],
    tapeAlphabet: ['␣'],
    states: [{ name: 'q0', isFinal: false }],
    start: 'q0',
    transitions: [],
    input: '',
    output: { kind: 'word' },
    declared: { states: true, inputAlphabet: true, tapeAlphabet: true, start: true, blank: false },
    comments: { header: [], trailing: [] }
  };
}

export function stateNames(spec: Spec): string[] {
  return spec.states.map((s) => s.name);
}

export function findTransition(spec: Spec, from: string, read: string[]): number {
  return spec.transitions.findIndex(
    (t) => t.from === from && t.read.length === read.length && t.read.every((g, i) => g === read[i])
  );
}

const RESERVED = new Set(['δ', 'delta']);

/** Why `name` cannot be a new state name, or null if it can. */
export function stateNameProblem(spec: Spec, name: string, current?: string): string | null {
  if (name === '') return 'Enter a name.';
  if (/\s/u.test(name)) return 'A state name cannot contain spaces.';
  if (/[(){},="*/]/u.test(name)) return 'Use letters, digits, _ or an apostrophe.';
  if (RESERVED.has(name)) return `${name} is reserved for transitions.`;
  if (Array.from(name).length > 16) return 'Use at most 16 characters.';
  if (name !== current && spec.states.some((s) => s.name === name))
    return `There is already a state ${name}.`;
  return null;
}

/** Why `glyph` cannot be added to Γ, or null if it can. */
export function symbolProblem(spec: Spec, glyph: string): string | null {
  const chars = Array.from(glyph);
  if (chars.length === 0) return 'Enter one character.';
  if (chars.length > 1) return 'A symbol is a single character.';
  if (/\s/u.test(glyph)) return 'A symbol cannot be a space; the blank is ␣.';
  if (/[(){},="*]/u.test(glyph)) return `${glyph} is part of the notation and cannot be a symbol.`;
  if (glyph === '_' && spec.blank === '␣') return '_ stands for the blank ␣.';
  if (spec.tapeAlphabet.includes(glyph)) return `${glyph} is already in Γ.`;
  return null;
}

/** A fresh state name in the style of the existing ones: q3 after q2, D after C. */
export function nextStateName(spec: Spec): string {
  const names = new Set(stateNames(spec));
  const letters = spec.states.length > 0 && spec.states.every((s) => /^[A-Z]$/.test(s.name));
  if (letters) {
    for (const c of 'ABCDEFGHIJKLMNOPQRSTUVWXY') if (!names.has(c)) return c;
  }
  const numbers = spec.states
    .map((s) => /^q(\d+)$/.exec(s.name)?.[1])
    .filter((n): n is string => n !== undefined)
    .map(Number);
  let n = numbers.length ? Math.max(...numbers) + 1 : spec.states.length;
  while (names.has(`q${n}`)) n++;
  return `q${n}`;
}

/**
 * Adds a state. `declare` writes Q explicitly, which keeps a state without transitions in the
 * description; a state that is the target of a transition does not need it.
 */
export function addState(spec: Spec, name: string, declare = true): Spec {
  if (spec.states.some((s) => s.name === name)) return spec;
  const next = copy(spec);
  next.states.push({ name, isFinal: false });
  if (declare) next.declared.states = true;
  return next;
}

export function renameState(spec: Spec, from: string, to: string): Spec {
  const next = copy(spec);
  const rename = (n: string) => (n === from ? to : n);
  for (const s of next.states) s.name = rename(s.name);
  for (const t of next.transitions) {
    t.from = rename(t.from);
    t.to = rename(t.to);
  }
  next.start = rename(next.start);
  return next;
}

/** Removes a state and every transition from or to it. The last state cannot be removed. */
export function removeState(spec: Spec, name: string): Spec {
  if (spec.states.length <= 1) return spec;
  const next = copy(spec);
  next.states = next.states.filter((s) => s.name !== name);
  next.transitions = next.transitions.filter((t) => t.from !== name && t.to !== name);
  if (next.start === name) next.start = next.states[0]!.name;
  return next;
}

export function setStart(spec: Spec, name: string): Spec {
  const next = copy(spec);
  next.start = name;
  next.declared.start = true;
  return next;
}

export function toggleFinal(spec: Spec, name: string): Spec {
  const next = copy(spec);
  for (const s of next.states) if (s.name === name) s.isFinal = !s.isFinal;
  return next;
}

export function addSymbol(spec: Spec, glyph: string, input: boolean): Spec {
  const next = copy(spec);
  if (!next.tapeAlphabet.includes(glyph)) next.tapeAlphabet.push(glyph);
  next.declared.tapeAlphabet = true;
  if (input && glyph !== next.blank && !next.inputAlphabet.includes(glyph)) {
    next.inputAlphabet.push(glyph);
    next.declared.inputAlphabet = true;
  }
  return next;
}

/** Transitions that read or write `glyph`, which removing it would delete. */
export function usesOfSymbol(spec: Spec, glyph: string): number[] {
  return spec.transitions.flatMap((t, i) =>
    t.read.includes(glyph) || t.write.includes(glyph) ? [i] : []
  );
}

export function removeSymbol(spec: Spec, glyph: string): Spec {
  if (glyph === spec.blank) return spec;
  const next = copy(spec);
  next.tapeAlphabet = next.tapeAlphabet.filter((g) => g !== glyph);
  next.inputAlphabet = next.inputAlphabet.filter((g) => g !== glyph);
  next.transitions = next.transitions.filter(
    (t) => !t.read.includes(glyph) && !t.write.includes(glyph)
  );
  next.declared.tapeAlphabet = true;
  next.declared.inputAlphabet = true;
  return next;
}

export function setInputSymbol(spec: Spec, glyph: string, input: boolean): Spec {
  if (glyph === spec.blank) return spec;
  const next = copy(spec);
  next.inputAlphabet = input
    ? next.tapeAlphabet.filter((g) => g === glyph || next.inputAlphabet.includes(g))
    : next.inputAlphabet.filter((g) => g !== glyph);
  next.declared.inputAlphabet = true;
  return next;
}

function ensureStates(spec: Spec, names: string[]) {
  for (const name of names) {
    if (!spec.states.some((s) => s.name === name)) spec.states.push({ name, isFinal: false });
  }
}

function ensureSymbols(spec: Spec, glyphs: string[]) {
  for (const g of glyphs) {
    if (!spec.tapeAlphabet.includes(g)) {
      spec.tapeAlphabet.push(g);
    }
  }
}

/**
 * Defines δ(from, read) = action, replacing an existing definition, or removes it when
 * `action` is null. A new transition goes after the last one of the same state, so the
 * description stays grouped by state. Missing states and symbols are added.
 */
export function setTransition(
  spec: Spec,
  from: string,
  read: string[],
  action: Action | null
): Spec {
  const next = copy(spec);
  const index = findTransition(next, from, read);
  if (action === null) {
    if (index >= 0) next.transitions.splice(index, 1);
    return next;
  }
  ensureStates(next, [from, action.to]);
  ensureSymbols(next, [...read, ...action.write]);
  if (index >= 0) {
    const t = next.transitions[index]!;
    t.to = action.to;
    t.write = [...action.write];
    t.dir = [...action.dir];
    return next;
  }
  const t: TransitionDto = {
    from,
    read: [...read],
    to: action.to,
    write: [...action.write],
    dir: [...action.dir],
    leading: []
  };
  let at = -1;
  next.transitions.forEach((x, i) => {
    if (x.from === from) at = i;
  });
  if (at < 0) {
    // Keep the order of the states: after the last transition of an earlier state.
    const order = stateNames(next);
    const rank = order.indexOf(from);
    next.transitions.forEach((x, i) => {
      if (order.indexOf(x.from) < rank) at = i;
    });
  }
  next.transitions.splice(at + 1, 0, t);
  return next;
}

/** Replaces transition `index` entirely (it may change its state or read symbols). */
export function replaceTransition(spec: Spec, index: number, t: Omit<TransitionDto, 'leading'>) {
  const old = spec.transitions[index];
  if (!old) return spec;
  if (old.from !== t.from || old.read.join('\u0001') !== t.read.join('\u0001')) {
    const clash = findTransition(spec, t.from, t.read);
    if (clash >= 0 && clash !== index) return spec;
  }
  const next = copy(spec);
  ensureStates(next, [t.from, t.to]);
  ensureSymbols(next, [...t.read, ...t.write]);
  const target = next.transitions[index]!;
  target.from = t.from;
  target.read = [...t.read];
  target.to = t.to;
  target.write = [...t.write];
  target.dir = [...t.dir];
  return next;
}

export function removeTransition(spec: Spec, index: number): Spec {
  const next = copy(spec);
  next.transitions.splice(index, 1);
  return next;
}

export function setName(spec: Spec, name: string): Spec {
  const next = copy(spec);
  next.name = name;
  return next;
}

export function setKind(spec: Spec, kind: KindDto): Spec {
  const next = copy(spec);
  next.kind = kind;
  return next;
}

function sorted(items: string[]) {
  return [...items].sort().join('\u0001');
}

/**
 * Whether two specs describe the same machine, ignoring source positions, comments and the
 * order of the alphabets. Used to check that formatting and parsing again kept every edit.
 */
export function sameMachine(a: Spec, b: Spec): boolean {
  const transitions = (s: Spec) =>
    s.transitions.map((t) => [t.from, t.read, t.to, t.write, t.dir].flat().join('\u0002'));
  return (
    a.name === b.name &&
    a.kind === b.kind &&
    a.tapes === b.tapes &&
    a.blank === b.blank &&
    a.start === b.start &&
    a.input === b.input &&
    JSON.stringify(a.output) === JSON.stringify(b.output) &&
    sorted(a.inputAlphabet) === sorted(b.inputAlphabet) &&
    sorted(a.tapeAlphabet) === sorted(b.tapeAlphabet) &&
    a.states.map((s) => `${s.name}:${s.isFinal}`).join() ===
      b.states.map((s) => `${s.name}:${s.isFinal}`).join() &&
    transitions(a).join('\u0003') === transitions(b).join('\u0003')
  );
}

/** A key that changes whenever running the machine could give a different result. */
export function behaviourKey(spec: Spec): string {
  return JSON.stringify([
    spec.kind,
    spec.tapes,
    spec.blank,
    spec.start,
    spec.inputAlphabet,
    spec.tapeAlphabet,
    spec.states,
    spec.output,
    spec.transitions.map((t) => [t.from, t.read, t.to, t.write, t.dir])
  ]);
}
