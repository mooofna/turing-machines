<!--
  Fig. 6.2: Antihydra, first checked against its rule on the machine itself, then the rule
  iterated far beyond what the machine reaches.

  Both parts start on their own when the figure comes within a viewport of the screen, in the
  batch worker, so that the text around the figure never waits for a reader's action. (a) The
  machine runs from the blank tape on the chain simulator; every time it reaches a configuration
  A(a, b), the row is compared with the rule: the new a and b and the number of steps since the
  previous row. This first check stops after a time budget or a number of rows that a phone
  also reaches within it. The one primary action continues the check to the end of the range in
  which this page counts steps exactly, 2^53; the worker's job cannot be resumed, so it simulates
  the machine again from the blank tape and the button's description says so. (b) The rule
  itself is iterated 100,000 times with big integers and the counter a is plotted. The figure
  publishes the confirmed rule applications, the iterations and the last value of a as they
  arrive. It reports that the halting condition was not reached within the iterations; it never
  claims that the machine does not halt.
-->
<script lang="ts">
  import { onDestroy } from 'svelte';
  import Fact from '#lib/components/Fact.svelte';
  import Live, { formatDuration, formatInteger } from '#lib/components/Live.svelte';
  import { facts } from '#lib/content/facts.ts';
  import { publish } from '#lib/content/live.svelte.ts';
  import { batch, type Call } from '#lib/engine/client.ts';
  import type { AntihydraRowDto, WalkSummaryDto } from '#lib/engine/pkg/turing_wasm.js';
  import type { FigureProps } from '../registry.ts';
  import { inView } from '../ch4/common/inview.ts';
  import { scrollRegion } from '../ch4/common/scroll.ts';
  import Fn from '../ch4/common/Fn.svelte';
  import WalkChart from './WalkChart.svelte';

  let { id }: FigureProps = $props();

  /** Iterations of the rule. */
  const ITERATIONS = 100_000;
  /** Samples of the walk for the chart. */
  const SAMPLES = 800;
  /** Rows of the check shown before the disclosure. */
  const VISIBLE = 12;
  /**
   * The automatic check stops after this many rows (the start and 31 rule applications, about
   * 0.3 s on a laptop and well under the budget on a phone) or after the budget, if earlier.
   */
  const AUTO_ROWS = 32;
  const AUTO_BUDGET_MS = 1_500;
  /** More rows than the check produces before the step count reaches 2^53. */
  const ALL_ROWS = 1_000;

  /**
   * `idle` before the figure is seen; `auto` while the first part runs; `partial` after it;
   * `full` while the check runs to the end; `complete` after that.
   */
  type Phase = 'idle' | 'auto' | 'partial' | 'full' | 'complete';

  let phase = $state<Phase>('idle');
  let rows = $state<AntihydraRowDto[]>([]);
  let walk = $state<WalkSummaryDto | null>(null);
  let walkMs = $state<number | null>(null);
  let walkDone = $state(false);
  let checkMs = $state<number | null>(null);
  let failed = $state(false);
  let expanded = $state(false);

  let calls: Call[] = [];
  let destroyed = false;

  const confirmed = $derived.by(() => {
    // Rule applications are the rows after the start that agree, as long as all before agree.
    let n = 0;
    for (const row of rows) {
      if (!row.agrees) break;
      if (row.iteration > 0) n += 1;
    }
    return n;
  });
  const allAgree = $derived(rows.length > 0 && rows.every((r) => r.agrees));
  const lastRow = $derived(rows.at(-1) ?? null);
  const shownRows = $derived(expanded ? rows : rows.slice(0, VISIBLE));
  const checking = $derived(phase === 'auto' || phase === 'full');

  /**
   * Runs the machine check from the blank tape for at most `maxRows` rows and `budgetMs`
   * milliseconds. Rows the table already has are not added again, so a second run continues
   * the table where the first stopped.
   */
  function check(maxRows: number, budgetMs: number | null, next: Phase) {
    const started = performance.now();
    const call = batch().call({ type: 'antihydra-check', maxRows }, (response) => {
      if (destroyed || response.type !== 'antihydra-rows') return;
      const fresh = response.rows.filter((r) => r.iteration >= rows.length);
      if (fresh.length === 0) return;
      rows = [...rows, ...fresh];
      publish('antihydra-checked', confirmed);
    });
    calls.push(call);
    const timer = budgetMs === null ? null : setTimeout(() => call.cancel(), budgetMs);
    call.done
      .then(() => {
        if (destroyed) return;
        checkMs = performance.now() - started;
        publish('antihydra-checked', confirmed);
        phase = next;
      })
      .catch(() => {
        if (!destroyed) failed = true;
      })
      .finally(() => {
        if (timer !== null) clearTimeout(timer);
      });
  }

  /** Starts both parts once, when the figure comes near the screen. */
  function start() {
    if (phase !== 'idle') return;
    phase = 'auto';
    const iterate = batch().call(
      { type: 'antihydra-walk', iterations: ITERATIONS, samples: SAMPLES },
      (response) => {
        if (destroyed || response.type !== 'antihydra-walk') return;
        walk = response.summary;
        walkMs = response.elapsedMs;
        publish('antihydra-iterations', walk.iterations);
        publish('antihydra-balance', walk.balance);
      }
    );
    calls.push(iterate);
    iterate.done
      .then(() => {
        if (!destroyed && walk) walkDone = true;
      })
      .catch(() => {
        if (!destroyed) failed = true;
      });
    check(AUTO_ROWS, AUTO_BUDGET_MS, 'partial');
  }

  /** Runs the check again from the blank tape, to the end of the exact range. */
  function continueCheck() {
    if (phase !== 'partial') return;
    phase = 'full';
    check(ALL_ROWS, null, 'complete');
  }

  onDestroy(() => {
    destroyed = true;
    for (const call of calls) call.cancel();
  });

  const walkStatus = $derived.by(() => {
    if (!walk) return '';
    if (walk.halted) {
      return `The counter fell below zero after ${formatInteger(walk.iterations)} iterations: the machine halts there.`;
    }
    if (!walkDone)
      return `${formatInteger(walk.iterations)} of ${formatInteger(ITERATIONS)} iterations.`;
    return (
      `The counter did not reach the halting condition within ${formatInteger(walk.iterations)} ` +
      `iterations. This says nothing about later iterations.`
    );
  });

  /** About eleven sampled values for the table view of the chart. */
  const table = $derived.by(() => {
    const s = walk?.samples ?? [];
    if (s.length === 0) return [];
    const picks = new Set<number>();
    for (let i = 0; i <= 10; i++) picks.add(Math.round((i * (s.length - 1)) / 10));
    return [...picks].map((i) => s[i]!);
  });
