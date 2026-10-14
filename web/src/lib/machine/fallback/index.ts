// DEVELOPMENT FALLBACK FOR THE STUB ENGINE.
//
// While the Rust engine is being written, the WebAssembly module is a stub: its parser returns a
// fixed machine, its formatter drops declarations, its session walks over the input and its
// layout puts states on a line. That is too little to develop the playground against, so each
// stubbed service is detected here by a behavioural probe and replaced by a small TypeScript
// version. Once the real engine is built every probe fails, nothing here is used, and this
// folder can be deleted together with the two imports of it (lib/machine/services.ts and
// lib/engine/engine.worker.ts).

import type { MachineSpecDto } from '#lib/engine/pkg/turing_wasm.js';
import type { Wasm } from '#lib/engine/wasm.ts';
import { emptySpec } from './dsl';
import { FallbackSession, type SessionLike } from './sim';

export { FallbackSession, type SessionLike };
export * as dsl from './dsl';
export * as graph from './graph';

export type Stubbed = 'parse' | 'format' | 'share' | 'standard' | 'graph' | 'session';

function probeSpec(): MachineSpecDto {
  const spec = emptySpec();
  spec.states = [
    { name: 's', isFinal: false },
    { name: 't', isFinal: false },
    { name: 'u', isFinal: false }
  ];
  spec.start = 's';
  spec.tapeAlphabet = ['␣', 'x'];
  spec.transitions = [
    { from: 's', read: ['␣'], to: 't', write: ['x'], dir: ['L'], leading: [], comment: 'probe' },
    { from: 't', read: ['␣'], to: 'u', write: ['x'], dir: ['L'], leading: [] },
    { from: 'u', read: ['␣'], to: 's', write: ['x'], dir: ['L'], leading: [] },
    { from: 'u', read: ['x'], to: 'u', write: ['x'], dir: ['R'], leading: [] }
  ];
  return spec;
}

function attempt(test: () => boolean): boolean {
  try {
    return test();
  } catch {
    return true;
  }
}

/** Which editor services of `w` are still the stub. */
export function stubbedServices(w: Wasm): Set<Stubbed> {
  const stubbed = new Set<Stubbed>();
  const spec = probeSpec();
  if (attempt(() => !w.parseSource('δ(s, a) = (t, b, L)').spec?.states.some((s) => s.name === 's')))
    stubbed.add('parse');
  if (attempt(() => !w.formatSpec(spec, false).includes('probe'))) stubbed.add('format');
  if (attempt(() => w.encodeShare(spec) === '1.stub')) stubbed.add('share');
  if (attempt(() => w.fromStandard('1RB1LB_1LA1RZ').spec?.name === '1RB1LB_1LA1RZ'))
    stubbed.add('standard');
  if (
    attempt(() => {
      const routes = w.graphRoute(
        spec,
        new Float64Array([0, 100, 0]),
        new Float64Array([0, 0, 100])
      );
      const loop = routes.routes.find((r) => r.edge === 3);
      return loop?.path === 'M -10 80 c -20 -50 20 -50 10 0';
    })
  )
    stubbed.add('graph');
  return stubbed;
}

/** Whether the WebAssembly session ignores the machine. */
export function sessionIsStub(w: Wasm): boolean {
  return attempt(() => {
    const s = new w.Session(probeSpec(), '');
    try {
      return s.step(1, 0).state !== 't';
    } finally {
      s.free();
    }
  });
}
