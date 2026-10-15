<!--
  Fig. 5.1: the enumeration in tree normal form for two, three or four states, run in this
  browser with the site's own deciders, next to the counts of the formally verified enumeration
  of Coq-BB5 from the facts ledger.

  One primary action, Start, runs the size chosen in the segmented control. Two and three states
  take milliseconds in one worker; four states visit 858,909 nodes and are split into shards over
  up to four workers. The table is a funnel: all nodes, the halting ones, then one row per decider
  family with the machines it resolved and how many remain, then the records.
-->
<script lang="ts">
  import { onDestroy } from 'svelte';
  import Fact from '#lib/components/Fact.svelte';
  import Live, { formatDuration, formatInteger } from '#lib/components/Live.svelte';
  import { facts } from '#lib/content/facts.ts';
  import { batch } from '#lib/engine/client.ts';
  import type { FigureProps } from '../registry.ts';
  import Segmented from '../ch4/common/Segmented.svelte';
  import { scrollRegion } from '../ch4/common/scroll.ts';
  import Fn from '../ch4/common/Fn.svelte';
  import SourceLink from '../ch4/common/SourceLink.svelte';
  import { enumerate, Pool, type Job, type Outcome, type Progress } from './enumerate.ts';

  let { id }: FigureProps = $props();

  type Size = 2 | 3 | 4;
  type FactId = keyof typeof facts;

  /** The decider families of the pipeline for each size, in pipeline order (docs/deciders.md). */
  const PIPELINE: Record<Size, string[]> = {
    2: ['loops', 'ngram-cps'],
    3: ['loops', 'ngram-cps', 'ngram-cps-history'],
    4: ['loops', 'ngram-cps', 'ngram-cps-history', 'ngram-cps-lru', 'repwl']
  };

  const LABELS: Record<string, string> = {
    loops: 'Cyclers and translated cyclers',
    'ngram-cps': 'n-gram closed position sets',
    'ngram-cps-history': 'n-gram CPS with history',
    'ngram-cps-lru': 'n-gram CPS with LRU',
    repwl: 'Repeated word lists'
  };

  /** The verified counts and values, from the ledger. */
  const VERIFIED: Record<
    Size,
    { nodes: FactId; halting: FactId; nonHalting: FactId; steps: FactId; ones: FactId }
  > = {
    2: {
      nodes: 'bb2-tnf-count',
      halting: 'bb2-tnf-halting',
      nonHalting: 'bb2-tnf-nonhalting',
      steps: 'bb2-steps',
      ones: 'bb2-ones'
    },
    3: {
      nodes: 'bb3-tnf-count',
      halting: 'bb3-tnf-halting',
      nonHalting: 'bb3-tnf-nonhalting',
      steps: 'bb3-steps',
      ones: 'bb3-ones'
    },
    4: {
      nodes: 'bb4-tnf-count',
      halting: 'bb4-tnf-halting',
      nonHalting: 'bb4-tnf-nonhalting',
      steps: 'bb4-steps',
      ones: 'bb4-ones'
    }
  };

  /** Four states are split into this many shards when several workers are available. */
  const SHARDS = 16;

  let size = $state<Size>(3);
  let results = $state<Partial<Record<Size, Outcome>>>({});
  let live = $state<Progress | null>(null);
  let running = $state<Size | null>(null);
  let failed = $state(false);

  let job: Job | null = null;
  const pool = new Pool();
  let destroyed = false;

  const shown = $derived<Progress | null>(
    running === size ? live : (results[size]?.progress ?? null)
  );
  const outcome = $derived(running === size ? null : (results[size] ?? null));
  const verified = $derived(VERIFIED[size]);

  async function start() {
    if (running !== null) return;
    const states = size;
    running = states;
    failed = false;
    live = null;
    const cores = typeof navigator === 'undefined' ? 1 : navigator.hardwareConcurrency || 1;
    const count = states === 4 ? Math.max(1, Math.min(4, cores)) : 1;
    const engines = count > 1 ? pool.workers(count) : [batch()];
    job = enumerate(states, engines, count > 1 ? SHARDS : 1, (progress) => {
      if (!destroyed) live = progress;
    });
    try {
      const result = await job.done;
      if (!destroyed) results[states] = result;
    } catch {
      if (!destroyed) failed = true;
    } finally {
      job = null;
      if (!destroyed) running = null;
    }
  }

  onDestroy(() => {
    destroyed = true;
    job?.cancel();
    pool.terminate();
  });

  const value = (fact: FactId) => Number(facts[fact].value);

  /** The funnel rows of the decider families, with the machines that remain after each. */
  const stages = $derived.by(() => {
    const keys = [...PIPELINE[size]];
    for (const d of shown?.deciders ?? []) {
      if (d.nonHalting > 0 && !keys.includes(d.key)) keys.push(d.key);
    }
    let remaining = shown ? shown.nodes - shown.halting : null;
    return keys.map((key) => {
      const count = shown?.deciders.find((d) => d.key === key)?.nonHalting ?? null;
      if (remaining !== null && count !== null) remaining -= count;
      return { key, label: LABELS[key] ?? key, count, remaining };
    });
  });

  const nonHalting = $derived(shown ? shown.deciders.reduce((n, d) => n + d.nonHalting, 0) : null);

  /** "agrees" once the run is complete and the count equals the verified one. */
  function compare(count: number | null | undefined, fact: FactId): string {
    if (!outcome || count === null || count === undefined) return '';
    return count === value(fact) ? 'agrees' : 'differs';
  }

  const status = $derived.by(() => {
    if (failed) return 'The enumeration failed in this browser.';
    if (running !== null) {
      const n = live?.nodes ?? 0;
      return `Enumerating ${running} states: ${n.toLocaleString('en-US')} machines so far.`;
    }
    if (!outcome) return `Press Start to enumerate all machines with ${size} states.`;
    const holdouts = outcome.progress.holdouts;
    const resolved =
      holdouts === 0
        ? 'every machine is resolved'
        : `${holdouts.toLocaleString('en-US')} machines remain unresolved`;
    const workers = outcome.workers > 1 ? ` on ${outcome.workers} workers` : '';
    return `Finished in ${formatDuration(outcome.elapsedMs)} in this browser${workers}; ${resolved}.`;
  });