</script>

{#snippet power()}2<span class="visually-hidden"> to the power of </span><sup>53</sup>{/snippet}

<div class="antihydra" {@attach inView(start, '100% 0px')}>
  <p class="intro">
    Simulates the machine <Fact fact={facts['antihydra-machine']} cite={false} /> and compares it with
    the rule, then iterates the rule {formatInteger(ITERATIONS)} times.
  </p>

  <section class="part" aria-labelledby="{id}-check">
    <h3 id="{id}-check" class="part-title">The machine against the rule</h3>
    <p class="status" role="status">
      {#if failed}
        The computation failed in this browser.
      {:else if rows.length === 0}
        {checking ? 'Simulating the machine from the blank tape.' : ''}
      {:else if checking}
        {formatInteger(confirmed)} rule applications confirmed so far, up to step
        {formatInteger(lastRow?.steps ?? 0)}.
      {:else if !allAgree}
        The machine and the rule differ; the check stopped at the first difference.
      {:else if phase === 'partial'}
        The first {formatInteger(confirmed)} rule applications agree with the rule, up to step
        {formatInteger(lastRow?.steps ?? 0)}, simulated in {formatDuration(checkMs ?? 0)} in this browser.
      {:else}
        All {formatInteger(confirmed)} rule applications agree with the rule, up to step
        {formatInteger(lastRow?.steps ?? 0)}, simulated in {formatDuration(checkMs ?? 0)} in this browser.
        The check stops there: later step counts would exceed {@render power()}, beyond which this
        page cannot count steps exactly.
      {/if}
    </p>
    <div class="scroll" {@attach scrollRegion('Rule applications, scrolls sideways')}>
      <table class="table rows" aria-busy={checking}>
        <caption class="visually-hidden">
          Configurations A(a, b) reached by the machine, with the steps between them and the steps
          the rule predicts
        </caption>
        <thead>
          <tr>
            <th scope="col" class="num">Round</th>
            <th scope="col" class="num"><Fn name="a" /></th>
            <th scope="col" class="num"><Fn name="b" /></th>
            <th scope="col" class="num">Steps in the round</th>
            <th scope="col" class="num wide-only">Rule predicts</th>
            <th scope="col" class="wide-only"><span class="visually-hidden">Comparison</span></th>
          </tr>
        </thead>
        <tbody id="{id}-rows">
          {#each shownRows as row (row.iteration)}
            <tr>
              <td class="num">{row.iteration === 0 ? 'start' : formatInteger(row.iteration)}</td>
              <td class="num">{formatInteger(row.a)}</td>
              <td class="num">{formatInteger(row.b)}</td>
              <td class="num"
                >{formatInteger(row.delta)}<span class="agree-inline" class:differs={!row.agrees}
                  >{row.agrees
                    ? 'agrees with the rule'
                    : `rule predicts ${formatInteger(row.predicted)}`}</span
                ></td
              >
              <td class="num wide-only">{formatInteger(row.predicted)}</td>
              <td class="agree wide-only" class:differs={!row.agrees}
                >{row.agrees ? 'agrees' : 'differs'}</td
              >
            </tr>
          {:else}
            <tr>
              <td colspan="6" class="empty">{checking ? 'Simulating' : 'Not run yet.'}</td>
            </tr>
          {/each}
        </tbody>
      </table>
    </div>
    {#if rows.length > VISIBLE}
      <button
        class="more"
        type="button"
        aria-expanded={expanded}
        aria-controls="{id}-rows"
        onclick={() => (expanded = !expanded)}
      >
        {expanded
          ? `Show the first ${VISIBLE} rows`
          : `Show all ${formatInteger(rows.length)} rows`}
      </button>
    {/if}
    {#if phase === 'partial' || phase === 'full'}
      <div class="continue">
        <button
          class="button primary"
          type="button"
          onclick={continueCheck}
          disabled={phase === 'full'}
          aria-describedby="{id}-continue"
        >
          <span
            >{#if phase === 'full'}Checking to {@render power()} steps{:else}Continue the check to
              {@render power()} steps{/if}</span
          >
        </button>
        <p id="{id}-continue" class="hint">
          Simulates the machine again from the blank tape, since the check cannot resume where it
          stopped, and compares every rule application until the step count would exceed
          {@render power()}. This takes much longer than the first part.
        </p>
      </div>
    {/if}
  </section>

  <section class="part" aria-labelledby="{id}-walk">
    <h3 id="{id}-walk" class="part-title">The rule, iterated</h3>
    <p class="status" role="status">{walkStatus}</p>
    <WalkChart
      samples={walk?.samples ?? []}
      total={ITERATIONS}
      label="Line chart of the counter a over {formatInteger(
        ITERATIONS
      )} iterations of the rule, starting at a = 0."
    />
    {#if walk}
      <dl class="summary" aria-busy={!walkDone}>
        <div>
          <dt>Iterations</dt>
          <dd>
            <Live value={walk?.iterations ?? null} pending={!walkDone} placeholder="" />
          </dd>
        </div>
        <div>
          <dt>Rounds with <Fn name="b" /> even</dt>
          <dd>
            <Live value={walk?.evens ?? null} pending={!walkDone} placeholder="" />
          </dd>
        </div>
        <div>
          <dt>Rounds with <Fn name="b" /> odd</dt>
          <dd><Live value={walk?.odds ?? null} pending={!walkDone} placeholder="" /></dd>
        </div>
        <div>
          <dt>Smallest <Fn name="a" /> after the start</dt>
          <dd>
            <Live value={walk?.minBalance ?? null} pending={!walkDone} placeholder="" />
          </dd>
        </div>
        <div>
          <dt><Fn name="a" /> after the last iteration</dt>
          <dd>
            <Live value={walk?.balance ?? null} pending={!walkDone} placeholder="" />
          </dd>
        </div>
        <div>
          <dt>Binary digits of <Fn name="b" /> + 4</dt>
          <dd><Live value={walk?.bits ?? null} pending={!walkDone} placeholder="" /></dd>
        </div>
        <div>
          <dt>Time in this browser</dt>
          <dd>
            <Live
              value={walkDone ? walkMs : null}
              pending={!walkDone}
              format={formatDuration}
              placeholder=""
            />
          </dd>
        </div>
      </dl>
    {/if}
    {#if table.length > 0}
      <details class="samples-block">
        <summary><span>Sampled values of the counter <Fn name="a" /></span></summary>
        <table class="table samples">
          <caption class="visually-hidden">Sampled values of the counter a</caption>
          <thead>
            <tr
              ><th scope="col" class="num">Iteration</th><th scope="col" class="num"
                ><Fn name="a" /></th
              ></tr
            >
          </thead>
          <tbody>
            {#each table as s (s.iteration)}
              <tr
                ><td class="num">{formatInteger(s.iteration)}</td><td class="num"
                  >{formatInteger(s.balance)}</td
                ></tr
              >
            {/each}
          </tbody>
        </table>
      </details>
    {/if}
  </section>
</div>

<style>
  .intro {
    max-width: var(--measure);
    margin: 0 0 var(--space-3);
    font-size: var(--text-sm);
    color: var(--muted);
  }

  .intro :global(.fact) {
    color: var(--text);
    white-space: normal;
    overflow-wrap: anywhere;
  }

  .continue {
    display: flex;
    flex-wrap: wrap;
    align-items: center;
    gap: var(--space-1) var(--space-3);
    margin-top: var(--space-3);
  }

  .continue .hint {
    flex: 1 1 18rem;
    max-width: var(--measure);
    margin: 0;
    font-size: var(--text-sm);
    color: var(--muted);
  }

  .button :global(sup),
  .status :global(sup),
  .hint :global(sup) {
    font-size: 0.72em;
    line-height: 0;
  }

  .part {
    padding-top: var(--space-2);
    border-top: 1px solid var(--rule-strong);
  }

  .part + .part {
    margin-top: var(--space-4);
  }

  .part-title {
    margin: 0 0 0.3rem;
    font-size: var(--text-sm);
    font-weight: 650;
    letter-spacing: 0;
  }

  .status {
    min-height: 1.45em;
    max-width: var(--measure);
    margin: 0 0 var(--space-2);
    font-size: var(--text-sm);
    color: var(--muted);
  }

  .scroll {
    overflow-x: auto;
  }

  .rows {
    min-width: 32rem;
  }

  .rows thead th.num,
  .samples thead th.num {
    font-family: inherit;
    font-size: inherit;
  }

  .rows td {
    padding-block: 0.35rem;
  }

  .agree {
    width: 4rem;
    font-size: var(--text-xs);
    color: var(--muted);
  }

  .agree.differs {
    color: var(--text);
    font-weight: 650;
  }

  .empty {
    color: var(--muted);
  }

  .agree-inline {
    display: none;
  }

  @media (max-width: 47.99rem) {
    .rows {
      min-width: 0;
      font-size: var(--text-xs);
    }

    .rows :global(th),
    .rows :global(td) {
      padding-right: 0.6rem;
    }

    .wide-only {
      display: none;
    }

    .agree-inline {
      display: block;
      font-family: var(--font-sans);
      color: var(--muted);
    }

    .agree-inline.differs {
      color: var(--text);
      font-weight: 650;
    }
  }

  .more {
    margin-top: var(--space-1);
    padding: 0.4rem 0;
    border: 0;
    background: none;
    font-size: var(--text-sm);
    color: var(--text);
    text-decoration: underline;
    text-decoration-thickness: 1px;
    text-underline-offset: 0.18em;
    text-decoration-color: color-mix(in srgb, currentColor 35%, transparent);
    cursor: pointer;
  }

  .more:hover {
    color: var(--accent);
    text-decoration-color: var(--accent);
  }

  .summary {
    display: grid;
    grid-template-columns: repeat(4, minmax(0, 1fr));
    gap: var(--space-2) var(--gutter);
    margin: var(--space-3) 0 0;
    font-size: var(--text-sm);
    line-height: 1.4;
  }

  .summary div {
    padding-top: 0.5rem;
    border-top: 1px solid var(--rule);
  }

  dt {
    color: var(--muted);
    margin-bottom: 0.2rem;
  }

  dd {
    margin: 0;
    font-size: var(--text-base);
  }

  .samples-block {
    max-width: var(--measure);
  }

  .samples {
    max-width: 20rem;
  }

  @media (max-width: 47.99rem) {
    .summary {
      grid-template-columns: repeat(2, minmax(0, 1fr));
    }
  }
</style>
