// The state of the playground: one machine, its description, its run and the diagram layout.
//
// The spec is the single source of truth. Text edits parse into a new spec (the last valid spec
// stays while the text has errors); table and diagram edits build a new spec and write the text
// again with formatSpec, which keeps comments. Every change of behaviour reopens the session at
// step 0.

import type {
  BreakpointsDto,
  DiagnosticDto,
  KindDto,
  MachineSpecDto,
  SnapshotDto,
  StopReasonDto
} from '#lib/engine/pkg/turing_wasm.js';
import type { EditorServices } from '#lib/machine/services.ts';
import { SessionClient } from '#lib/machine/session.ts';
import { buildHash, type SharedLink, type SharedMachine } from '#lib/share.ts';
import * as edit from './edit';

export type Origin = 'load' | 'text' | 'table' | 'diagram';

export interface Point {
  x: number;
  y: number;
}

export type Positions = Record<string, Point>;

export type Selection = { kind: 'state'; name: string } | { kind: 'transition'; index: number };

export type Breakpoint =
  { kind: 'state'; state: string } | { kind: 'transition'; from: string; read: string[] };

/** Why the last run stopped: the engine's reason, the run budget, or the user. */
export type StopReason = StopReasonDto | 'paused';

export interface Notice {
  id: number;
  text: string;
  undo: boolean;
}

interface Entry {
  source: string;
  spec: MachineSpecDto | null;
  positions: Positions;
  origin: Origin;
  galleryId: string | null;
}

/** Slider positions 0 to 100; 100 is as fast as the engine can go. */
export const SPEED_MAX = 100;

/** Steps per second at slider position `speed`: 1, 10, 100, 1000 at 0, 25, 50, 75. */
export function stepsPerSecond(speed: number): number {
  return speed >= SPEED_MAX ? Infinity : Math.max(1, Math.round(10 ** (speed / 25)));
}

/** Runs longer than this open the space-time diagram. */
export const LONG_RUN = 200;

const UNDO_LIMIT = 100;

function message(error: unknown): string {
  return error instanceof Error ? error.message : String(error);
}

export class Playground {
  // The machine.
  source = $state('');
  spec = $state.raw<MachineSpecDto | null>(null);
  diagnostics = $state.raw<DiagnosticDto[]>([]);
  origin = $state<Origin>('load');
  galleryId = $state<string | null>(null);

  // The run.
  input = $state('');
  snapshot = $state.raw<SnapshotDto | null>(null);
  running = $state(false);
  speed = $state(25);
  budget = $state(10_000_000);
  stop = $state<StopReason | null>(null);
  engineError = $state<string | null>(null);
  measuredRate = $state(0);
  /** Set when a run went past LONG_RUN steps; the page opens the space-time diagram once. */
  longRun = $state(0);

  // The editors.
  positions = $state.raw<Positions>({});
  selection = $state.raw<Selection | null>(null);
  breakpoints = $state.raw<Breakpoint[]>([]);
  radius = $state(16);
  notice = $state.raw<Notice | null>(null);
  canUndo = $state(false);
  canRedo = $state(false);
  ready = $state(false);
  /** Increases whenever a new machine is loaded, so views can fit it. */
  loads = $state(0);

