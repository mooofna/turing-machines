<!--
  Fig. 2.2: the number of machines with n states and two symbols, written out in full.

  The reader chooses n from 1 to 6 (the one interaction). The figure shows how Radó's formula
  arises (2n entries, each with 4(n + 1) options), the power, and its value computed exactly by
  the engine (machineCount), with the number of digits. A second line gives the count in which
  all transitions into the halting state are one option, (4n + 1)^2n, as the chapter text does;
  that number is computed with exact integers in the browser.
-->
<script lang="ts">
  import { editor } from '#lib/engine/wasm.ts';
  import type { FigureProps } from '../types.ts';
  import { errorText, inView } from '../ch1/kit.ts';
  import Plate from '../ch1/Plate.svelte';

  let { id }: FigureProps = $props();
  const uid = $props.id();

  const CHOICES = [1, 2, 3, 4, 5, 6];

  let n = $state(5);
  let counts = $state.raw<Record<number, string> | null>(null);
  let failure = $state<string | null>(null);
  const visible = inView(start);

  async function start() {
    try {
      const w = await editor();
      const out: Record<number, string> = {};
      for (const k of CHOICES) {
        const value = w.machineCount(k, 2);
        if (value === undefined) throw new Error(`no count for ${k} states`);
        out[k] = value;
      }
      counts = out;
    } catch (error) {
      failure = errorText(error);
    }
  }

  const group = (digits: string) => digits.replace(/\B(?=(\d{3})+(?!\d))/g, ',');

  const base = $derived(4 * (n + 1));
  const exponent = $derived(2 * n);
  const value = $derived(counts?.[n] ?? null);
  const alternative = $derived((BigInt(4 * n + 1) ** BigInt(2 * n)).toString());
</script>

<Plate {id} busy={!counts && !failure} attach={visible}>
  <fieldset class="choose">
    <legend class="fig-label">Number of states <i>n</i></legend>
    <div class="segments">
      {#each CHOICES as k (k)}
        <label class="segment" class:on={n === k}>
          <input class="visually-hidden" type="radio" name="{uid}-n" value={k} bind:group={n} />
          <span class="mono">{k}</span>
        </label>
      {/each}
    </div>
  </fieldset>

  <div class="result" aria-live="polite">
    <p class="why">
      {exponent} entries in the table, each with
      <span class="mono">4(n + 1) = {base}</span> options:
    </p>
    <p class="power mono">
      <span class="formula">{base}<sup>{exponent}</sup> =</span>
    </p>
    <p class="number mono" class:pending={value === null}>
      {value === null ? (failure ? '' : 'computing') : group(value)}
    </p>
    <p class="digits">
      {#if value !== null}
        A number with {value.length} digits.
      {/if}
      {#if failure}The engine could not compute the count: {failure}{/if}
    </p>
  </div>

  <p class="other">
    Counting all transitions into the halting state as one option gives
    <span class="mono part">(4n + 1)<sup>2n</sup></span>
    <span class="mono part">= {4 * n + 1}<sup>{exponent}</sup></span>
    <span class="mono part">= {group(alternative)}</span> machines.
  </p>
</Plate>

<style>
  .choose {
    margin: 0;
    padding: 0;
    border: 0;
    min-width: 0;
  }

  .choose legend {
    padding: 0;
  }

  .segments {
    display: flex;
  }

  .segment {
    display: grid;
    place-items: center;
    width: 2.75rem;
    height: 2.5rem;
    margin-left: -1px;
    border: 1px solid var(--rule-strong);
    background: var(--bg);
    cursor: pointer;
    font-size: var(--text-sm);
  }

  .segment:first-child {
    margin-left: 0;
  }

  .segment:hover {
    color: var(--accent);
  }

  .segment.on {
    position: relative;
    background: var(--accent);
    border-color: var(--accent);
    color: var(--on-accent);
  }

  .segment:has(:focus-visible) {
    position: relative;
    outline: 2px solid var(--accent);
    outline-offset: 2px;
  }

  .segment .mono {
    font-size: var(--text-sm);
  }

  .result {
    margin-top: var(--space-3);
  }

  .why {
    margin: 0 0 0.5rem;
    color: var(--muted);
  }

  .why .mono {
    color: var(--text);
    white-space: nowrap;
  }

  .power {
    margin: 0;
    font-size: var(--text-base);
  }

  .power sup {
    font-size: 0.68em;
    line-height: 0;
  }

  .number {
    margin: 0.15rem 0 0;
    font-size: var(--text-xl);
    line-height: 1.2;
    letter-spacing: -0.01em;
    overflow-wrap: anywhere;
  }

  .number.pending {
    color: var(--muted);
  }

  .digits {
    min-height: 1.45em;
    margin: 0.4rem 0 0;
    color: var(--muted);
  }

  .other {
    margin: var(--space-3) 0 0;
    padding-top: var(--space-2);
    border-top: 1px solid var(--rule);
    color: var(--muted);
    max-width: 40rem;
  }

  .other .mono {
    color: var(--text);
  }

  .other .part {
    white-space: nowrap;
  }

  .other sup {
    font-size: 0.7em;
    line-height: 0;
  }
</style>
