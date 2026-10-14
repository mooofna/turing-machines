<!--
  Fig. 1.1: the binary increment machine of the first lecture, stepped one transition at a time.

  The reader enters a binary number with the least significant bit first. The tape shows the
  head and the state; Step applies one transition (the primary action), Back undoes one, Reset
  returns to the start configuration. Below, the configurations reached so far form a derivation
  in the course notation, and the transition table outlines the entry the next step applies.
  When the machine halts, the output is read as a binary number and converted to decimal.
-->
<script lang="ts">
  import type { MachineSpecDto, SnapshotDto } from '#lib/engine/pkg/turing_wasm.js';
  import { count } from '#lib/machine/describe.ts';
  import { SessionClient } from '#lib/machine/session.ts';
  import TapeView from '#lib/machine/TapeView.svelte';
  import type { FigureProps } from '../types.ts';
  import { errorText, gallerySpec, inView, scrollRegion } from './kit.ts';
  import Plate from './Plate.svelte';
  import TransitionGrid from './TransitionGrid.svelte';

  let { id }: FigureProps = $props();
  const uid = $props.id();

  /** Inputs longer than this would leave the tape window on phones without adding anything. */
  const MAX_BITS = 16;
  /** Configurations shown in full before the derivation is shortened with ⊢*. */
  const SHOWN = 4;

  let spec = $state.raw<MachineSpecDto | null>(null);
  let snapshot = $state.raw<SnapshotDto | null>(null);
  /** trail[k] is the configuration after k steps, for every k up to the current step. */
  let trail = $state.raw<string[]>([]);
  let text = $state('1101');
  let word = $state('1101');
  let radius = $state(12);
  let failure = $state<string | null>(null);
  let started = $state(false);
  let session: SessionClient | null = null;
  const visible = inView(start);
  const wide = scrollRegion('Transition table, scrolls sideways');

  const problem = $derived.by(() => {
    if (/[^01]/.test(text)) return 'Use only the digits 0 and 1.';
    if (text.length > MAX_BITS) return `Use at most ${MAX_BITS} digits.`;
    return null;
  });

  /** The value of a binary word with the least significant bit first. */
  function lsbValue(bits: string): number {
    let value = 0;
    for (let i = bits.length - 1; i >= 0; i--) value = value * 2 + (bits[i] === '1' ? 1 : 0);
    return value;
  }

  const inputValue = $derived(problem ? null : lsbValue(text));
  const halted = $derived(snapshot?.status === 'halted');
  const tape = $derived(snapshot?.tapes[0]);
  const read = $derived(tape ? (tape.cells[tape.head - tape.start] ?? '␣') : null);
  const at = $derived(snapshot && read ? { state: snapshot.state, read } : null);
  const next = $derived(
    snapshot?.nextTransition !== undefined ? spec?.transitions[snapshot.nextTransition] : undefined
  );

  const derivation = $derived.by(() => {
    if (trail.length <= SHOWN) return { head: null, tail: trail, offset: 0 };
    return { head: trail[0]!, tail: trail.slice(-(SHOWN - 1)), offset: trail.length - SHOWN + 1 };
  });

  const glyph = (g: string) => (g === '␣' || g === '_' ? '␣' : g);

  function apply(s: SnapshotDto | null) {
    if (!s) return;
    snapshot = s;
    trail = [...trail.slice(0, s.step), s.configuration];
  }

  async function open() {
    if (!spec || !session) return;
    try {
      const s = await session.open(spec, word, radius);
      if (!s) return;
      snapshot = s;
      trail = [s.configuration];
      failure = null;
    } catch (error) {
      failure = errorText(error);
    }
  }

  async function start() {
    started = true;
    try {
      spec = await gallerySpec('binary-increment');
      session = new SessionClient();
      await open();
    } catch (error) {
      failure = errorText(error);
    }
  }

  $effect(() => () => session?.close());

  function edit(value: string) {
    text = value.trim();
    if (problem || text === word) return;
    word = text;
    void open();
  }

  async function step() {
    if (!session || halted) return;
    apply(await session.step(1, radius).catch(fail));
  }

  async function back() {
    if (!session || !snapshot?.step) return;
    apply(await session.back(1, radius).catch(fail));
  }

  async function reset() {
    if (!session) return;
    apply(await session.seek(0, radius).catch(fail));
  }

  function fail(error: unknown) {
    failure = errorText(error);
    return null;
  }

  function resize(r: number) {
    if (r === radius) return;
    radius = r;
    if (session && snapshot) void session.snapshot(r).then((s) => s && (snapshot = s));
  }
</script>

