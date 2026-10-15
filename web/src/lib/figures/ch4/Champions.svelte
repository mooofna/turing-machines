<!--
  Fig. 4.1: the busy beaver champions for one to five states (and the machine that leaves Σ(3)
  ones), each run to its halt in the engine worker and drawn as a space-time diagram.

  The one interaction is the choice of champion. The run is measured on its own, then drawn:
  exactly, one row per step, for the short runs, and as an overview that samples the run for
  the five-state champion. The machine strings come from the facts ledger; steps, ones and the
  time are what this browser measured.
-->
<script lang="ts">
  import { onDestroy } from 'svelte';
  import Fact from '#lib/components/Fact.svelte';
  import Live, { formatDuration, formatInteger } from '#lib/components/Live.svelte';
  import { facts } from '#lib/content/facts.ts';
  import { engine } from '#lib/engine/client.ts';
  import type { MachineSpecDto } from '#lib/engine/pkg/turing_wasm.js';
  import { editor } from '#lib/engine/wasm.ts';
  import type { FigureProps } from '../registry.ts';
  import { inView } from './common/inview.ts';
  import Plate, { type PlateLayout } from './common/Plate.svelte';
  import Segmented from './common/Segmented.svelte';

  let { id }: FigureProps = $props();

  type Key = 's1' | 's2' | 's3' | 'sigma3' | 's4' | 's5';

  interface Champion {
    key: Key;
    label: string;
    name: string;
    fact: (typeof facts)[keyof typeof facts];
    /** What the machine attains, for the description of the diagram. */
    attains: string;
  }

  const CHAMPIONS: readonly Champion[] = [
    {
      key: 's1',
      label: 'S(1)',
      name: 'S(1), one state',
      fact: facts['champion-bb1'],
      attains: 'S(1)'
    },
    {
      key: 's2',
      label: 'S(2)',
      name: 'S(2), two states',
      fact: facts['champion-bb2'],
      attains: 'S(2)'
    },
    {
      key: 's3',
      label: 'S(3)',
      name: 'S(3), three states',
      fact: facts['champion-bb3'],
      attains: 'S(3)'
    },
    {
      key: 'sigma3',
      label: 'Σ(3)',
      name: 'Σ(3), three states, most ones',
      fact: facts['champion-sigma3'],
      attains: 'Σ(3)'
    },
    {
      key: 's4',
      label: 'S(4)',
      name: 'S(4), four states',
      fact: facts['champion-bb4'],
      attains: 'S(4)'
    },
    {
      key: 's5',
      label: 'S(5)',
      name: 'S(5), five states',
      fact: facts['champion-bb5'],
      attains: 'S(5)'
    }
  ];

  /** Runs longer than this are drawn as an overview. */
  const EXACT_LIMIT = 2_000;
  /** More steps than any champion needs; a run stops when the machine halts. */
  const BUDGET = 100_000_000;

  interface Run {
    spec: MachineSpecDto;
    steps: number;
    ones: number;
    elapsedMs: number;
    /** Leftmost and rightmost head position during the run. */
    extent: [number, number];
  }

  let choice = $state<Key>('s4');
  let seen = $state(false);
  let runs = $state<Partial<Record<Key, Run>>>({});
  let failed = $state<Partial<Record<Key, boolean>>>({});
  let layout = $state<PlateLayout | null>(null);

  const champion = $derived(CHAMPIONS.find((c) => c.key === choice) ?? CHAMPIONS[4]!);
  const run = $derived(runs[choice]);
  const pending = $derived(seen && !run && !failed[choice]);

  let destroyed = false;
  const started = new Set<Key>();

  /** Runs the champion `key` to its halt in its own session, once. */
  async function measure(key: Key): Promise<void> {
    if (started.has(key)) return;
    started.add(key);
    const session = `${id}-${key}`;
    const worker = engine();
    try {
      const text = String((CHAMPIONS.find((c) => c.key === key) ?? CHAMPIONS[0]!).fact.value);
      const parsed = (await editor()).fromStandard(text).spec;
      if (!parsed) throw new Error(`cannot parse ${text}`);
      if (destroyed) return;
      await worker.request(
        { type: 'open', session, spec: parsed, input: '', radius: 0 },
        'snapshot'
      );
      const final = await worker.request(
        {
          type: 'run',
          session,
          maxSteps: BUDGET,
          breakpoints: { transitions: [], states: [] },
          radius: 0
        },
        'stopped'
      );
      worker.call({ type: 'close', session });
      if (destroyed) return;
      const snapshot = final.result.snapshot;
      runs[key] = {
        spec: parsed,
        steps: snapshot.step,
        ones: snapshot.ones,
        elapsedMs: final.elapsedMs,
        extent: snapshot.extent
      };
    } catch {
      if (!destroyed) failed[key] = true;
    }
  }

  $effect(() => {
    if (seen) void measure(choice);
  });

  onDestroy(() => {
    destroyed = true;
  });

  const description = $derived(
    run
      ? `Space-time diagram of ${champion.fact.display}, the machine that attains ${champion.attains}, ` +
          `from the blank tape until it halts after ${formatInteger(run.steps)} steps with ` +
          `${formatInteger(run.ones)} ones on the tape. Time runs downwards.`
      : 'Space-time diagram, not computed yet.'
  );

  const reading = $derived.by(() => {
    if (!run || !layout) return '';
    if (layout.kind === 'exact') {
      return (
        `Each row is the tape after one more step, from the blank tape at step 0 to the ` +
        `halting configuration at step ${formatInteger(run.steps)}; filled cells hold a 1 and ` +
        `the blue cell is the head.`
      );
    }
    const per = layout.cellsPerColumn;
    const cells = per < 1.5 ? 'one cell' : `about ${formatInteger(Math.round(per))} cells`;
    return (
      `Each of the ${formatInteger(layout.rows)} rows shows the tape after a multiple of ` +
      `${formatInteger(layout.stride)} steps, the last after step ${formatInteger(run.steps)}. ` +
      `Each column covers ${cells}, darker where more of them hold a 1, and blue marks the ` +
      `cells the head visited since the row above.`
    );
  });
