// Integration tests of the WebAssembly module in Node, against the module built by `npm run wasm`.
//
// They exercise the exported API the way the page and the worker use it: plain objects in, plain
// objects out, positions in UTF-16 code units.

import { readFileSync } from 'node:fs';
import { beforeAll, describe, expect, it } from 'vitest';

import {
  AntihydraCheckJob,
  AntihydraWalkJob,
  EnumerationJob,
  Session,
  SpaceTimeJob,
  checkSpec,
  decide,
  decodeShare,
  encodeShare,
  formatSpec,
  fromStandard,
  graphLayout,
  graphRoute,
  initSync,
  machineCount,
  parseSource,
  toStandard,
  upArrow,
  version,
  type AntihydraRowDto,
  type BreakpointsDto,
  type MachineSpecDto,
  type RunResultDto
} from './pkg/turing_wasm.js';

const BB4 = '1RB1LB_1LA0LC_1RZ1LD_1RD0RA';
const BB5 = '1RB1LC_1RC1RB_1RD0LE_1LA1LD_1RZ0LA';

const SOURCE = `// Binary increment, least significant bit first
machine "increment"
kind transducer
input "1101"
Σ = {0, 1}
Γ = {0, 1, ␣}
F = {q1}

δ(q0, 1) = (q0, 0, R) // carry
δ(q0, 0) = (q1, 1, R)
δ(q0, ␣) = (q1, 1, R)
`;

const NO_BREAKPOINTS: BreakpointsDto = { transitions: [], states: [], onOutput: false };

beforeAll(() => {
  initSync({ module: readFileSync(new URL('./pkg/turing_wasm_bg.wasm', import.meta.url)) });
});

function parsed(source: string): MachineSpecDto {
  const result = parseSource(source);
  expect(result.diagnostics.filter((d) => d.severity === 'error')).toEqual([]);
  if (!result.spec) throw new Error('no spec');
  return result.spec;
}

function standard(text: string): MachineSpecDto {
  const spec = fromStandard(text).spec;
  if (!spec) throw new Error(`not a machine: ${text}`);
  return spec;
}

function withoutSpans(spec: MachineSpecDto): MachineSpecDto {
  return { ...spec, transitions: spec.transitions.map((t) => ({ ...t, span: undefined })) };
}

