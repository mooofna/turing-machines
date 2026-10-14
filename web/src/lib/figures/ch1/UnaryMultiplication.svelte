<!--
  Fig. 1.2: the machine A_mul from the first exercise sheet, with decimal input and output.

  The figure carries out the exercise as it was posed: two decimal numbers are encoded in unary
  as #x^m#x^n#, the machine runs on that tape, and the unary output is decoded to decimal. The
  stages are set in one line from left to right. Run (the primary action) runs the machine in the
  engine and then replays the run on the tape at a calm speed, or shows the final tape at once
  under reduced motion; the step slider moves through the run.
-->
<script lang="ts">
  import type { MachineSpecDto, SnapshotDto } from '#lib/engine/pkg/turing_wasm.js';
  import { count } from '#lib/machine/describe.ts';
  import { SessionClient } from '#lib/machine/session.ts';
  import TapeView from '#lib/machine/TapeView.svelte';
  import type { FigureProps } from '../types.ts';
  import { errorText, gallerySpec, inView, reducedMotion } from './kit.ts';
  import Plate from './Plate.svelte';

  let { id }: FigureProps = $props();
  const uid = $props.id();

  const MAX = 12;
  /** A_mul needs 23,273 steps for 12 × 12; the budget only guards against surprises. */
  const BUDGET = 10_000_000;
  /** Output words longer than this are written with an exponent. */
  const LITERAL = 24;

  let spec = $state.raw<MachineSpecDto | null>(null);
  let snapshot = $state.raw<SnapshotDto | null>(null);
  let mText = $state('3');
  let nText = $state('4');
  let m = $state(3);
  let n = $state(4);
  let result = $state.raw<{
    m: number;
    n: number;
    steps: number;
    word: string;
    value: string;
  } | null>(null);
  let playing = $state(false);
  let computing = $state(false);
  let failure = $state<string | null>(null);
  let radius = $state(12);
  let session: SessionClient | null = null;
  let frame = 0;
  let token = 0;
  const visible = inView(start);

  function parse(text: string): number | null {
    if (!/^\d{1,2}$/.test(text.trim())) return null;
    const v = Number(text.trim());
    return v <= MAX ? v : null;
  }

  const problem = $derived(
    parse(mText) === null || parse(nText) === null ? `Choose whole numbers from 0 to ${MAX}.` : null
  );
  const encoded = $derived(`#${'x'.repeat(m)}#${'x'.repeat(n)}#`);
  const total = $derived(result?.steps ?? 0);
  const step = $derived(snapshot?.step ?? 0);

  async function open() {
    if (!spec || !session) return;
    stop();
    try {
      const s = await session.open(spec, encoded, radius);
      if (s) snapshot = s;
      failure = null;
    } catch (error) {
      failure = errorText(error);
    }
  }

  async function start() {
    try {
      spec = await gallerySpec('unary-multiplication');
      session = new SessionClient();
      await open();
    } catch (error) {
      failure = errorText(error);
    }
  }

  $effect(() => () => {
    stop();
    session?.close();
  });

  function edit(which: 'm' | 'n', text: string) {
    if (which === 'm') mText = text;
    else nText = text;
    const v = parse(text);
    if (v === null) return;
    if ((which === 'm' ? m : n) === v) return;
    if (which === 'm') m = v;
    else n = v;
    result = null;
    void open();
  }

  /** Computes the run in the engine, if it is not known yet for these factors. */
  async function compute(): Promise<boolean> {
    if (!session || !spec) return false;
    if (result && result.m === m && result.n === n) return true;
    computing = true;
    try {
      const outcome = await session.run(BUDGET, { transitions: [], states: [] }, radius);
      if (!outcome) return false;
      const s = outcome.result.snapshot;
      snapshot = s;
      if (s.status !== 'halted' || !s.output) {
        failure = `the machine did not halt within ${count(BUDGET)} steps`;
        return false;
      }
      result = { m, n, steps: s.step, word: s.output.word, value: s.output.value ?? '' };
      return true;
    } catch (error) {
      failure = errorText(error);
      return false;
    } finally {
      computing = false;
    }
  }

  async function seek(target: number) {
    if (!session) return null;
    const s = await session.seek(target, radius).catch((error: unknown) => {
      failure = errorText(error);
      return null;
    });
    if (s) snapshot = s;
    return s;
  }

  function stop() {
    token++;
    cancelAnimationFrame(frame);
    playing = false;
  }

  async function run() {
    if (playing) {
      stop();
      return;
    }
    if (!(await compute()) || !result) return;
    const end = result.steps;
    if (reducedMotion()) {
      await seek(end);
      return;
    }
    // Replay from the slider position, or from the start if the run is at its end.
    const from = step >= end ? 0 : step;
    const s = await seek(from);
    if (!s) return;
    // About fifty steps a second, between two and a half and six seconds in total.
    const seconds = Math.min(6, Math.max(2.5, (end - from) / 50));
    const rate = (end - from) / seconds;
    const mine = ++token;
    playing = true;
    const t0 = performance.now();
    let busy = false;
    const tick = (now: number) => {
      if (mine !== token) return;
      const target = Math.min(end, from + Math.floor(((now - t0) / 1000) * rate));
      if (!busy && target !== snapshot?.step) {
        busy = true;
        void seek(target).then(() => {
          busy = false;
          if (mine === token && target >= end) stop();
        });
      }
      if (mine === token) frame = requestAnimationFrame(tick);
    };
    frame = requestAnimationFrame(tick);
  }

  function scrub(value: number) {
    stop();
    void seek(value);
  }

  function resize(r: number) {
    if (r === radius) return;
    radius = r;
    if (session && snapshot && !playing) void session.snapshot(r).then((s) => s && (snapshot = s));
  }

  const announce = $derived(
    result && result.m === m && result.n === n
      ? `${m} × ${n} = ${result.value}, computed by A_mul in ${count(result.steps)} steps.`
      : ''
  );