<Plate {id} busy={!snapshot && !failure} attach={visible} class="increment">
  <div class="input-row">
    <label class="fig-label" for="{uid}-input">Binary number, least significant bit first</label>
    <div class="field-line">
      <input
        id="{uid}-input"
        class="fig-field bits"
        value={text}
        inputmode="numeric"
        autocomplete="off"
        spellcheck="false"
        autocapitalize="off"
        maxlength={MAX_BITS + 4}
        aria-invalid={problem ? 'true' : undefined}
        aria-describedby="{uid}-hint"
        oninput={(e) => edit(e.currentTarget.value)}
        onkeydown={(e) => {
          if (e.key === 'Enter') {
            e.preventDefault();
            void step();
          }
        }}
      />
      <span id="{uid}-hint" class="fig-hint" class:problem>
        {#if problem}{problem}{:else}<span class="mono value">= {count(inputValue ?? 0)}</span>{/if}
      </span>
    </div>
  </div>

  <div class="tape">
    <TapeView
      tapes={snapshot?.tapes ?? null}
      current={snapshot?.state ?? 'q0'}
      slide={140}
      onradius={resize}
    />
  </div>

  <div class="actions">
    <div class="buttons">
      <button
        class="button primary"
        type="button"
        onclick={step}
        disabled={!snapshot || halted || !!problem}>Step</button
      >
      <button class="button" type="button" onclick={back} disabled={!snapshot?.step}>Back</button>
      <button class="button" type="button" onclick={reset} disabled={!snapshot?.step}>Reset</button>
    </div>
    <p class="counter">
      <span class="fig-label">Step</span><span class="mono"
        >{snapshot ? count(snapshot.step) : ''}</span
      >
    </p>
  </div>

  <div class="detail">
    <div class="run">
      <p class="fig-label">Configurations</p>
      <p class="derivation mono">
        {#if derivation.head !== null}
          <span class="config">{derivation.head}</span>
          <span class="turnstile" aria-hidden="true">⊢<sup>*</sup></span><span
            class="visually-hidden">yields in several steps</span
          >
        {/if}
        {#each derivation.tail as config, i (derivation.offset + i)}
          {#if i > 0}<span class="turnstile" aria-hidden="true">⊢</span><span
              class="visually-hidden">yields</span
            >{/if}
          <span class="config" class:now={i === derivation.tail.length - 1}>{config}</span>
        {/each}
      </p>

      <p class="fig-status outcome" aria-live="polite">
        {#if failure}
          The engine could not run this machine: {failure}
        {:else if !snapshot}
          {started ? 'Loading the machine.' : ''}
        {:else if halted}
          <strong>Halted</strong> after {snapshot.step === 1
            ? 'one step'
            : `${count(snapshot.step)} steps`}:
          <span class="mono">δ({snapshot.state}, {glyph(read ?? '␣')})</span>
          is undefined. Output
          <span class="mono out">{snapshot.output?.word ?? ''}</span>, which is
          <strong class="mono">{snapshot.output?.value ?? ''}</strong>.
        {:else if next}
          Next: <span class="mono"
            >δ({next.from}, {glyph(next.read[0] ?? '')}) = ({next.to}, {glyph(next.write[0] ?? '')}, {next
              .dir[0]})</span
          >
        {/if}
      </p>
    </div>

    <div class="table">
      <p class="fig-label">Transitions</p>
      <div class="grid-scroll" {@attach wide}>
        <TransitionGrid {spec} {at} label="Transition table of the increment machine" />
      </div>
    </div>
  </div>
</Plate>

<style>
  .input-row {
    margin-bottom: var(--space-2);
  }

  .field-line {
    display: flex;
    flex-wrap: wrap;
    align-items: center;
    gap: 0.4rem 0.9rem;
  }

  .bits {
    width: 13rem;
    letter-spacing: 0.08em;
  }

  .value {
    font-size: var(--text-sm);
    color: var(--text);
  }

  .tape {
    margin-inline: calc(-1 * var(--space-3));
    padding-inline: var(--space-3);
    border-block: 1px solid var(--rule);
    padding-block: var(--space-2) 0.25rem;
  }

  .actions {
    display: flex;
    flex-wrap: wrap;
    align-items: center;
    justify-content: space-between;
    gap: var(--space-2);
    margin-top: var(--space-2);
  }

  .buttons {
    display: flex;
    gap: 0.5rem;
  }

  .buttons .button {
    min-width: 5rem;
    justify-content: center;
  }

  .counter {
    display: flex;
    align-items: baseline;
    gap: 0.6rem;
    margin: 0;
  }

  .counter .fig-label {
    margin: 0;
  }

  .counter .mono {
    min-width: 2ch;
    font-size: var(--text-sm);
  }

  .detail {
    display: grid;
    grid-template-columns: minmax(0, 1fr) auto;
    gap: var(--space-3);
    margin-top: var(--space-3);
  }

  .derivation {
    display: flex;
    flex-wrap: wrap;
    align-items: baseline;
    gap: 0.15rem 0.6rem;
    min-height: 1.6rem;
    margin: 0 0 var(--space-2);
    font-size: var(--text-sm);
    color: var(--muted);
  }

  .config {
    white-space: pre;
  }

  .config.now {
    color: var(--text);
  }

  .turnstile {
    color: var(--muted);
  }

  .turnstile sup {
    font-size: 0.75em;
    line-height: 0;
  }

  .outcome {
    max-width: 30rem;
  }

  .outcome .mono {
    color: var(--text);
  }

  .out {
    letter-spacing: 0.04em;
  }

  .table {
    min-width: 0;
  }

  @media (max-width: 47.99rem) {
    .tape {
      margin-inline: 0;
      padding-inline: 0;
    }

    .actions {
      flex-wrap: nowrap;
    }

    .buttons .button {
      min-width: 0;
      padding-inline: 0.85rem;
    }

    .detail {
      grid-template-columns: minmax(0, 1fr);
      gap: var(--space-2);
    }

    .grid-scroll {
      overflow-x: auto;
    }
  }
</style>