describe('editor services', () => {
  it('parses and formats the course notation', () => {
    const spec = parsed(SOURCE);
    expect(spec.name).toBe('increment');
    expect(spec.blank).toBe('␣');
    expect(spec.inputAlphabet).toEqual(['0', '1']);
    expect(spec.transitions.map((t) => t.read[0])).toEqual(['1', '0', '␣']);

    const unicode = formatSpec(spec, false);
    expect(unicode).toContain('δ(q0, ␣) = (q1, 1, R)');
    expect(unicode).toContain('Σ');
    expect(withoutSpans(parsed(unicode))).toEqual(withoutSpans(spec));
    expect(formatSpec(parsed(unicode), false)).toBe(unicode);

    const ascii = formatSpec(spec, true);
    expect(ascii).not.toMatch(/[δΣΓ␣]/);
    expect(withoutSpans(parsed(ascii))).toEqual(withoutSpans(spec));
  });

  it('reports transition spans in UTF-16 code units', () => {
    const spec = parsed(SOURCE);
    for (const t of spec.transitions) {
      expect(t.span).toBeDefined();
      const text = SOURCE.slice(t.span!.from, t.span!.to);
      expect(text.startsWith(`δ(${t.from}, ${t.read[0]})`)).toBe(true);
    }
  });

  it('reports diagnostics with UTF-16 positions, lines and columns', () => {
    const source = 'Q = {q0, q1}\nΣ = {x}\nδ(q0, ␣) = (q9, x, R)\n';
    const result = parseSource(source);
    expect(result.spec).toBeUndefined();
    const error = result.diagnostics.find(
      (d) => d.severity === 'error' && d.message.includes('q9')
    );
    expect(error).toBeDefined();
    expect(error!.line).toBe(3);
    expect(source.slice(error!.from!, error!.to!)).toContain('q9');
    const lineStart = source.split('\n').slice(0, 2).join('\n').length + 1;
    expect(error!.from! - lineStart + 1).toBe(error!.column);
  });

  it('checks specs from the editor without throwing', () => {
    const spec = parsed(SOURCE);
    expect(checkSpec(spec).diagnostics.filter((d) => d.severity === 'error')).toEqual([]);
    const broken = structuredClone(spec);
    broken.transitions[1]!.write = ['10'];
    const [diagnostic] = checkSpec(broken).diagnostics;
    expect(diagnostic?.code).toBe('E102');
    expect(diagnostic?.from).toBe(spec.transitions[1]!.span!.from);
    expect(() => formatSpec(broken, false)).toThrow(/one character/);
  });

  it('round-trips the standard format', () => {
    const spec = standard(BB4);
    expect(spec.kind).toBe('beaver');
    expect(spec.states.length).toBeGreaterThanOrEqual(4);
    expect(toStandard(spec)).toBe(BB4);
    const error = fromStandard('1RB1LB_1LA1R?');
    expect(error.spec).toBeUndefined();
    expect(error.diagnostics[0]?.code).toBe('E101');
  });

  it('encodes and decodes share links', () => {
    const spec = parsed(SOURCE);
    const payload = encodeShare(spec);
    expect(payload).toMatch(/^1\.[A-Za-z0-9_-]+$/);
    const decoded = decodeShare(payload);
    expect(decoded.diagnostics).toEqual([]);
    expect(decoded.source).toBe(formatSpec(spec, false));
    expect(withoutSpans(decoded.spec!)).toEqual(withoutSpans(spec));
    for (const bad of ['', 'hello', '2.AAAA', '1.!!!', '1.AAAAAAAA', payload.slice(0, 12)]) {
      const result = decodeShare(bad);
      expect(result.spec).toBeUndefined();
      expect(result.diagnostics[0]?.code).toBe('E100');
    }
  });

  it('lays out and routes the state diagram', () => {
    const spec = parsed(SOURCE);
    const layout = graphLayout(spec, 'auto');
    expect(layout.nodes.map((n) => n.name)).toEqual(['q0', 'q1']);
    expect(layout.nodes[0]!.isStart).toBe(true);
    const routes = graphRoute(
      spec,
      new Float64Array(layout.nodes.map((n) => n.x)),
      new Float64Array(layout.nodes.map((n) => n.y))
    );
    expect(routes.routes).toHaveLength(layout.edges.length);
    expect(() => graphLayout(spec, 'spiral')).toThrow(/unknown layout/);
    expect(() => graphRoute(spec, new Float64Array(1), new Float64Array(1))).toThrow();
  });

  it('computes towers and counts', () => {
    expect(upArrow(2, 4, 100).exact).toBe('65536');
    expect(upArrow(2, 5, 100).digits).toBe('19729');
    expect(machineCount(5, 2)).toBe('63403380965376');
  });

  it('turns malformed values into exceptions and keeps working', () => {
    expect(() => formatSpec({} as MachineSpecDto, false)).toThrow();
    expect(() => formatSpec(null as unknown as MachineSpecDto, false)).toThrow();
    expect(() => new Session({ ...parsed(SOURCE), tapes: 0 }, '')).toThrow();
    expect(() => new Session(parsed(SOURCE), '012')).toThrow(/'2'/);
    expect(() =>
      checkSpec({ ...parsed(SOURCE), kind: 'robot' } as unknown as MachineSpecDto)
    ).toThrow();
    expect(version()).toMatch(/^\d+\.\d+\.\d+/);
    expect(parseSource(SOURCE).spec).toBeDefined();
  });
});

