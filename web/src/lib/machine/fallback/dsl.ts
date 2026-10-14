// DEVELOPMENT FALLBACK, see ./index.ts. A small parser and formatter for the description
// language, used only while the WebAssembly module is the stub whose parser returns a fixed
// machine. It follows docs/dsl.md closely enough to develop the playground against; the Rust
// parser is the reference and replaces this file entirely.

import type {
  DecoderDto,
  DiagnosticDto,
  DirDto,
  KindDto,
  MachineSpecDto,
  ParseResultDto,
  TransitionDto
} from '#lib/engine/pkg/turing_wasm.js';

interface Token {
  text: string;
  from: number;
  to: number;
  string?: boolean;
}

const PUNCT = new Set(['(', ')', '{', '}', ',', '=']);
const KINDS: KindDto[] = ['acceptor', 'transducer', 'enumerator', 'beaver'];

function tokenize(line: string, offset: number): { tokens: Token[]; comment?: string } {
  const tokens: Token[] = [];
  let i = 0;
  while (i < line.length) {
    const c = line[i]!;
    if (c === ' ' || c === '\t' || c === '\r') {
      i++;
    } else if (c === '/' && line[i + 1] === '/') {
      return {
        tokens,
        comment: line
          .slice(i + 2)
          .replace(/^ /, '')
          .trimEnd()
      };
    } else if (c === '"') {
      const end = line.indexOf('"', i + 1);
      const stop = end < 0 ? line.length : end + 1;
      tokens.push({
        text: line.slice(i + 1, end < 0 ? line.length : end),
        from: offset + i,
        to: offset + stop,
        string: true
      });
      i = stop;
    } else if (PUNCT.has(c)) {
      tokens.push({ text: c, from: offset + i, to: offset + i + 1 });
      i++;
    } else {
      let j = i;
      while (j < line.length && !/[\s(){},="]/.test(line[j]!) && !line.startsWith('//', j)) j++;
      tokens.push({ text: line.slice(i, j), from: offset + i, to: offset + j });
      i = j;
    }
  }
  return { tokens };
}

class Cursor {
  i = 0;
  constructor(readonly tokens: Token[]) {}
  peek(): Token | undefined {
    return this.tokens[this.i];
  }
  next(): Token | undefined {
    return this.tokens[this.i++];
  }
  eat(text: string): boolean {
    if (this.peek()?.text === text && !this.peek()?.string) {
      this.i++;
      return true;
    }
    return false;
  }
  expect(text: string): Token {
    const t = this.peek();
    if (!t || t.text !== text || t.string) throw new Problem(`expected '${text}'`, t);
    this.i++;
    return t;
  }
  word(what: string): Token {
    const t = this.peek();
    if (!t || PUNCT.has(t.text) || t.string) throw new Problem(`expected ${what}`, t);
    this.i++;
    return t;
  }
  done(): boolean {
    return this.i >= this.tokens.length;
  }
}

class Problem extends Error {
  constructor(
    message: string,
    readonly token?: Token
  ) {
    super(message);
  }
}

/** A list `{a, b}`, a tuple `(a, b)` or a single word. */
function group(c: Cursor, what: string): { items: Token[]; open?: string } {
  const open = c.peek()?.text;
  if (open === '{' || open === '(') {
    const close = open === '{' ? '}' : ')';
    c.next();
    const items: Token[] = [];
    if (c.eat(close)) return { items, open };
    for (;;) {
      items.push(c.word(what));
      if (c.eat(close)) return { items, open };
      c.expect(',');
    }
  }
  return { items: [c.word(what)] };
}

function isDelta(text: string) {
  return text === 'δ' || text === 'delta';
}

function dir(t: Token): DirDto {
  const d = t.text.toUpperCase();
  if (d === 'L' || d === 'R' || d === 'N') return d;
  throw new Problem(`'${t.text}' is not a move; use L, R or N`, t);
}

interface Line {
  text: string;
  from: number;
}

function lines(source: string): Line[] {
  const out: Line[] = [];
  let from = 0;
  for (const text of source.split('\n')) {
    out.push({ text, from });
    from += text.length + 1;
  }
  return out;
}

function position(source: string, offset: number) {
  const before = source.slice(0, offset);
  const line = before.split('\n').length;
  const column = offset - before.lastIndexOf('\n');
  return { line, column };
}

export function emptySpec(): MachineSpecDto {
  return {
    name: '',
    kind: 'transducer',
    tapes: 1,
    blank: '␣',
    inputAlphabet: [],
    tapeAlphabet: [],
    states: [],
    start: '',
    transitions: [],
    input: '',
    output: { kind: 'word' },
    declared: {
      states: false,
      inputAlphabet: false,
      tapeAlphabet: false,
      start: false,
      blank: false
    },
    comments: { header: [], trailing: [] }
  };
}

export function parse(source: string): ParseResultDto {
  const spec = emptySpec();
  const diagnostics: DiagnosticDto[] = [];
  const error = (message: string, from: number, to: number, code = 'E001') => {
    diagnostics.push({
      severity: 'error',
      code,
      message,
      from,
      to,
      ...position(source, from),
      related: []
    });
  };

  let declaredQ: string[] | null = null;
  let declaredGamma: string[] | null = null;
  let declaredSigma: string[] | null = null;
  let finals: string[] = [];
  let seenStatement = false;
  let pending: string[] = [];
  const tapesAt: { tokens: Token[]; t: TransitionDto }[] = [];

  for (const line of lines(source)) {
    const { tokens, comment } = tokenize(line.text, line.from);
    if (tokens.length === 0) {
      if (comment !== undefined) {
        if (!seenStatement) spec.comments.header.push(comment);
        else pending.push(comment);
      }
      continue;
    }
    seenStatement = true;
    const c = new Cursor(tokens);
    try {
      const head = c.next()!;
      if (isDelta(head.text)) {
        c.expect('(');
        const from = c.word('a state');
        c.expect(',');
        const read = group(c, 'a symbol');
        c.expect(')');
        c.expect('=');
        c.expect('(');
        const to = c.word('a state');
        c.expect(',');
        const write = group(c, 'a symbol');
        c.expect(',');
        const dirs = group(c, 'a move').items.map(dir);
        const close = c.expect(')');
        if (!c.done()) throw new Problem('unexpected text after the transition', c.peek());
        // A set {a, b} as the read symbol stands for one transition per element.
        const reads: string[][] =
          read.open === '{' ? read.items.map((r) => [r.text]) : [read.items.map((r) => r.text)];
        for (const r of reads) {
          const w = write.items.map((x, i) => (x.text === '*' ? (r[i] ?? x.text) : x.text));
          const t: TransitionDto = {
            from: from.text,
            read: r,
            to: to.text,
            write: w,
            dir: dirs,
            leading: pending,
            span: { from: head.from, to: close.to }
          };
          if (comment !== undefined) t.comment = comment;
          pending = [];
          spec.transitions.push(t);
          tapesAt.push({ tokens, t });
        }
        continue;
      }
      switch (head.text) {
        case 'machine':
          spec.name = c.next()?.text ?? '';
          break;
        case 'kind': {
          const k = c.word('a kind');
          if (!KINDS.includes(k.text as KindDto)) throw new Problem(`unknown kind '${k.text}'`, k);
          spec.kind = k.text as KindDto;
          break;
        }
        case 'input':
          spec.input = c.next()?.text ?? '';
          break;
        case 'output': {
          const k = c.word('word, unary or binary').text;
          let decoder: DecoderDto = { kind: 'word' };
          if (k === 'unary') decoder = { kind: 'unary', symbol: c.word('a symbol').text };
          else if (k === 'binary') {
            const o = c.peek()?.text === 'lsb' ? 'lsb' : 'msb';
            if (!c.done()) c.next();
            decoder = { kind: 'binary', order: o };
          }
          spec.output = decoder;
          break;
        }
        case 'tapes': {
          c.eat('=');
          const n = Number(c.word('a number').text);
          if (!(n >= 1 && n <= 3)) throw new Problem('a machine has 1 to 3 tapes', head);
          spec.tapes = n;
          break;
        }
        case 'blank':
          c.expect('=');
          spec.blank = c.word('a symbol').text;
          spec.declared.blank = true;
          break;
        case 'start':
          c.expect('=');
          spec.start = c.word('a state').text;
          spec.declared.start = true;
          break;
        case 'Q':
        case 'F':
        case 'Σ':
        case 'Sigma':
        case 'Γ':
        case 'Gamma': {
          c.expect('=');
          const items = group(c, 'an element').items.map((t) => t.text);
          if (head.text === 'Q') declaredQ = items;
          else if (head.text === 'F') finals = items;
          else if (head.text === 'Σ' || head.text === 'Sigma') declaredSigma = items;
          else declaredGamma = items;
          break;
        }
        default:
          throw new Problem(`unknown declaration '${head.text}'`, head);
      }
      if (!c.done()) throw new Problem('unexpected text', c.peek());
    } catch (e) {
      if (!(e instanceof Problem)) throw e;
      const t = e.token ?? tokens[tokens.length - 1]!;
      error(e.message, t.from, Math.max(t.to, t.from + 1));
    }
  }
  spec.comments.trailing = pending;

  // Blank aliases.
  const blank = spec.blank;
  const norm = (g: string) => (g === '_' || g === '␣' ? blank : g);
  for (const t of spec.transitions) {
    t.read = t.read.map(norm);
    t.write = t.write.map(norm);
  }

  // Tape count consistency.
  for (const { tokens, t } of tapesAt) {
    if (
      t.read.length !== spec.tapes ||
      t.write.length !== spec.tapes ||
      t.dir.length !== spec.tapes
    ) {
      const first = tokens[0]!;
      error(
        `this transition is for ${t.read.length} tape${t.read.length === 1 ? '' : 's'}, the machine has ${spec.tapes}`,
        first.from,
        tokens[tokens.length - 1]!.to,
        'E006'
      );
    }
  }

  // States.
  const states: string[] = [];
  const addState = (s: string) => {
    if (s && !states.includes(s)) states.push(s);
  };
  if (declaredQ) {
    spec.declared.states = true;
    declaredQ.forEach(addState);
    for (const { tokens, t } of tapesAt) {
      for (const s of [t.from, t.to]) {
        if (!declaredQ.includes(s)) {
          const tok = tokens.find((x) => x.text === s)!;
          error(`'${s}' is not in Q`, tok.from, tok.to, 'E002');
        }
      }
    }
  } else {
    addState(spec.start);
    for (const t of spec.transitions) {
      addState(t.from);
      addState(t.to);
    }
    finals.forEach(addState);
  }
  if (!spec.start) spec.start = states[0] ?? 'q0';
  addState(spec.start);
  spec.states = states.map((name) => ({ name, isFinal: finals.includes(name) }));

  // Alphabets.
  const gamma: string[] = [];
  const addSym = (g: string) => {
    if (!gamma.includes(g)) gamma.push(g);
  };
  if (declaredGamma) {
    spec.declared.tapeAlphabet = true;
    (declaredGamma as string[]).map(norm).forEach(addSym);
    addSym(blank);
  } else {
    for (const t of spec.transitions) {
      t.read.forEach(addSym);
      t.write.forEach(addSym);
    }
    (declaredSigma ?? []).forEach(addSym);
    for (const ch of spec.input) addSym(ch);
    addSym(blank);
  }
  spec.tapeAlphabet = gamma;
  if (declaredSigma) {
    spec.declared.inputAlphabet = true;
    spec.inputAlphabet = (declaredSigma as string[]).map(norm);
  } else {
    spec.inputAlphabet = gamma.filter((g) => g !== blank);
  }
  if (declaredGamma) {
    for (const { tokens, t } of tapesAt) {
      for (const g of [...t.read, ...t.write]) {
        if (!gamma.includes(g)) {
          const tok = tokens.find((x) => norm(x.text) === g) ?? tokens[0]!;
          error(`'${g}' is not in Γ`, tok.from, tok.to, 'E003');
        }
      }
    }
  }

  // Determinism.
  const seen = new Map<string, number>();
  spec.transitions.forEach((t, i) => {
    const key = `${t.from}\u0001${t.read.join('\u0001')}`;
    if (seen.has(key)) {
      const span = t.span ?? { from: 0, to: 0 };
      error(`δ(${t.from}, ${t.read.join(', ')}) is defined twice`, span.from, span.to, 'E004');
    }
    seen.set(key, i);
  });

  const hasErrors = diagnostics.some((d) => d.severity === 'error');
  return { spec: hasErrors ? undefined : spec, diagnostics };
}

function list(items: string[]) {
  return `{${items.join(', ')}}`;
}

function tuple(items: string[]) {
  return items.length === 1 ? items[0]! : `(${items.join(', ')})`;
}

export function format(spec: MachineSpecDto, ascii: boolean): string {
  const delta = ascii ? 'delta' : 'δ';
  const sym = (g: string) => (ascii && g === '␣' ? '_' : g);
  const out: string[] = [];
  for (const c of spec.comments.header) out.push(`// ${c}`.trimEnd());
  const meta: string[] = [];
  if (spec.name) meta.push(`machine "${spec.name}"`);
  meta.push(`kind ${spec.kind}`);
  if (spec.tapes > 1) meta.push(`tapes = ${spec.tapes}`);
  if (spec.input) meta.push(`input "${spec.input}"`);
  if (spec.output.kind === 'unary') meta.push(`output unary ${spec.output.symbol}`);
  if (spec.output.kind === 'binary') meta.push(`output binary ${spec.output.order}`);
  if (out.length) out.push('');
  out.push(...meta, '');
  const decl: string[] = [];
  if (spec.declared.states) decl.push(`Q = ${list(spec.states.map((s) => s.name))}`);
  if (spec.declared.inputAlphabet)
    decl.push(`${ascii ? 'Sigma' : 'Σ'} = ${list(spec.inputAlphabet.map(sym))}`);
  if (spec.declared.tapeAlphabet)
    decl.push(`${ascii ? 'Gamma' : 'Γ'} = ${list(spec.tapeAlphabet.map(sym))}`);
  if (spec.declared.blank) decl.push(`blank = ${sym(spec.blank)}`);
  if (spec.declared.start) decl.push(`start = ${spec.start}`);
  const finals = spec.states.filter((s) => s.isFinal).map((s) => s.name);
  if (finals.length) decl.push(`F = ${list(finals)}`);
  if (decl.length) out.push(...decl, '');
  for (const t of spec.transitions) {
    for (const c of t.leading ?? []) out.push(`// ${c}`.trimEnd());
    const line = `${delta}(${t.from}, ${tuple(t.read.map(sym))}) = (${t.to}, ${tuple(
      t.write.map(sym)
    )}, ${tuple(t.dir)})`;
    out.push(t.comment ? `${line}  // ${t.comment}` : line);
  }
  if (spec.comments.trailing.length) {
    out.push('');
    for (const c of spec.comments.trailing) out.push(`// ${c}`.trimEnd());
  }
  while (out.length && out[out.length - 1] === '') out.pop();
  return out.join('\n') + '\n';
}

/** The bbchallenge standard format, for example `1RB1LB_1LA1RZ`. */
export function fromStandard(text: string): ParseResultDto {
  const groups = text.trim().split('_');
  const letters = 'ABCDEFGHIJKLMNOPQRSTUVWXY';
  const symbols = groups[0] ? groups[0].length / 3 : 0;
  const bad = (message: string): ParseResultDto => ({
    spec: undefined,
    diagnostics: [{ severity: 'error', code: 'E010', message, related: [] }]
  });
  if (!Number.isInteger(symbols) || symbols < 2 || groups.some((g) => g.length !== symbols * 3))
    return bad('not a machine in the standard format');
  const spec = emptySpec();
  spec.kind = 'beaver';
  spec.blank = '0';
  spec.tapeAlphabet = Array.from({ length: symbols }, (_, i) => String(i));
  spec.inputAlphabet = [];
  spec.declared = {
    states: true,
    inputAlphabet: true,
    tapeAlphabet: true,
    start: true,
    blank: true
  };
  const names = groups.map((_, i) => letters[i]!);
  let halt = false;
  groups.forEach((g, s) => {
    for (let r = 0; r < symbols; r++) {
      const cell = g.slice(r * 3, r * 3 + 3);
      if (cell === '---') continue;
      const m = /^([0-9])([LRN])([A-Z])$/.exec(cell);
      if (!m) return;
      if (m[3] === 'Z' || m[3] === 'H') halt = true;
      spec.transitions.push({
        from: names[s]!,
        read: [String(r)],
        to: m[3] === 'H' ? 'Z' : m[3]!,
        write: [m[1]!],
        dir: [m[2] as DirDto],
        leading: []
      });
    }
  });
  if (halt) names.push('Z');
  spec.states = names.map((name) => ({ name, isFinal: false }));
  spec.start = 'A';
  return { spec, diagnostics: [] };
}

export function toStandard(spec: MachineSpecDto): string | undefined {
  if (spec.tapes !== 1) return undefined;
  const live = spec.states.filter((s) => spec.transitions.some((t) => t.from === s.name));
  const symbols = spec.tapeAlphabet;
  if (!symbols.every((g, i) => g === String(i))) return undefined;
  const letter = (name: string) => {
    const i = live.findIndex((s) => s.name === name);
    return i < 0 ? 'Z' : 'ABCDEFGHIJKLMNOPQRSTUVWXY'[i];
  };
  return live
    .map((s) =>
      symbols
        .map((g) => {
          const t = spec.transitions.find((t) => t.from === s.name && t.read[0] === g);
          if (!t || t.dir[0] === 'N') return '---';
          return `${t.write[0]}${t.dir[0]}${letter(t.to)}`;
        })
        .join('')
    )
    .join('_');
}

function toBase64Url(text: string): string {
  const bytes = new TextEncoder().encode(text);
  let bin = '';
  for (const b of bytes) bin += String.fromCharCode(b);
  return btoa(bin).replace(/\+/g, '-').replace(/\//g, '_').replace(/=+$/, '');
}

function fromBase64Url(text: string): string {
  const bin = atob(text.replace(/-/g, '+').replace(/_/g, '/'));
  return new TextDecoder().decode(Uint8Array.from(bin, (c) => c.charCodeAt(0)));
}

/** Not compressed and not compatible with the Rust encoding; marked with `0.`. */
export function encodeShare(spec: MachineSpecDto): string {
  return `0.${toBase64Url(format(spec, false))}`;
}

export function decodeShare(payload: string): ParseResultDto {
  try {
    return parse(fromBase64Url(payload.replace(/^\d+\./, '')));
  } catch {
    return {
      spec: undefined,
      diagnostics: [
        { severity: 'error', code: 'E011', message: 'this link is damaged', related: [] }
      ]
    };
  }
}