</script>

<div class="enumeration">
  <div class="controls">
    <Segmented
      legend="States"
      name="{id}-states"
      options={[
        { value: 2, label: '2' },
        { value: 3, label: '3' },
        { value: 4, label: '4' }
      ]}
      bind:value={size}
      disabled={running !== null}
    />
    <button class="button primary" type="button" onclick={start} disabled={running !== null}>
      {running !== null ? 'Running' : 'Start'}
    </button>
    <p class="status" role="status" aria-live="polite">{status}</p>
  </div>

  <div class="scroll" {@attach scrollRegion('Enumeration results, scrolls sideways')}>
    <table class="table funnel" aria-busy={running !== null}>
      <caption class="visually-hidden">
        Enumeration of the machines with {size} states in tree normal form: counts in this browser and
        in the formally verified enumeration of Coq-BB5
      </caption>
      <thead>
        <tr>
          <th scope="col">Machines with {size} states</th>
          <th scope="col" class="num">This browser</th>
          <th scope="col" class="num rem">Remaining</th>
          <th scope="col" class="num">
            <SourceLink id="coqbb5" label="Coq-BB5" /><span class="sub">formally verified</span>
          </th>
          <th scope="col" class="wide-only"><span class="visually-hidden">Comparison</span></th>
        </tr>
      </thead>
      <tbody>
        <tr class="total">
          <th scope="row">Nodes in tree normal form</th>
          <td class="num"
            ><Live value={shown?.nodes ?? null} pending={running === size} placeholder="" /></td
          >
          <td class="num rem"></td>
          <td class="num"
            ><Fact fact={facts[verified.nodes]} cite={false} /><span class="agree-inline"
              >{compare(shown?.nodes, verified.nodes)}</span
            ></td
          >
          <td class="agree wide-only">{compare(shown?.nodes, verified.nodes)}</td>
        </tr>
        <tr>
          <th scope="row"
            >Halting{#if shown}<span class="remain">
                {formatInteger(shown.nodes - shown.halting)} remain</span
              >{/if}</th
          >
          <td class="num"
            ><Live value={shown?.halting ?? null} pending={running === size} placeholder="" /></td
          >
          <td class="num rem">
            <Live
              value={shown ? shown.nodes - shown.halting : null}
              pending={running === size}
              placeholder=""
            />
          </td>
          <td class="num"
            ><Fact fact={facts[verified.halting]} cite={false} /><span class="agree-inline"
              >{compare(shown?.halting, verified.halting)}</span
            ></td
          >
          <td class="agree wide-only">{compare(shown?.halting, verified.halting)}</td>
        </tr>
        {#each stages as stage, i (stage.key)}
          <tr class="stage" class:last={i === stages.length - 1}>
            <th scope="row"
              ><span class="indent"
                >{stage.label}{#if stage.remaining !== null}<span class="remain">
                    {formatInteger(stage.remaining)} remain</span
                  >{/if}</span
              ></th
            >
            <td class="num"
              ><Live value={stage.count} pending={running === size} placeholder="" /></td
            >
            <td class="num rem"
              ><Live value={stage.remaining} pending={running === size} placeholder="" /></td
            >
            <td class="num"></td>
            <td class="agree wide-only"></td>
          </tr>
        {/each}
        <tr class="sum">
          <th scope="row">Never halt, all deciders</th>
          <td class="num"><Live value={nonHalting} pending={running === size} placeholder="" /></td>
          <td class="num rem"></td>
          <td class="num"
            ><Fact fact={facts[verified.nonHalting]} cite={false} /><span class="agree-inline"
              >{compare(nonHalting, verified.nonHalting)}</span
            ></td
          >
          <td class="agree wide-only">{compare(nonHalting, verified.nonHalting)}</td>
        </tr>
        <tr>
          <th scope="row">Holdouts</th>
          <td class="num"
            ><Live value={shown?.holdouts ?? null} pending={running === size} placeholder="" /></td
          >
          <td class="num rem"></td>
          <td class="num"></td>
          <td class="agree wide-only"></td>
        </tr>
        <tr class="record">
          <th scope="row">
            <Fn name="S" arg={size} />, the most steps
            {#if shown?.maxSteps}<span class="machine mono wide-only">{shown.maxSteps.machine}</span
              >{/if}
          </th>
          <td class="num"
            ><Live
              value={shown?.maxSteps?.value ?? null}
              pending={running === size}
              placeholder=""
            /></td
          >
          <td class="num rem"></td>
          <td class="num"
            ><Fact fact={facts[verified.steps]} cite={false} /><span class="agree-inline"
              >{compare(shown?.maxSteps?.value, verified.steps)}</span
            ></td
          >
          <td class="agree wide-only">{compare(shown?.maxSteps?.value, verified.steps)}</td>
        </tr>
        {#if shown?.maxSteps}
          <tr class="machine-row">
            <td colspan="5" class="mono">{shown.maxSteps.machine}</td>
          </tr>
        {/if}
        <tr class="record">
          <th scope="row">
            <Fn name="Σ" arg={size} />, the most ones
            {#if shown?.maxOnes}<span class="machine mono wide-only">{shown.maxOnes.machine}</span
              >{/if}
          </th>
          <td class="num"
            ><Live
              value={shown?.maxOnes?.value ?? null}
              pending={running === size}
              placeholder=""
            /></td
          >
          <td class="num rem"></td>
          <td class="num"></td>
          <td class="agree wide-only"></td>
        </tr>
        {#if shown?.maxOnes}
          <tr class="machine-row">
            <td colspan="5" class="mono">{shown.maxOnes.machine}</td>
          </tr>
        {/if}
      </tbody>
    </table>
  </div>

  <p class="note">
    The deciders of this site are not formally verified. The column Coq-BB5 gives the counts of the
    formally verified enumeration and the value of <Fn name="S" arg={size} /> it proves, for comparison.
  </p>
</div>

<style>
  .controls {
    display: flex;
    flex-wrap: wrap;
    align-items: center;
    gap: var(--space-2) var(--space-3);
    margin-bottom: var(--space-3);
  }

  .status {
    flex: 1 1 16rem;
    margin: 0;
    font-size: var(--text-sm);
    color: var(--muted);
  }

  .scroll {
    overflow-x: auto;
  }

  .scroll:focus-visible {
    outline-offset: 2px;
  }

  .funnel {
    min-width: 36rem;
  }

  .funnel thead th {
    vertical-align: bottom;
  }

  .funnel thead th.num {
    font-family: inherit;
    font-size: inherit;
  }

  .sub {
    display: block;
    font-size: var(--text-xs);
  }

  .funnel tbody th {
    font-weight: 400;
  }

  .total th,
  .sum th,
  .record th {
    font-weight: 600;
  }

  .indent {
    display: block;
    padding-left: 1.25rem;
    color: var(--muted);
  }

  .stage td,
  .stage th {
    border-bottom-color: transparent;
    padding-block: 0.3rem;
  }

  .stage.last td,
  .stage.last th {
    border-bottom-color: var(--rule);
  }

  .sum th,
  .sum td {
    border-top: 1px solid var(--rule);
  }

  .machine {
    display: block;
    overflow-wrap: anywhere;
    margin-top: 0.2rem;
    font-weight: 400;
    color: var(--muted);
  }

  .agree {
    width: 4.5rem;
    font-size: var(--text-xs);
    color: var(--muted);
    white-space: nowrap;
  }

  .remain,
  .agree-inline,
  .machine-row {
    display: none;
  }

  @media (max-width: 47.99rem) {
    .funnel {
      min-width: 0;
    }

    .funnel thead th.num {
      white-space: normal;
    }

    .indent {
      padding-left: 0.75rem;
    }

    .rem {
      display: none;
    }

    .remain {
      display: block;
      font-size: var(--text-xs);
      font-weight: 400;
      color: var(--muted);
      font-variant-numeric: tabular-nums;
    }

    .wide-only {
      display: none;
    }

    .agree-inline {
      display: block;
      font-family: var(--font-sans);
      font-size: var(--text-xs);
      color: var(--muted);
    }

    .machine-row {
      display: table-row;
    }

    .record:has(+ .machine-row) th,
    .record:has(+ .machine-row) td {
      border-bottom-color: transparent;
      padding-bottom: 0.2rem;
    }

    .machine-row td {
      padding-top: 0;
      font-size: var(--text-xs);
      color: var(--muted);
      overflow-wrap: anywhere;
    }
  }

  .note {
    max-width: var(--measure);
    margin: var(--space-2) 0 0;
    font-size: var(--text-sm);
    color: var(--muted);
  }
</style>