</script>

<div class="champions" {@attach inView(() => (seen = true))}>
  <div class="controls">
    <Segmented
      legend="Champion"
      name="{id}-champion"
      options={CHAMPIONS.map((c) => ({ value: c.key, label: c.label, name: c.name }))}
      bind:value={choice}
    />
  </div>

  <div class="layout">
    <Plate
      spec={run?.spec ?? null}
      mode={run && run.steps > EXACT_LIMIT ? 'overview' : 'exact'}
      last={run?.steps ?? 0}
      extent={run?.extent ?? [0, 0]}
      label={description}
      waiting={seen ? 'Computing' : ''}
      failed={failed[choice] ?? false}
      onlayout={(l) => (layout = l)}
    />

    <dl class="readout" aria-live="polite" aria-busy={pending}>
      <div class="machine">
        <dt>Machine</dt>
        <dd><Fact fact={champion.fact} cite={false} /></dd>
      </div>
      <div>
        <dt>Steps until it halts</dt>
        <dd><Live value={run?.steps ?? null} {pending} /></dd>
      </div>
      <div>
        <dt>Ones left on the tape</dt>
        <dd><Live value={run?.ones ?? null} {pending} /></dd>
      </div>
      <div>
        <dt>Time in this browser</dt>
        <dd><Live value={run?.elapsedMs ?? null} {pending} format={formatDuration} /></dd>
      </div>
    </dl>
  </div>

  <p class="reading">{reading}</p>
</div>

<style>
  .controls {
    margin-bottom: var(--space-2);
  }

  .layout {
    display: grid;
    grid-template-columns: minmax(0, 2fr) minmax(0, 1fr);
    gap: var(--space-2) var(--gutter);
    align-items: start;
  }

  .readout {
    display: grid;
    gap: var(--space-2);
    margin: 0;
    font-size: var(--text-sm);
    line-height: 1.4;
  }

  .readout div {
    padding-top: 0.6rem;
    border-top: 1px solid var(--rule);
  }

  .readout div:first-child {
    padding-top: 0;
    border-top: 0;
  }

  dt {
    color: var(--muted);
    margin-bottom: 0.2rem;
  }

  dd {
    margin: 0;
    font-size: var(--text-lg);
    line-height: 1.2;
  }

  .machine dd {
    font-size: var(--text-sm);
    overflow-wrap: anywhere;
  }

  .machine dd :global(.fact) {
    white-space: normal;
  }

  .reading {
    max-width: var(--measure);
    margin: var(--space-2) 0 0;
    font-size: var(--text-sm);
    color: var(--muted);
  }

  @media (max-width: 47.99rem) {
    .layout {
      grid-template-columns: minmax(0, 1fr);
    }

    .layout {
      --plate-height: 24rem;
    }

    .readout {
      grid-template-columns: repeat(2, minmax(0, 1fr));
      gap: var(--space-2) var(--gutter);
    }

    .readout .machine {
      grid-column: 1 / -1;
    }

    .readout div:nth-child(2) {
      padding-top: 0.6rem;
      border-top: 1px solid var(--rule);
    }
  }
</style>