  readonly hasErrors = $derived(this.diagnostics.some((d) => d.severity === 'error'));
  /** The table and diagram show the last valid machine while the text has errors. */
  readonly stale = $derived(this.hasErrors && this.spec !== null);
  readonly inputProblem = $derived.by(() => this.#inputProblem(this.spec, this.input));

  #svc: EditorServices | null = null;
  #session: SessionClient | null = null;
  #undo: Entry[] = [];
  #redo: Entry[] = [];
  #textStreak = false;
  #runToken = 0;
  #frame = 0;
  #noticeId = 0;
  #noticeTimer: ReturnType<typeof setTimeout> | undefined;

  get services(): EditorServices {
    if (!this.#svc) throw new Error('the editor services have not loaded');
    return this.#svc;
  }

  attach(svc: EditorServices): void {
    this.#svc = svc;
    this.#session = new SessionClient();
    this.ready = true;
  }

  detach(): void {
    this.pause();
    this.#session?.close();
    this.#session = null;
    clearTimeout(this.#noticeTimer);
  }

  // Loading and text.

  /** Loads a machine from a description or a spec; clears undo, breakpoints and layout. */
  load(machine: string | MachineSpecDto, options: { galleryId?: string; input?: string } = {}) {
    const svc = this.services;
    this.#halt();
    let spec: MachineSpecDto | null;
    let source: string;
    let diagnostics: DiagnosticDto[];
    if (typeof machine === 'string') {
      const parsed = svc.parseSource(machine);
      source = machine;
      spec = parsed.spec ?? null;
      diagnostics = parsed.diagnostics;
    } else {
      ({ spec, source, diagnostics } = this.#formatted(machine));
    }
    this.source = source;
    this.spec = spec;
    this.diagnostics = diagnostics;
    this.origin = 'load';
    this.galleryId = options.galleryId ?? null;
    this.input = options.input ?? spec?.input ?? '';
    this.positions = spec ? this.#place(spec, {}) : {};
    this.selection = null;
    this.breakpoints = [];
    this.#undo = [];
    this.#redo = [];
    this.#textStreak = false;
    this.#syncUndo();
    this.notice = null;
    this.loads++;
    void this.#reopen();
  }

  /** Takes new text from the source editor. */
  setSource(text: string): void {
    if (text === this.source) return;
    const parsed = this.services.parseSource(text);
    if (!this.#textStreak) {
      this.#pushUndo();
      this.#textStreak = true;
    }
    this.source = text;
    this.diagnostics = parsed.diagnostics;
    const spec = parsed.spec;
    if (!spec) return;
    const previous = this.spec;
    this.origin = 'text';
    this.galleryId = null;
    if (previous && this.input === previous.input && spec.input !== previous.input) {
      this.input = spec.input;
    }
    this.spec = spec;
    this.positions = this.#place(spec, this.positions);
    if (!previous || edit.behaviourKey(previous) !== edit.behaviourKey(spec)) void this.#reopen();
  }

  /** Writes the text again from the last valid machine, dropping text with errors. */
  discardText(): void {
    if (!this.spec) return;
    const { source, diagnostics } = this.#formatted(this.spec);
    this.source = source;
    this.diagnostics = diagnostics;
    this.#textStreak = false;
  }

  /** The canonical text of `spec`, and the spec parsed back from it if nothing was lost. */
  #formatted(next: MachineSpecDto) {
    const svc = this.services;
    const plain = edit.withoutSpans(next);
    const source = svc.formatSpec(plain);
    const parsed = svc.parseSource(source);
    // Parsing the formatted text gives the transitions their source positions.
    if (parsed.spec && edit.sameMachine(parsed.spec, plain)) {
      return { spec: parsed.spec, source, diagnostics: parsed.diagnostics };
    }
    if (import.meta.env.DEV) {
      console.warn('format and parse do not round-trip', parsed.diagnostics, source);
    }
    return { spec: plain, source, diagnostics: [] as DiagnosticDto[] };
  }

  // Structural edits.

  #commit(next: MachineSpecDto, origin: Origin, notice?: string, at?: Positions): void {
    const previous = this.spec;
    if (!previous || next === previous) return;
    this.#pushUndo();
    const { spec, source, diagnostics } = this.#formatted(next);
    this.spec = spec;
    this.source = source;
    this.diagnostics = diagnostics;
    this.origin = origin;
    this.galleryId = null;
    this.#textStreak = false;
    this.positions = this.#place(spec, { ...this.positions, ...at });
    if (notice) this.#notify(notice, true);
    if (edit.behaviourKey(previous) !== edit.behaviourKey(spec)) void this.#reopen();
  }

  /** Defines or removes δ(from, read). */
  setTransition(
    from: string,
    read: string[],
    action: edit.Action | null,
    origin: Origin = 'table',
    at?: Positions
  ): void {
    if (!this.spec) return;
    const known = new Set(edit.stateNames(this.spec));
    const created = action && !known.has(action.to) ? action.to : null;
    const removed = action === null && edit.findTransition(this.spec, from, read) >= 0;
    const label = `δ(${from}, ${read.join(', ')})`;
    const notice = created
      ? `Added state ${created} and ${label}.`
      : removed
        ? `Removed ${label}.`
        : undefined;
    this.#commit(edit.setTransition(this.spec, from, read, action), origin, notice, at);
  }

  editTransition(
    index: number,
    patch: Partial<
      Pick<MachineSpecDto['transitions'][number], 'from' | 'read' | 'to' | 'write' | 'dir'>
    >,
    origin: Origin = 'table'
  ): void {
    const t = this.spec?.transitions[index];
    if (!this.spec || !t) return;
    this.#commit(edit.replaceTransition(this.spec, index, { ...t, ...patch }), origin);
  }

  addTransition(
    from: string,
    read: string[],
    action: edit.Action,
    origin: Origin = 'diagram',
    at?: Positions
  ): void {
    this.setTransition(from, read, action, origin, at);
  }

  removeTransition(index: number, origin: Origin = 'table'): void {
    const t = this.spec?.transitions[index];
    if (!this.spec || !t) return;
    this.#commit(
      edit.removeTransition(this.spec, index),
      origin,
      `Removed δ(${t.from}, ${t.read.join(', ')}).`
    );
  }

  /** Adds a state, optionally at a diagram position; returns its name. */
  addState(name?: string, at?: Point, origin: Origin = 'table'): string | null {
    if (!this.spec) return null;
    const state = name ?? edit.nextStateName(this.spec);
    if (edit.stateNameProblem(this.spec, state)) return null;
    this.#commit(edit.addState(this.spec, state), origin, undefined, at ? { [state]: at } : {});
    return state;
  }

  renameState(from: string, to: string, origin: Origin = 'table'): string | null {
    if (!this.spec || from === to) return null;
    const problem = edit.stateNameProblem(this.spec, to, from);
    if (problem) return problem;
    const positions = { ...this.positions };
    if (positions[from]) {
      positions[to] = positions[from];
      delete positions[from];
    }
    this.positions = positions;
    this.breakpoints = this.breakpoints.map((b) =>
      b.kind === 'state' && b.state === from
        ? { ...b, state: to }
        : b.kind === 'transition' && b.from === from
          ? { ...b, from: to }
          : b
    );
    if (this.selection?.kind === 'state' && this.selection.name === from) {
      this.selection = { kind: 'state', name: to };
    }
    this.#commit(edit.renameState(this.spec, from, to), origin);
    return null;
  }

  removeState(name: string, origin: Origin = 'table'): void {
    if (!this.spec || this.spec.states.length <= 1) return;
    const count = this.spec.transitions.filter((t) => t.from === name || t.to === name).length;
    const notice =
      count === 0
        ? `Deleted state ${name}.`
        : `Deleted state ${name} and ${count} transition${count === 1 ? '' : 's'}.`;
    this.breakpoints = this.breakpoints.filter(
      (b) => (b.kind === 'state' ? b.state : b.from) !== name
    );
    if (this.selection?.kind === 'state' && this.selection.name === name) this.selection = null;
    this.#commit(edit.removeState(this.spec, name), origin, notice);
  }

  setStart(name: string, origin: Origin = 'table'): void {
    if (this.spec && this.spec.start !== name) this.#commit(edit.setStart(this.spec, name), origin);
  }

  toggleFinal(name: string, origin: Origin = 'table'): void {
    if (this.spec) this.#commit(edit.toggleFinal(this.spec, name), origin);
  }

  addSymbol(glyph: string, input = true, origin: Origin = 'table'): string | null {
    if (!this.spec) return null;
    const problem = edit.symbolProblem(this.spec, glyph);
    if (problem) return problem;
    this.#commit(edit.addSymbol(this.spec, glyph, input), origin);
    return null;
  }

  removeSymbol(glyph: string, origin: Origin = 'table'): void {
    if (!this.spec || glyph === this.spec.blank) return;
    const uses = edit.usesOfSymbol(this.spec, glyph).length;
    const notice =
      uses === 0
        ? `Removed symbol ${glyph}.`
        : `Removed symbol ${glyph} and ${uses} transition${uses === 1 ? '' : 's'}.`;
    this.#commit(edit.removeSymbol(this.spec, glyph), origin, notice);
  }

  setInputSymbol(glyph: string, input: boolean, origin: Origin = 'table'): void {
    if (this.spec) this.#commit(edit.setInputSymbol(this.spec, glyph, input), origin);
  }

  setName(name: string): void {
    if (this.spec && this.spec.name !== name) this.#commit(edit.setName(this.spec, name), 'table');
  }

  setKind(kind: KindDto): void {
    if (this.spec && this.spec.kind !== kind) this.#commit(edit.setKind(this.spec, kind), 'table');
  }

  /** A new machine with one state and only the blank symbol. */
  newMachine(): void {
    const had = this.spec !== null;
    const before = had ? this.#entry() : null;
    this.load(edit.emptyMachine());
    if (before) {
      this.#undo = [before];
      this.#syncUndo();
      this.#notify('Started a new machine.', true);
    }
  }

  // Diagram positions.

  movePositions(next: Positions): void {
    this.positions = next;
  }

  /** Places states without a position (new ones) where the layout puts them, avoiding overlap. */
  #place(spec: MachineSpecDto, known: Positions): Positions {
    const out: Positions = {};
    const missing: string[] = [];
    for (const s of spec.states) {
      const p = known[s.name];
      if (p) out[s.name] = p;
      else missing.push(s.name);
    }
    if (missing.length === 0) return out;
    const layout = this.services.graphLayout(spec);
    const fresh = Object.keys(out).length === 0;
    for (const node of layout.nodes) {
      if (!missing.includes(node.name)) continue;
      let { x, y } = node;
      if (!fresh) {
        const gap = layout.radius * 3;
        while (Object.values(out).some((p) => Math.hypot(p.x - x, p.y - y) < gap)) x += gap;
      }
      out[node.name] = { x, y };
    }
    return out;
  }

