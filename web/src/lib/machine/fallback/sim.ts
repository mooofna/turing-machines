// DEVELOPMENT FALLBACK, see ./index.ts. A plain reference simulator with the interface of the
// WebAssembly `Session`, used only while the module is the stub whose session ignores the
// machine. Semantics as in docs/dsl.md: input from position 0, head on 0, two-way infinite
// tapes, halt when δ is undefined.

import type {
  BreakpointsDto,
  MachineSpecDto,
  OutputDto,
  RunResultDto,
  SnapshotDto,
  StopReasonDto
} from '#lib/engine/pkg/turing_wasm.js';

interface Undo {
  state: string;
  heads: number[];
  writes: [tape: number, pos: number, old: string][];
  transition: number;
  emitted: boolean;
  extent: [number, number];
}

/** The interface of the WebAssembly `Session` that the worker uses. */
export interface SessionLike {
  snapshot(radius: number): SnapshotDto;
  step(count: number, radius: number): SnapshotDto;
  back(count: number, radius: number): SnapshotDto;
  seek(step: number, radius: number): SnapshotDto;
  run(maxSteps: number, breakpoints: BreakpointsDto, radius: number): RunResultDto;
  free(): void;
}

const HISTORY = 2_000_000;

export class FallbackSession implements SessionLike {
  readonly #spec: MachineSpecDto;
  readonly #input: string[];
  readonly #table = new Map<string, number>();
  #tapes: Map<number, string>[] = [];
  #heads: number[] = [];
  #state = '';
  #steps = 0;
  #history: Undo[] = [];
  #extent: [number, number] = [0, 0];
  #emitted: string[] = [];
  #emittedCount = 0;
  #last: number | undefined;

  constructor(spec: MachineSpecDto, input: string) {
    this.#spec = spec;
    this.#input = Array.from(input);
    const sigma = new Set(spec.inputAlphabet);
    for (const ch of this.#input) {
      if (!sigma.has(ch)) throw new Error(`'${ch}' is not in the input alphabet`);
    }
    spec.transitions.forEach((t, i) => this.#table.set(this.#key(t.from, t.read), i));
    this.#reset();
  }

  #key(state: string, read: string[]) {
    return `${state}\u0001${read.join('\u0001')}`;
  }

