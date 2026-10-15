<!--
  Fig. 6.1: towers of twos, 2↑↑k for k = 1 to 6, computed exactly with `upArrow` in this browser.

  The one interaction is the choice of k. Up to k = 4 the value is shown; for k = 5 the number of
  decimal digits and all of them, in lines of ten-digit groups with the position of the first
  digit of each line; for k = 6 only the size of the number of its digits, derived from the
  leading digits of 2↑↑5. The figure publishes 2↑↑4 and the digit count of 2↑↑5 for the text.
-->
<script lang="ts">
  import { formatInteger } from '#lib/components/Live.svelte';
  import { publish } from '#lib/content/live.svelte.ts';
  import { editor } from '#lib/engine/wasm.ts';
  import type { FigureProps } from '../registry.ts';
  import { inView } from '../ch4/common/inview.ts';
  import { scrollRegion } from '../ch4/common/scroll.ts';
  import Fn from '../ch4/common/Fn.svelte';
  import Segmented from '../ch4/common/Segmented.svelte';
  import {
    arrowsMath,
    digitLines,
    digitsOfPowerOfTwo,
    digitsOfSix,
    scientific,
    towerFormula
  } from './towers.ts';

  let { id }: FigureProps = $props();

  type K = 1 | 2 | 3 | 4 | 5 | 6;

  let k = $state<K>(5);
  /** The decimal digits of 2↑↑1 to 2↑↑5, once computed. */
  let values = $state<Partial<Record<K, string>>>({});
  let failed = $state(false);
  let blockWidth = $state(0);

  async function compute() {
    try {
      const wasm = await editor();
      const next: Partial<Record<K, string>> = {};
      for (const height of [1, 2, 3, 4, 5] as const) {
        const tower = wasm.upArrow(2, height, 100_000);
        if (!tower.exact) throw new Error(`2↑↑${height} has no exact value`);
        next[height] = tower.exact;
      }
      values = next;
      publish('tower-4', Number(next[4]));
      publish('tower-5-digits', next[5]!.length);
    } catch {
      failed = true;
    }
  }

  const five = $derived(values[5]);
  const six = $derived(five ? digitsOfPowerOfTwo(five) : null);

  /** Ten-digit groups per line, as many as fit the block. */
  const groups = $derived.by(() => {
    // IBM Plex Mono advances 0.6 em per digit; the block's type is 13px, its index column 5 em.
    const char = 13 * 0.6;
    const available = blockWidth - 6 * char - 16;
    return Math.max(1, Math.min(5, Math.floor(available / (11 * char))));
  });
  const lines = $derived(five ? digitLines(five, groups) : []);
  const indexWidth = $derived(String(lines.at(-1)?.first ?? 1).length);
</script>

{#snippet legend()}Height <Fn name="k" />{/snippet}

<div class="towers" {@attach inView(() => void compute(), '100% 0px')}>
  <div class="controls">
    <Segmented
      {legend}
      name="{id}-k"
      options={[1, 2, 3, 4, 5, 6].map((n) => ({ value: n as K, label: String(n) }))}
      bind:value={k}
    />
  </div>

  <div class="plate frame" data-status={failed ? 'error' : values[5] ? 'done' : 'computing'}>
    <div class="summary" aria-live="polite">
      <p class="formula">
        {@html towerFormula(k, values[4] ? formatInteger(Number(values[4])) : undefined)}
      </p>

      {#if failed}
        <p class="muted">The value could not be computed in this browser.</p>
      {:else if k <= 4}
        <p class="value mono">
          {#if values[k]}{formatInteger(Number(values[k]))}{:else}<span class="muted"
              >Computing</span
            >{/if}
        </p>
      {:else if k === 5}
        <p class="count">
          {#if five}<span class="mono">{formatInteger(five.length)}</span> decimal digits{:else}<span
              class="muted">Computing</span
            >{/if}
        </p>
      {:else if six}
        <p class="count">
          about {@html scientific(six.mantissa.toFixed(3), formatInteger(six.exponent))} decimal digits
        </p>
        <p class="explain">
          The number of digits of {@html arrowsMath(6)} is {@html digitsOfSix}. Written out, this
          count alone has
          <span class="mono">{formatInteger(six.exponent + 1)}</span> digits, so {@html arrowsMath(
            6
          )} itself cannot be written out.
        </p>
      {:else}
        <p class="count"><span class="muted">Computing</span></p>
      {/if}
    </div>

    {#if k === 5 && five && !failed}
      <div
        class="digits mono"
        bind:clientWidth={blockWidth}
        {@attach scrollRegion(`The ${formatInteger(five.length)} decimal digits of 2↑↑5`)}
      >
        {#each lines as line (line.first)}
          <div class="line">
            <span class="index" style:min-width="{indexWidth}ch" aria-hidden="true"
              >{line.first}</span
            >{#each line.groups as group, g (g)}<span class="group">{group}</span>{/each}
          </div>
        {/each}
      </div>
    {/if}
  </div>
</div>

<style>
  .controls {
    margin-bottom: var(--space-2);
  }

  .plate {
    display: flex;
    flex-direction: column;
    min-height: 21rem;
    padding: var(--space-3);
  }

  .formula {
    margin: 0 0 var(--space-2);
    font-size: var(--text-xl);
    line-height: 1.6;
    overflow-x: auto;
  }

  .summary {
    display: flex;
    flex-direction: column;
    flex: none;
  }

  .value {
    margin: 0;
    font-size: var(--text-xl);
    line-height: 1.2;
  }

  .count {
    margin: 0 0 var(--space-2);
    font-size: var(--text-base);
  }

  .explain {
    max-width: var(--measure);
    margin: 0;
    font-size: var(--text-sm);
    color: var(--muted);
  }

  .digits {
    flex: 1;
    max-height: 13rem;
    overflow: auto;
    border-top: 1px solid var(--rule);
    padding-top: 0.6rem;
    font-size: 0.8125rem;
    line-height: 1.7;
    font-variant-numeric: tabular-nums;
  }

  .line {
    white-space: nowrap;
  }

  .index {
    display: inline-block;
    margin-right: 1.4em;
    text-align: right;
    color: var(--muted);
  }

  .group + .group {
    margin-left: 1ch;
  }

  @media (max-width: 47.99rem) {
    .plate {
      min-height: 18rem;
      padding: var(--space-2);
    }
  }
</style>