  /** Lays the diagram out again from scratch. */
  arrange(): void {
    if (!this.spec) return;
    this.positions = this.#place(this.spec, {});
    this.loads++;
  }

  // Undo.

  #entry(): Entry {
    return {
      source: this.source,
      spec: this.spec,
      positions: this.positions,
      origin: this.origin,
      galleryId: this.galleryId
    };
  }

  #pushUndo(): void {
    this.#undo.push(this.#entry());
    if (this.#undo.length > UNDO_LIMIT) this.#undo.shift();
    this.#redo = [];
    this.#syncUndo();
  }

  #syncUndo(): void {
    this.canUndo = this.#undo.length > 0;
    this.canRedo = this.#redo.length > 0;
  }

  #restore(entry: Entry): void {
    const previous = this.spec;
    this.source = entry.source;
    this.spec = entry.spec;
    this.diagnostics = this.services.parseSource(entry.source).diagnostics;
    this.positions = entry.positions;
    this.origin = entry.origin;
    this.galleryId = entry.galleryId;
    this.#textStreak = false;
    this.notice = null;
    const changed =
      !previous || !entry.spec || edit.behaviourKey(previous) !== edit.behaviourKey(entry.spec);
    if (changed) void this.#reopen();
  }

  undo(): void {
    const entry = this.#undo.pop();
    if (!entry) return;
    this.#redo.push(this.#entry());
    this.#restore(entry);
    this.#syncUndo();
  }

  redo(): void {
    const entry = this.#redo.pop();
    if (!entry) return;
    this.#undo.push(this.#entry());
    this.#restore(entry);
    this.#syncUndo();
  }

  #notify(text: string, undo: boolean): void {
    clearTimeout(this.#noticeTimer);
    const id = ++this.#noticeId;
    this.notice = { id, text, undo };
    this.#noticeTimer = setTimeout(() => {
      if (this.notice?.id === id) this.notice = null;
    }, 12_000);
  }

  dismissNotice(): void {
    this.notice = null;
  }

  // Input.

  #inputProblem(spec: MachineSpecDto | null, input: string): string | null {
    if (!spec) return null;
    const sigma = spec.inputAlphabet;
    for (const ch of Array.from(input)) {
      if (!sigma.includes(ch)) {
        const set = sigma.length ? `{${sigma.join(', ')}}` : 'empty';
        return ch === ' '
          ? `The input cannot contain spaces. Σ is ${set}.`
          : `${ch} is not in the input alphabet. Σ is ${set}.`;
      }
    }
    return null;
  }

  setInput(text: string): void {
    if (text === this.input) return;
    this.input = text;
    void this.#reopen();
  }

  // The run.

  #breakpoints(): BreakpointsDto {
    const spec = this.spec;
    const transitions: number[] = [];
    const states: string[] = [];
    for (const b of this.breakpoints) {
      if (b.kind === 'state') states.push(b.state);
      else if (spec) {
        const i = edit.findTransition(spec, b.from, b.read);
        if (i >= 0) transitions.push(i);
      }
    }
    return { transitions, states };
  }

  toggleBreakpoint(b: Breakpoint): void {
    const same = (x: Breakpoint) =>
      x.kind === b.kind &&
      (x.kind === 'state'
        ? x.state === (b as { state: string }).state
        : b.kind === 'transition' && x.from === b.from && x.read.join() === b.read.join());
    this.breakpoints = this.breakpoints.some(same)
      ? this.breakpoints.filter((x) => !same(x))
      : [...this.breakpoints, b];
  }

  hasBreakpoint(b: Breakpoint): boolean {
    return this.breakpoints.some(
      (x) =>
        x.kind === b.kind &&
        (x.kind === 'state'
          ? b.kind === 'state' && x.state === b.state
          : b.kind === 'transition' && x.from === b.from && x.read.join() === b.read.join())
    );
  }

  async #reopen(): Promise<void> {
    this.#halt();
    this.stop = null;
    this.engineError = null;
    const spec = this.spec;
    const session = this.#session;
    if (!spec || !session || this.inputProblem) {
      this.snapshot = null;
      return;
    }
    try {
      const snapshot = await session.open(spec, this.input, this.radius);
      if (snapshot) this.snapshot = snapshot;
    } catch (error) {
      this.snapshot = null;
      this.engineError = message(error);
    }
  }

  async #apply(request: Promise<SnapshotDto | null> | undefined): Promise<void> {
    if (!request) return;
    try {
      const snapshot = await request;
      if (snapshot) this.snapshot = snapshot;
    } catch (error) {
      this.engineError = message(error);
    }
  }

  setRadius(radius: number): void {
    if (radius === this.radius) return;
    this.radius = radius;
    if (!this.running && this.snapshot) void this.#apply(this.#session?.snapshot(radius));
  }

  get canStep(): boolean {
    return this.snapshot?.status === 'running';
  }

  get canBack(): boolean {
    return (this.snapshot?.step ?? 0) > 0;
  }

  async step(): Promise<void> {
    if (!this.canStep) return;
    this.pause();
    this.stop = null;
    await this.#apply(this.#session?.step(1, this.radius));
  }

  async back(): Promise<void> {
    if (!this.canBack) return;
    this.pause();
    this.stop = null;
    await this.#apply(this.#session?.back(1, this.radius));
  }

  async reset(): Promise<void> {
    this.pause();
    this.stop = null;
    await this.#apply(this.#session?.seek(0, this.radius));
  }

  async seek(step: number): Promise<void> {
    this.pause();
    this.stop = null;
    await this.#apply(this.#session?.seek(Math.max(0, Math.floor(step)), this.radius));
  }

  /** Stops the run loops without touching the visible state. */
  #halt(): void {
    this.#runToken++;
    cancelAnimationFrame(this.#frame);
    this.#session?.stop();
    this.running = false;
  }

  pause(): void {
    if (!this.running) return;
    this.#halt();
    this.stop = 'paused';
  }

  toggleRun(): void {
    if (this.running) this.pause();
    else void this.run();
  }

  async run(): Promise<void> {
    if (this.running || !this.snapshot || !this.#session) return;
    if (this.snapshot.status !== 'running') await this.reset();
    if (!this.snapshot || this.snapshot.status !== 'running') return;
    const token = ++this.#runToken;
    this.running = true;
    this.stop = null;
    const start = this.snapshot.step;
    // Breakpoints are checked before every step: step past the one the run stopped at.
    const next = this.snapshot.nextTransition;
    if (next !== undefined && this.#breakpoints().transitions.includes(next)) {
      await this.#apply(this.#session.step(1, this.radius));
      if (token !== this.#runToken) return;
    }
    this.#drive(token, start);
  }

  /** Changes the speed; a running machine switches between frame steps and the worker run. */
  setSpeed(speed: number): void {
    const wasMax = this.speed >= SPEED_MAX;
    this.speed = speed;
    const isMax = speed >= SPEED_MAX;
    if (this.running && wasMax !== isMax && this.snapshot) {
      const start = this.snapshot.step;
      this.#runToken++;
      cancelAnimationFrame(this.#frame);
      this.#session?.stop();
      this.#drive(this.#runToken, start);
    }
  }

  #drive(token: number, start: number): void {
    if (this.speed >= SPEED_MAX) void this.#runInWorker(token, start);
    else this.#runInFrames(token, start);
  }

  #finish(token: number, reason: StopReason): void {
    if (token !== this.#runToken) return;
    this.#runToken++;
    this.running = false;
    this.stop = reason;
    if ((this.snapshot?.step ?? 0) > LONG_RUN) this.longRun++;
  }

  #runInFrames(token: number, start: number): void {
    const session = this.#session;
    if (!session) return;
    let last = performance.now();
    let due = 1;
    let busy = false;
    const frame = (now: number) => {
      if (token !== this.#runToken) return;
      const rate = stepsPerSecond(this.speed);
      due = Math.min(due + ((now - last) / 1000) * rate, rate / 4 + 1);
      last = now;
      if (!busy && due >= 1) {
        const done = (this.snapshot?.step ?? start) - start;
        const count = Math.min(Math.floor(due), Math.max(1, this.budget - done));
        due -= count;
        busy = true;
        session
          .run(count, this.#breakpoints(), this.radius)
          .then((outcome) => {
            busy = false;
            if (!outcome || token !== this.#runToken) return;
            this.snapshot = outcome.result.snapshot;
            const reason = outcome.result.reason;
            if (reason !== 'budget') this.#finish(token, reason);
            else if (outcome.result.snapshot.step - start >= this.budget)
              this.#finish(token, 'budget');
          })
          .catch((error: unknown) => {
            this.engineError = message(error);
            this.#finish(token, 'paused');
          });
      }
      this.#frame = requestAnimationFrame(frame);
    };
    this.#frame = requestAnimationFrame(frame);
  }

  async #runInWorker(token: number, start: number): Promise<void> {
    const session = this.#session;
    if (!session) return;
    const done = (this.snapshot?.step ?? start) - start;
    try {
      const outcome = await session.run(
        Math.max(1, this.budget - done),
        this.#breakpoints(),
        this.radius,
        (progress) => {
          if (token !== this.#runToken) return;
          this.snapshot = progress.snapshot;
          this.measuredRate = progress.stepsPerSecond;
        }
      );
      // A paused run still reports where it stopped.
      if (!outcome) return;
      this.snapshot = outcome.result.snapshot;
      if (outcome.elapsedMs > 0) {
        this.measuredRate = (outcome.result.stepsDone / outcome.elapsedMs) * 1000;
      }
      this.#finish(token, outcome.result.reason);
    } catch (error) {
      this.engineError = message(error);
      this.#finish(token, 'paused');
    }
  }

  // Sharing.

  shareLink(): SharedLink | null {
    const spec = this.spec;
    if (!spec) return null;
    const svc = this.services;
    let machine: SharedMachine;
    const plain = spec.name === '' && spec.comments.header.length === 0;
    const standard = spec.kind === 'beaver' && plain ? svc.toStandard(spec) : undefined;
    if (this.galleryId) machine = { kind: 'gallery', id: this.galleryId };
    else if (standard) machine = { kind: 'standard', text: standard };
    else machine = { kind: 'source', payload: svc.encodeShare(spec) };
    return this.input === spec.input ? { machine } : { machine, input: this.input };
  }

  /** The address of this machine and input. */
  shareUrl(): string | null {
    const link = this.shareLink();
    if (!link) return null;
    const url = new URL(window.location.href);
    url.hash = buildHash(link);
    return url.href;
  }
}