  #reset() {
    const k = this.#spec.tapes;
    this.#tapes = Array.from({ length: k }, () => new Map<number, string>());
    this.#input.forEach((ch, i) => {
      if (ch !== this.#spec.blank) this.#tapes[0]!.set(i, ch);
    });
    this.#heads = Array.from({ length: k }, () => 0);
    this.#state = this.#spec.start;
    this.#steps = 0;
    this.#history = [];
    this.#extent = [0, 0];
    this.#emitted = [];
    this.#emittedCount = 0;
    this.#last = undefined;
  }

  #read(tape: number, pos: number) {
    return this.#tapes[tape]!.get(pos) ?? this.#spec.blank;
  }

  #next(): number | undefined {
    const read = this.#heads.map((h, i) => this.#read(i, h));
    return this.#table.get(this.#key(this.#state, read));
  }

  #isFinal(state: string) {
    return this.#spec.states.some((s) => s.name === state && s.isFinal);
  }

  #word(tape = 0) {
    const cells = [...this.#tapes[tape]!.keys()].sort((a, b) => a - b);
    if (cells.length === 0) return '';
    const out: string[] = [];
    for (let p = cells[0]!; p <= cells[cells.length - 1]!; p++) out.push(this.#read(tape, p));
    return out.join('');
  }

  #apply(): boolean {
    const index = this.#next();
    if (index === undefined) return false;
    const t = this.#spec.transitions[index]!;
    const undo: Undo = {
      state: this.#state,
      heads: [...this.#heads],
      writes: [],
      transition: this.#last ?? -1,
      emitted: false,
      extent: [...this.#extent]
    };
    t.write.forEach((g, i) => {
      const pos = this.#heads[i]!;
      undo.writes.push([i, pos, this.#read(i, pos)]);
      if (g === this.#spec.blank) this.#tapes[i]!.delete(pos);
      else this.#tapes[i]!.set(pos, g);
      const d = t.dir[i];
      this.#heads[i] = pos + (d === 'L' ? -1 : d === 'R' ? 1 : 0);
    });
    this.#state = t.to;
    this.#steps++;
    this.#last = index;
    const h = this.#heads[0]!;
    this.#extent = [Math.min(this.#extent[0], h), Math.max(this.#extent[1], h)];
    if (this.#spec.kind === 'enumerator' && this.#isFinal(t.to)) {
      this.#emitted.push(this.#word().replaceAll(this.#spec.blank, ''));
      if (this.#emitted.length > 50) this.#emitted.shift();
      this.#emittedCount++;
      undo.emitted = true;
    }
    this.#history.push(undo);
    if (this.#history.length > HISTORY) this.#history.shift();
    return true;
  }

  #undo(): boolean {
    const u = this.#history.pop();
    if (!u) return false;
    for (const [tape, pos, old] of u.writes.reverse()) {
      if (old === this.#spec.blank) this.#tapes[tape]!.delete(pos);
      else this.#tapes[tape]!.set(pos, old);
    }
    this.#heads = u.heads;
    this.#state = u.state;
    this.#steps--;
    this.#extent = u.extent;
    this.#last = u.transition < 0 ? undefined : u.transition;
    if (u.emitted) {
      this.#emitted.pop();
      this.#emittedCount--;
    }
    return true;
  }

  snapshot(radius: number): SnapshotDto {
    const next = this.#next();
    const halted = next === undefined;
    const tapes = this.#heads.map((head, i) => {
      const start = head - radius;
      const cells: string[] = [];
      for (let p = start; p <= head + radius; p++) cells.push(this.#read(i, p));
      return { head, start, cells };
    });
    const configuration = this.#heads
      .map((head, i) => {
        const cells = [...this.#tapes[i]!.keys()];
        const lo = Math.min(head, ...cells);
        const hi = Math.max(head, ...cells);
        let u = '';
        let v = '';
        for (let p = lo; p < head; p++) u += this.#read(i, p);
        for (let p = head; p <= hi; p++) v += this.#read(i, p);
        return [u, this.#state, v].filter((x) => x !== '').join(' ');
      })
      .join('\n');
    let ones = 0;
    for (const g of this.#tapes[0]!.values()) if (g !== this.#spec.blank) ones++;
    const snap: SnapshotDto = {
      step: this.#steps,
      state: this.#state,
      status: halted ? 'halted' : 'running',
      tapes,
      extent: [...this.#extent],
      configuration,
      ones,
      emitted: [...this.#emitted],
      emittedCount: this.#emittedCount
    };
    if (next !== undefined) snap.nextTransition = next;
    if (this.#last !== undefined) snap.lastTransition = this.#last;
    if (halted && this.#spec.kind === 'acceptor') {
      snap.verdict = this.#isFinal(this.#state) ? 'accepted' : 'rejected';
    }
    if (halted && this.#spec.kind === 'transducer') snap.output = this.#output();
    return snap;
  }

  #output(): OutputDto {
    const blank = this.#spec.blank;
    const word = this.#word().replace(new RegExp(`^${blank}+|${blank}+$`, 'gu'), '');
    const decoder = this.#spec.output;
    if (decoder.kind === 'unary') {
      return { word, value: String(Array.from(word).filter((g) => g === decoder.symbol).length) };
    }
    if (decoder.kind === 'binary' && /^[01]+$/.test(word)) {
      const bits = decoder.order === 'msb' ? word : Array.from(word).reverse().join('');
      return { word, value: BigInt(`0b${bits}`).toString() };
    }
    return { word };
  }

  step(count: number, radius: number): SnapshotDto {
    for (let i = 0; i < count && this.#apply(); i++);
    return this.snapshot(radius);
  }

  back(count: number, radius: number): SnapshotDto {
    return this.seek(Math.max(0, this.#steps - count), radius);
  }

  seek(step: number, radius: number): SnapshotDto {
    if (step < this.#steps) {
      while (this.#steps > step && this.#undo());
      if (this.#steps > step) {
        this.#reset();
      }
    }
    while (this.#steps < step && this.#apply());
    return this.snapshot(radius);
  }

  run(maxSteps: number, breakpoints: BreakpointsDto, radius: number): RunResultDto {
    const stopBefore = new Set(breakpoints.transitions);
    const stopIn = new Set(breakpoints.states);
    let done = 0;
    let reason: StopReasonDto = 'budget';
    while (done < maxSteps) {
      const next = this.#next();
      if (next === undefined) {
        reason = 'halted';
        break;
      }
      if (stopBefore.has(next)) {
        reason = 'breakpoint';
        break;
      }
      this.#apply();
      done++;
      if (stopIn.has(this.#state)) {
        reason = 'breakpoint';
        break;
      }
    }
    if (reason === 'budget' && this.#next() === undefined) reason = 'halted';
    return { snapshot: this.snapshot(radius), reason, stepsDone: done };
  }

  free(): void {}
}