</script>

<Plate {id} busy={(!snapshot && !failure) || computing} attach={visible}>
  <div class="pipeline">
    <div class="stage factors">
      <span class="fig-label" id="{uid}-factors">Decimal input</span>
      <div class="fields" role="group" aria-labelledby="{uid}-factors">
        <input
          class="fig-field number"
          type="number"
          min="0"
          max={MAX}
          step="1"
          inputmode="numeric"
          value={mText}
          aria-label="First factor, 0 to {MAX}"
          aria-invalid={parse(mText) === null ? 'true' : undefined}
          aria-describedby="{uid}-problem"
          oninput={(e) => edit('m', e.currentTarget.value)}
          onkeydown={(e) => e.key === 'Enter' && void run()}
        />
        <span class="times" aria-hidden="true">×</span>
        <input
          class="fig-field number"
          type="number"
          min="0"
          max={MAX}
          step="1"
          inputmode="numeric"
          value={nText}
          aria-label="Second factor, 0 to {MAX}"
          aria-invalid={parse(nText) === null ? 'true' : undefined}
          aria-describedby="{uid}-problem"
          oninput={(e) => edit('n', e.currentTarget.value)}
          onkeydown={(e) => e.key === 'Enter' && void run()}
        />
      </div>
    </div>
    <span class="arrow" aria-hidden="true">→</span>
    <div class="stage">
      <span class="fig-label">Unary input</span>
      <span class="word mono">{encoded}</span>
    </div>
    <span class="arrow machine" aria-hidden="true"
      >→<span class="by"><span class="name">A<sub>mul</sub></span></span>→</span
    >
    <div class="stage">
      <span class="fig-label">Unary output</span>
      <span class="word mono" class:pending={!result}>
        {#if !result}
          not run yet
        {:else if result.word === ''}
          <span class="plain">the empty word</span>
        {:else if result.word.length <= LITERAL}
          {result.word}
        {:else}
          x<sup>{result.word.length}</sup>
        {/if}
      </span>
    </div>
    <span class="arrow" aria-hidden="true">→</span>
    <div class="stage product">
      <span class="fig-label">Decimal output</span>
      <span class="value" class:pending={!result}>
        {#if result}
          <span class="mono">{m} × {n} = <strong>{result.value}</strong></span>
        {:else}
          <span class="mono">{m} × {n} = ?</span>
        {/if}
      </span>
    </div>
  </div>
  <p id="{uid}-problem" class="fig-hint problem">
    {#if problem}{problem}{/if}
  </p>

  <div class="tape">
    <TapeView
      tapes={snapshot?.tapes ?? null}
      current={snapshot?.state ?? 'q0'}
      slide={0}
      onradius={resize}
    />
  </div>

  <div class="controls">
    <button
      class="button primary run"
      type="button"
      onclick={run}
      disabled={!snapshot || !!problem || computing}>{playing ? 'Pause' : 'Run'}</button
    >
    <label class="scrub">
      <span class="fig-label">Step</span>
      <input
        type="range"
        min="0"
        max={total}
        step="1"
        value={step}
        disabled={!result}
        aria-valuetext={result ? `Step ${count(step)} of ${count(total)}` : 'Run the machine first'}
        oninput={(e) => scrub(Number(e.currentTarget.value))}
      />
      <span class="mono position"
        >{count(step)}{#if result}<span class="of">&nbsp;of {count(total)}</span>{/if}</span
      >
    </label>
  </div>

  <p class="fig-status summary">
    {#if failure}
      The engine could not run this machine: {failure}
    {:else if result && result.m === m && result.n === n}
      <strong>{m} × {n} = {result.value}</strong> in {count(result.steps)} steps of A<sub>mul</sub>.
    {:else}
      Run the machine to multiply {m} by {n}.
    {/if}
  </p>
  <p class="visually-hidden" aria-live="polite">{announce}</p>
</Plate>

<style>
  .pipeline {
    display: flex;
    flex-wrap: wrap;
    align-items: flex-end;
    gap: var(--space-1) 0.9rem;
  }

  .stage {
    display: grid;
    min-width: 0;
  }

  .fields {
    display: flex;
    align-items: center;
    gap: 0.45rem;
  }

  .number {
    width: 3.4rem;
    text-align: center;
  }

  .times {
    color: var(--muted);
  }

  .word {
    display: flex;
    align-items: center;
    min-height: 2.25rem;
    font-size: var(--text-sm);
    letter-spacing: 0.02em;
    overflow-wrap: anywhere;
  }

  .word .plain,
  .word.pending {
    font-family: var(--font-sans);
  }

  .word sup {
    font-size: 0.72em;
    line-height: 0;
  }

  .arrow {
    display: flex;
    align-items: center;
    height: 2.25rem;
    color: var(--muted);
  }

  .arrow.machine {
    gap: 0.35rem;
  }

  .by {
    padding: 0.15rem 0.45rem;
    border: 1px solid var(--rule-strong);
    color: var(--text);
    font-size: var(--text-sm);
  }

  .name sub {
    font-size: 0.72em;
  }

  .value {
    display: flex;
    align-items: center;
    min-height: 2.25rem;
    font-size: var(--text-sm);
  }

  .value strong {
    font-weight: 600;
  }

  .pending {
    color: var(--muted);
  }

  .fig-hint {
    margin: 0.3rem 0 0;
  }

  .fig-hint:empty {
    margin: 0;
  }

  .tape {
    margin: var(--space-1) calc(-1 * var(--space-3)) 0;
    padding: var(--space-2) var(--space-3) 0.25rem;
    border-block: 1px solid var(--rule);
  }

  .controls {
    display: flex;
    flex-wrap: wrap;
    align-items: center;
    gap: var(--space-1) var(--space-3);
    margin-top: var(--space-2);
  }

  .run {
    min-width: 5.5rem;
    justify-content: center;
  }

  .scrub {
    display: flex;
    flex: 1 1 18rem;
    align-items: center;
    gap: 0.75rem;
    min-width: 0;
  }

  .scrub .fig-label {
    margin: 0;
  }

  .scrub input {
    flex: 1;
    min-width: 6rem;
  }

  .position {
    min-width: 8.5rem;
    font-size: var(--text-sm);
  }

  .of {
    color: var(--muted);
  }

  .summary {
    margin-top: var(--space-2);
  }

  .summary sub {
    font-size: 0.72em;
  }

  @media (max-width: 47.99rem) {
    .pipeline {
      display: grid;
      grid-template-columns: minmax(0, 1fr);
      gap: 0.35rem;
    }

    .arrow {
      display: none;
    }

    .stage {
      grid-template-columns: 7.5rem minmax(0, 1fr);
      align-items: center;
    }

    .stage .fig-label {
      margin: 0;
    }

    .tape {
      margin-inline: 0;
      padding-inline: 0;
    }

    .position {
      min-width: 0;
    }
  }
</style>
