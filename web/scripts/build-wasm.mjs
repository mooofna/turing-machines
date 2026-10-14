// Builds the Rust crate turing-wasm for the browser and writes the JavaScript glue and the
// TypeScript definitions to src/lib/engine/pkg.
//
// Steps: cargo build (profile wasm-release), wasm-bindgen --target web, wasm-opt -O3, then a size
// check. The wasm-bindgen CLI must have exactly the version of the wasm-bindgen crate in
// Cargo.lock, otherwise the generated glue does not match the binary.

import { execFileSync } from 'node:child_process';
import { readFileSync, statSync } from 'node:fs';
import { gzipSync } from 'node:zlib';
import { dirname, join, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';

const web = resolve(dirname(fileURLToPath(import.meta.url)), '..');
const root = resolve(web, '..');
const out = join(web, 'src/lib/engine/pkg');
const wasm = join(root, 'target/wasm32-unknown-unknown/wasm-release/turing_wasm.wasm');

// Size budget of the optimised module (bytes).
const BUDGET_RAW = 600 * 1024;
const BUDGET_GZIP = 220 * 1024;

/**
 * @param {string} command
 * @param {string[]} args
 * @param {string} [cwd]
 */
function run(command, args, cwd = root) {
  execFileSync(command, args, { cwd, stdio: 'inherit' });
}

function lockedBindgenVersion() {
  const lock = readFileSync(join(root, 'Cargo.lock'), 'utf8');
  const match = lock.match(/name = "wasm-bindgen"\nversion = "([^"]+)"/);
  if (!match) throw new Error('wasm-bindgen not found in Cargo.lock');
  return match[1];
}

function cliBindgenVersion() {
  try {
    return execFileSync('wasm-bindgen', ['--version'], { encoding: 'utf8' }).trim().split(/\s+/)[1];
  } catch {
    return null;
  }
}

const locked = lockedBindgenVersion();
const cli = cliBindgenVersion();
if (cli !== locked) {
  console.error(
    `wasm-bindgen CLI ${cli ?? 'not found'} does not match the crate version ${locked}.\n` +
      `Install it with: cargo install wasm-bindgen-cli --version ${locked} --locked`
  );
  process.exit(1);
}

run('cargo', [
  'build',
  '-p',
  'turing-wasm',
  '--target',
  'wasm32-unknown-unknown',
  '--profile',
  'wasm-release'
]);
run('wasm-bindgen', ['--target', 'web', '--typescript', '--out-dir', out, wasm]);

const bindgenWasm = join(out, 'turing_wasm_bg.wasm');
run(
  join(web, 'node_modules/.bin/wasm-opt'),
  [
    '-O3',
    '--enable-bulk-memory',
    '--enable-nontrapping-float-to-int',
    '--enable-sign-ext',
    '--enable-mutable-globals',
    bindgenWasm,
    '-o',
    bindgenWasm
  ],
  web
);

const raw = statSync(bindgenWasm).size;
const gzip = gzipSync(readFileSync(bindgenWasm)).length;
/** @param {number} n */
const kb = (n) => `${(n / 1024).toFixed(0)} KB`;
console.log(`turing_wasm_bg.wasm: ${kb(raw)} raw, ${kb(gzip)} gzip`);
if (!process.env.SKIP_WASM_BUDGET && (raw > BUDGET_RAW || gzip > BUDGET_GZIP)) {
  console.error(`over budget (${kb(BUDGET_RAW)} raw, ${kb(BUDGET_GZIP)} gzip)`);
  process.exit(1);
}
