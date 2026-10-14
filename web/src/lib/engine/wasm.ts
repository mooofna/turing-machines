// Loads the WebAssembly module once per realm (page or worker).
//
// The page uses the synchronous editor services (parse, format, layout) through `editor()`;
// workers use the engines. Never call this during prerendering: only from onMount, event
// handlers or workers.

import init, * as wasm from './pkg/turing_wasm.js';
import wasmUrl from './pkg/turing_wasm_bg.wasm?url';

export type Wasm = typeof wasm;

let loading: Promise<Wasm> | null = null;

export function loadWasm(): Promise<Wasm> {
  loading ??= init({ module_or_path: wasmUrl }).then(() => wasm);
  return loading;
}

/** The editor services, after the module has loaded. */
export const editor = loadWasm;
