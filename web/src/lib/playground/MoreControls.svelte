<!--
  The controls behind "More controls": breakpoints, the run budget, going to a step, and asking
  the deciders whether the machine never halts.
-->
<script lang="ts">
  import { batch } from '#lib/engine/client.ts';
  import type { VerdictDto } from '#lib/engine/pkg/turing_wasm.js';
  import { count } from '#lib/machine/describe.ts';
  import { glyph } from './cell';
  import type { Breakpoint, Playground } from './state.svelte';

  interface Props {
    pg: Playground;
  }

  let { pg }: Props = $props();
  const id = $props.id();

  const spec = $derived(pg.spec);
  let bpState = $state('');
  let bpSymbol = $state('');
  let target = $state('');
  let proving = $state(false);
  let verdict = $state<{ text: string; key: string } | null>(null);

  const budgets = [10_000, 100_000, 1_000_000, 10_000_000, 100_000_000, 1_000_000_000];

  const verdictKey = $derived(spec ? JSON.stringify(spec.transitions) + spec.start : '');
  const canProve = $derived(spec !== null && spec.tapes === 1 && pg.input === '');
  const proveHint = $derived(
    !spec
      ? ''
      : spec.tapes > 1
        ? 'The deciders work on machines with one tape.'
        : pg.input !== ''
          ? 'The deciders start on a blank tape; clear the input to use them.'
          : ''
  );

  $effect(() => {
    if (spec && !spec.states.some((s) => s.name === bpState)) bpState = spec.states[0]?.name ?? '';
  });

  function addBreakpoint(event: SubmitEvent) {
    event.preventDefault();
    if (!bpState) return;
    const b: Breakpoint =
      bpSymbol === ''
        ? { kind: 'state', state: bpState }
        : { kind: 'transition', from: bpState, read: [bpSymbol] };
    if (!pg.hasBreakpoint(b)) pg.toggleBreakpoint(b);
  }

  function go(event: SubmitEvent) {
    event.preventDefault();
    const n = Number(target.replace(/[,\s_]/g, ''));
    if (Number.isFinite(n) && n >= 0) void pg.seek(n);
  }

  function describe(v: VerdictDto): string {
    switch (v.kind) {
      case 'halts':
        return `It halts after ${count(v.steps)} steps with ${count(v.ones)} ones.`;
      case 'non-halting':
        return `It never halts. Proved by ${v.label}.`;
      case 'unknown':
        return 'None of the deciders could settle it.';
      case 'unsupported':
        return v.reason;
    }
  }

  async function prove() {
    if (!spec || !canProve) return;
    proving = true;
    verdict = null;
    const key = verdictKey;
    try {
      const response = await batch().request(
        { type: 'decide', spec: $state.snapshot(spec) },
        'verdict'
      );
      verdict = { text: describe(response.verdict), key };
    } catch (error) {
      verdict = { text: error instanceof Error ? error.message : String(error), key };
    } finally {
      proving = false;
    }
  }
</script>

{#if spec}
  <div class="more">
    <section class="block" aria-labelledby={`${id}-bp`}>
      <h3 id={`${id}-bp`}>Breakpoints</h3>
      {#if pg.breakpoints.length}
        <ul class="list">
          {#each pg.breakpoints as b (JSON.stringify(b))}
            <li>
              <span class="mono"
                >{b.kind === 'state'
                  ? `entering ${b.state}`
                  : `before δ(${b.from}, ${b.read.map(glyph).join(', ')})`}</span
              >
              <button type="button" class="link" onclick={() => pg.toggleBreakpoint(b)}
                >Remove</button
              >
            </li>
          {/each}
        </ul>
      {:else}
        <p class="muted">A run stops when it enters a state or is about to apply a transition.</p>
      {/if}
      <form class="inline" onsubmit={addBreakpoint}>
        <label>
          <span>State</span>
          <select bind:value={bpState} class="mono">
            {#each spec.states as s (s.name)}<option value={s.name}>{s.name}</option>{/each}
          </select>
        </label>
        <label>
          <span>Reading</span>
          <select bind:value={bpSymbol} class="mono" disabled={spec.tapes > 1}>
            <option value="">any symbol</option>
            {#if spec.tapes === 1}
              {#each spec.tapeAlphabet as g (g)}<option value={g}>{glyph(g)}</option>{/each}
            {/if}
          </select>
        </label>
        <button class="button" type="submit">Add breakpoint</button>
      </form>
    </section>

    <section class="block" aria-labelledby={`${id}-budget`}>
      <h3 id={`${id}-budget`}>Run budget</h3>
      <label class="inline">
        <span>Pause a run after</span>
        <select
          class="mono"
          value={pg.budget}
          onchange={(e) => (pg.budget = Number(e.currentTarget.value))}
        >
          {#each budgets as b (b)}<option value={b}>{count(b)}</option>{/each}
        </select>
        <span>steps</span>
      </label>
      <form class="inline" onsubmit={go}>
        <label>
          <span>Go to step</span>
          <input class="mono" inputmode="numeric" bind:value={target} placeholder="0" />
        </label>
        <button class="button" type="submit">Go</button>
      </form>
    </section>

    <section class="block" aria-labelledby={`${id}-prove`}>
      <h3 id={`${id}-prove`}>Does it halt?</h3>
      <p class="muted">
        No program decides this for every machine. The deciders behind the busy beaver proof settle
        many small ones.
      </p>
      <button type="button" class="button" onclick={prove} disabled={!canProve || proving}
        >{proving ? 'Trying...' : 'Try to prove that it never halts'}</button
      >
      <p class="result" aria-live="polite">
        {#if verdict && verdict.key === verdictKey}{verdict.text}{:else if proveHint}<span
            class="muted">{proveHint}</span
          >{/if}
      </p>
    </section>
  </div>
{/if}

<style>
  .more {
    display: grid;
    gap: var(--space-3);
    padding-top: var(--space-1);
  }

  h3 {
    font-size: var(--text-sm);
    font-weight: 650;
    margin-bottom: 0.4rem;
  }

  p {
    font-size: var(--text-sm);
    margin-bottom: 0.6rem;
    max-width: var(--measure);
  }

  .list {
    margin: 0 0 0.6rem;
    padding: 0;
    list-style: none;
    font-size: var(--text-sm);
  }

  .list li {
    display: flex;
    gap: 1rem;
    align-items: baseline;
    padding-block: 0.2rem;
  }

  .inline {
    display: flex;
    flex-wrap: wrap;
    align-items: end;
    gap: 0.5rem 0.75rem;
    font-size: var(--text-sm);
  }

  label.inline {
    align-items: center;
    margin-bottom: 0.75rem;
  }

  form.inline label {
    display: grid;
    gap: 0.2rem;
  }

  form.inline label span {
    font-size: var(--text-xs);
    color: var(--muted);
  }

  select,
  input {
    height: 2.25rem;
    padding: 0 0.5rem;
    border: 1px solid var(--rule-strong);
    border-radius: 0;
    background: var(--field);
    color: var(--text);
    font-size: var(--text-sm);
  }

  input {
    width: 8rem;
  }

  select:focus-visible,
  input:focus-visible {
    outline: 2px solid var(--accent);
    outline-offset: 1px;
  }

  .link {
    border: 0;
    background: none;
    padding: 0;
    color: var(--muted);
    font-size: var(--text-xs);
    cursor: pointer;
    text-decoration: underline;
    text-underline-offset: 0.18em;
  }

  .link:hover {
    color: var(--text);
  }

  .result {
    margin-top: 0.6rem;
    min-height: 1.5em;
  }
</style>