describe('sessions', () => {
  it('steps binary increment to the halt and back', () => {
    const session = new Session(parsed(SOURCE), '1101');
    try {
      const start = session.snapshot(3);
      expect(start.configuration).toBe('q0 1101');
      expect(start.tapes[0]!.cells.join('')).toBe('␣␣␣1101');
      expect(start.nextTransition).toBe(0);

      let snapshot = start;
      while (snapshot.status === 'running') snapshot = session.step(1, 3);
      expect(snapshot.step).toBe(3);
      expect(snapshot.status).toBe('halted');
      expect(snapshot.state).toBe('q1');
      expect(snapshot.lastTransition).toBe(1);
      expect(snapshot.output?.word).toBe('0011');

      expect(session.back(1, 3).step).toBe(2);
      expect(session.back(10, 3)).toEqual(start);
      expect(session.seek(3, 3)).toEqual(snapshot);
    } finally {
      session.free();
    }
  });

  it('runs the BB(5) champion to the halt', () => {
    const session = new Session(standard(BB5), '');
    try {
      const started = performance.now();
      let steps = 0;
      let result: RunResultDto;
      do {
        result = session.run(50_000_000, NO_BREAKPOINTS, 0);
        steps += result.stepsDone;
      } while (result.reason === 'budget');
      const elapsed = performance.now() - started;
      console.log(`BB(5): ${steps} steps in ${elapsed.toFixed(0)} ms`);
      expect(result.reason).toBe('halted');
      expect(steps).toBe(47_176_870);
      expect(result.snapshot.step).toBe(47_176_870);
      expect(result.snapshot.ones).toBe(4_098);
    } finally {
      session.free();
    }
  }, 60_000);

  it('stops at breakpoints', () => {
    const session = new Session(standard(BB4), '');
    try {
      const result = session.run(1_000, { transitions: [], states: ['C'], onOutput: false }, 0);
      expect(result.reason).toBe('breakpoint');
      expect(result.snapshot.state).toBe('C');
    } finally {
      session.free();
    }
  });
});

describe('workers', () => {
  it('renders a space-time overview of BB(4)', () => {
    const job = new SpaceTimeJob(standard(BB4), '', {
      width: 200,
      height: 100,
      mode: { kind: 'overview', maxSteps: 1_000_000 }
    });
    try {
      let progress = job.advance(1_000);
      while (!progress.done) progress = job.advance(1_000);
      expect(progress.halted).toBe(true);
      expect(progress.rowsDone).toBe(100);
      const pixels = job.rgba({ background: '#ffffff', ink: '#111111', accent: '#215caf' });
      expect(pixels.length).toBe(200 * 100 * 4);
      expect(() => job.rgba({ background: 'white', ink: '#111', accent: '#215caf' })).toThrow();
    } finally {
      job.free();
    }
  });

  it('enumerates all 3-state machines', () => {
    const job = new EnumerationJob(3, 2, 0, 1);
    try {
      const started = performance.now();
      let progress = job.advance(1_000);
      while (!progress.done) progress = job.advance(1_000);
      console.log(`3-state enumeration: ${performance.now() - started} ms`);
      expect(progress.nodes).toBe(5_417);
      expect(progress.maxSteps?.value).toBe(21);
      expect(progress.maxOnes?.value).toBe(6);
      expect(progress.holdouts).toBe(0);
      expect(progress.deciders[0]?.key).toBe('loops');
      expect(job.holdouts(10).machines).toEqual([]);
    } finally {
      job.free();
    }
  }, 60_000);

  it('decides a simple cycler', () => {
    expect(decide(standard('0RB---_0LA---'))).toMatchObject({ kind: 'non-halting' });
    expect(decide(standard('1RB1LB_1LA1RZ'))).toEqual({ kind: 'halts', steps: 6, ones: 4 });
  });

  it('checks Antihydra rules on the machine', () => {
    const job = new AntihydraCheckJob();
    try {
      const rows: AntihydraRowDto[] = [];
      while (rows.length < 20) {
        const batch = job.advance(10_000);
        rows.push(...batch.rows);
        if (batch.done) break;
      }
      expect(rows.length).toBeGreaterThanOrEqual(20);
      expect(rows.every((r) => r.agrees)).toBe(true);
      expect([rows[0]!.a, rows[0]!.b]).toEqual([0, 4]);
    } finally {
      job.free();
    }
  });

  it('iterates the Antihydra rule', () => {
    const job = new AntihydraWalkJob();
    try {
      const summary = job.advance(10_000, 100);
      expect(summary.iterations).toBe(10_000);
      expect(summary.halted).toBe(false);
      expect(summary.samples.length).toBeLessThanOrEqual(100);
    } finally {
      job.free();
    }
  });
});
