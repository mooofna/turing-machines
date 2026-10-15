<!--
  Fig. 6.2: Antihydra, first checked against its rule on the machine itself, then the rule
  iterated far beyond what the machine reaches.

  One primary action, Run, starts both jobs in the batch worker. (a) The machine runs from the
  blank tape on the chain simulator; every time it reaches a configuration A(a, b), the row is
  compared with the rule: the new a and b and the number of steps since the previous row. The
  check ends where step counts would leave the range in which this page counts exactly. (b) The
  rule itself is iterated with big integers, and the counter a is plotted. The figure publishes
  the number of confirmed rule applications, the number of iterations and the final value of a.
  It reports that the halting condition was not reached within the iterations; it never claims
  that the machine does not halt.
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

  let rows = $state<AntihydraRowDto[]>([]);
  let checkDone = $state(false);
  let walk = $state<WalkSummaryDto | null>(null);
  let walkMs = $state<number | null>(null);
  let walkDone = $state(false);
  let checkMs = $state<number | null>(null);
  let running = $state(false);
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

  function run() {
    if (running) return;
    running = true;
    failed = false;
    rows = [];
    checkDone = false;
    walk = null;
    walkDone = false;
    expanded = false;
    const engine = batch();
    const started = performance.now();

    const check = engine.call({ type: 'antihydra-check', maxRows: 1_000 }, (response) => {
      if (destroyed || response.type !== 'antihydra-rows') return;
      if (response.rows.length > 0) rows = [...rows, ...response.rows];
    });
    const iterate = engine.call(
      { type: 'antihydra-walk', iterations: ITERATIONS, samples: SAMPLES },
      (response) => {
        if (destroyed || response.type !== 'antihydra-walk') return;
        walk = response.summary;
        walkMs = response.elapsedMs;
      }
    );
    calls = [check, iterate];

    const checked = check.done.then(() => {
      if (destroyed) return;
      checkMs = performance.now() - started;
      checkDone = true;
      publish('antihydra-checked', confirmed);
    });
    const walked = iterate.done.then(() => {
      if (destroyed || !walk) return;
      walkDone = true;
      publish('antihydra-iterations', walk.iterations);
      publish('antihydra-balance', walk.balance);
    });
    Promise.all([checked, walked])
      .catch(() => {
        if (!destroyed) failed = true;
      })
      .finally(() => {
        if (!destroyed) running = false;
        calls = [];
      });
  }

  onDestroy(() => {
    destroyed = true;
    for (const call of calls) call.cancel();
  });

  const checkStatus = $derived.by(() => {
    if (rows.length === 0) return running ? 'Simulating the machine from the blank tape.' : '';
    const steps = lastRow ? formatInteger(lastRow.steps) : '';
    if (!checkDone) {
      return `${formatInteger(confirmed)} rule applications confirmed so far, up to step ${steps}.`;
    }
    if (!allAgree)
      return 'The machine and the rule differ; the check stopped at the first difference.';
    return (
      `All ${formatInteger(confirmed)} rule applications agree with the rule, up to step ${steps}, ` +
      `simulated in ${formatDuration(checkMs ?? 0)} in this browser. The check stops there: ` +
      `later step counts would be too large for this page to count exactly.`
    );
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

<div class="antihydra">
  <div class="controls">
    <button class="button primary" type="button" onclick={run} disabled={running}>
      {running ? 'Running' : rows.length > 0 ? 'Run again' : 'Run'}
    </button>
    <p class="intro">
      Simulates the machine <Fact fact={facts['antihydra-machine']} cite={false} /> and compares it with
      the rule, then iterates the rule {formatInteger(ITERATIONS)} times.
    </p>
  </div>

  <section class="part" aria-labelledby="{id}-check">
    <h3 id="{id}-check" class="part-title">The machine against the rule</h3>
    <p class="status" role="status">{checkStatus}</p>
    <div class="scroll" {@attach scrollRegion('Rule applications, scrolls sideways')}>
      <table class="table rows" aria-busy={running && !checkDone}>
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
              <td colspan="6" class="empty">{running ? 'Simulating' : 'Not run yet.'}</td>
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
      <dl class="summary" aria-busy={running && !walkDone}>
        <div>
          <dt>Iterations</dt>
          <dd>
            <Live value={walk?.iterations ?? null} pending={running && !walkDone} placeholder="" />
          </dd>
        </div>
        <div>
          <dt>Rounds with <Fn name="b" /> even</dt>
          <dd>
            <Live value={walk?.evens ?? null} pending={running && !walkDone} placeholder="" />
          </dd>
        </div>
        <div>
          <dt>Rounds with <Fn name="b" /> odd</dt>
          <dd><Live value={walk?.odds ?? null} pending={running && !walkDone} placeholder="" /></dd>
        </div>
        <div>
          <dt>Smallest <Fn name="a" /> after the start</dt>
          <dd>
            <Live value={walk?.minBalance ?? null} pending={running && !walkDone} placeholder="" />
          </dd>
        </div>
        <div>
          <dt><Fn name="a" /> after the last iteration</dt>
          <dd>
            <Live value={walk?.balance ?? null} pending={running && !walkDone} placeholder="" />
          </dd>
        </div>
        <div>
          <dt>Binary digits of <Fn name="b" /> + 4</dt>
          <dd><Live value={walk?.bits ?? null} pending={running && !walkDone} placeholder="" /></dd>
        </div>
        <div>
          <dt>Time in this browser</dt>
          <dd>
            <Live
              value={walkDone ? walkMs : null}
              pending={running && !walkDone}
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

  {#if failed}
    <p class="status">The computation failed in this browser.</p>
  {/if}
</div>

<style>
  .controls {
    display: flex;
    flex-wrap: wrap;
    align-items: center;
    gap: var(--space-2) var(--space-3);
    margin-bottom: var(--space-3);
  }

  .intro {
    flex: 1 1 18rem;
    max-width: var(--measure);
    margin: 0;
    font-size: var(--text-sm);
    color: var(--muted);
  }

  .intro :global(.fact) {
    color: var(--text);
    white-space: normal;
    overflow-wrap: anywhere;
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
