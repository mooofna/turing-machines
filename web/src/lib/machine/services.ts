// Synchronous editor services (parse, format, layout, routing, share links) on the main thread.
//
// All machine semantics live in Rust; this module only gives the WebAssembly functions a typed,
// non-throwing surface. Load it from onMount or an event handler, never during prerendering.

import type {
  DiagnosticDto,
  EdgeRouteDto,
  GraphLayoutDto,
  MachineSpecDto,
  ParseResultDto
} from '#lib/engine/pkg/turing_wasm.js';
import { editor, type Wasm } from '#lib/engine/wasm.ts';
import { dsl, graph, stubbedServices } from './fallback';

export type LayoutKind = 'auto' | 'layered' | 'circular';

export interface EditorServices {
  parseSource(source: string): ParseResultDto;
  /** The canonical description of `spec`; comments are kept. */
  formatSpec(spec: MachineSpecDto): string;
  checkSpec(spec: MachineSpecDto): DiagnosticDto[];
  graphLayout(spec: MachineSpecDto, kind?: LayoutKind): GraphLayoutDto;
  /** One route per edge of `graphLayout(spec)`, for node centres in the order of `spec.states`. */
  graphRoute(spec: MachineSpecDto, xs: Float64Array, ys: Float64Array): EdgeRouteDto[];
  encodeShare(spec: MachineSpecDto): string;
  decodeShare(payload: string): ParseResultDto;
  fromStandard(text: string): ParseResultDto;
  /** The standard format, or undefined if the machine cannot be written in it. */
  toStandard(spec: MachineSpecDto): string | undefined;
}

function failure(error: unknown): ParseResultDto {
  return {
    spec: undefined,
    diagnostics: [
      {
        severity: 'error',
        code: 'E000',
        message: error instanceof Error ? error.message : String(error),
        related: []
      }
    ]
  };
}

function safe(run: () => ParseResultDto): ParseResultDto {
  try {
    return run();
  } catch (error) {
    return failure(error);
  }
}

function build(w: Wasm): EditorServices {
  const stub = stubbedServices(w);
  return {
    parseSource: (source) => safe(() => (stub.has('parse') ? dsl.parse : w.parseSource)(source)),
    formatSpec: (spec) =>
      stub.has('format') ? dsl.format(spec, false) : w.formatSpec(spec, false),
    checkSpec: (spec) => {
      try {
        return w.checkSpec(spec).diagnostics;
      } catch (error) {
        return failure(error).diagnostics;
      }
    },
    graphLayout: (spec, kind = 'auto') =>
      stub.has('graph') ? graph.layout(spec) : w.graphLayout(spec, kind),
    graphRoute: (spec, xs, ys) =>
      (stub.has('graph') ? graph.route(spec, xs, ys) : w.graphRoute(spec, xs, ys)).routes,
    encodeShare: (spec) => (stub.has('share') ? dsl.encodeShare(spec) : w.encodeShare(spec)),
    decodeShare: (payload) =>
      safe(() => (stub.has('share') ? dsl.decodeShare : w.decodeShare)(payload)),
    fromStandard: (text) =>
      safe(() => (stub.has('standard') ? dsl.fromStandard : w.fromStandard)(text)),
    toStandard: (spec) => {
      try {
        return stub.has('standard') ? dsl.toStandard(spec) : w.toStandard(spec);
      } catch {
        return undefined;
      }
    }
  };
}

let loading: Promise<EditorServices> | null = null;

/** The editor services, after the WebAssembly module has loaded. */
export function services(): Promise<EditorServices> {
  loading ??= editor().then(build);
  return loading;
}
