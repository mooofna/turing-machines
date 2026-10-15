<!--
  Fig. 5.2: one machine per decider family of chapter 5, with its space-time diagram, the
  verdict of this site's decider pipeline and the evidence.

  The machines are fixed examples chosen so that each is resolved by its family and by no
  earlier stage of the pipeline; the verdict is computed in this browser with `decide`. For the
  cycler and the translated cycler the figure also finds the repetition in the run itself, step
  by step, and brackets it in the diagram. The certificates of the other two deciders (closed
  sets of local views or of abstract word lists) are not shown; the text says what they are.
-->
<script lang="ts">
  import { onDestroy } from 'svelte';
  import { formatInteger } from '#lib/components/Live.svelte';
  import { engine } from '#lib/engine/client.ts';
  import type { MachineSpecDto, SnapshotDto, VerdictDto } from '#lib/engine/pkg/turing_wasm.js';
  import { editor } from '#lib/engine/wasm.ts';
  import type { FigureProps } from '../registry.ts';
  import { inView } from '../ch4/common/inview.ts';
  import Plate from '../ch4/common/Plate.svelte';
  import Segmented from '../ch4/common/Segmented.svelte';
  import { cycle, translation, type Frame } from './evidence.ts';

  let { id }: FigureProps = $props();

  type Key = 'cycler' | 'translated' | 'ngram' | 'repwl';

  interface Example {
    key: Key;
    label: string;
    /** The decider family expected to resolve the machine. */
    decider: string;
    machine: string;
    /** Steps shown in the diagram. */
    steps: number;
    /** What the decider looks for, after chapter 5. */
    looksFor: string;
  }

  const EXAMPLES: readonly Example[] = [
    {
      key: 'cycler',
      label: 'Cyclers',
      decider: 'loops',
      machine: '1RB0RD_0RC---_1LC0LA_1RA0LC',
      steps: 63,
      looksFor: 'the same configuration, state, head position and tape, at two different times.'
    },
    {
      key: 'translated',
      label: 'Translated cyclers',
      decider: 'loops',
      machine: '0RB---_1RD1RA_1RA0LA_1LC1RA',
      steps: 99,
      looksFor:
        'two times at which the head is on a new cell at the edge of the visited region, in the same state, with the same cells behind it.'
    },
    {
      key: 'ngram',
      label: 'n-gram CPS',
      decider: 'ngram-cps',
      machine: '1RB0LD_1LA0RC_1RB---_1LA0RB',
      steps: 399,
      looksFor:
        'a set of short words that may occur on the tape and of situations at the head that contains the start, is closed under the steps of the machine and contains no situation that halts.'
    },
    {
      key: 'repwl',
      label: 'Repeated word lists',
      decider: 'repwl',
      machine: '1RB0RB_1LC1RB_---0LD_1RA1LD',
      steps: 399,
      looksFor:
        'a closed set of abstract configurations in which the tape is a list of repeated words, every count above a threshold replaced by "at least", and none of them halts.'
    }
  ];

  interface Result {
    spec: MachineSpecDto;
    verdict: VerdictDto;
    extent: [number, number];
    evidence: string;
    mark: { from: number; to: number; text: string } | null;
  }

  let choice = $state<Key>('cycler');
  let seen = $state(false);
  let results = $state<Partial<Record<Key, Result>>>({});
  let failed = $state<Partial<Record<Key, boolean>>>({});
  let destroyed = false;
  const started = new Set<Key>();

  const example = $derived(EXAMPLES.find((e) => e.key === choice) ?? EXAMPLES[0]!);
  const result = $derived(results[choice]);
  const pending = $derived(seen && !result && !failed[choice]);

  function frame(snapshot: SnapshotDto, blank: string): Frame {
    const window = snapshot.tapes[0]!;
    const tape = new Map<number, string>();
    window.cells.forEach((glyph, i) => {
      if (glyph !== blank) tape.set(window.start + i, glyph);
    });
    return { state: snapshot.state, head: window.head, tape };
  }

  function describe(e: Example, frames: Frame[]): Pick<Result, 'evidence' | 'mark'> {
    if (e.key === 'cycler') {
      const found = cycle(frames);
      if (!found) return { evidence: 'No repetition within the steps shown.', mark: null };
      const period = found.to - found.from;
      return {
        evidence:
          `The configuration after step ${found.from} occurs again after step ${found.to}: ` +
          `the same state, ${frames[found.from]!.state}, the same head position and the same ` +
          `tape. From step ${found.from} on, the run repeats every ${period} steps.`,
        mark: { from: found.from, to: found.to, text: `${period} steps` }
      };
    }
    const found = translation(frames);
    if (!found) return { evidence: 'No repetition within the steps shown.', mark: null };
    const period = found.to - found.from;
    const side = found.shift > 0 ? 'right' : 'left';
    const cells = Math.abs(found.shift);
    return {
      evidence:
        `After steps ${found.from} and ${found.to} the head is on a new cell at the ${side} ` +
        `end of the visited cells, in state ${found.state}, and the ${found.cells} cells up to ` +
        `the head agree. From step ${found.from} on, the run repeats every ${period} steps, ` +
        `${cells} ${cells === 1 ? 'cell' : 'cells'} further ${side}.`,
      mark: { from: found.from, to: found.to, text: `${period} steps` }
    };
  }

  async function study(key: Key): Promise<void> {
    if (started.has(key)) return;
    started.add(key);
    const e = EXAMPLES.find((x) => x.key === key)!;
    const worker = engine();
    const session = `${id}-${key}`;
    let opened = false;
    try {
      const spec = (await editor()).fromStandard(e.machine).spec;
      if (!spec || destroyed) throw new Error(`cannot parse ${e.machine}`);
      const { verdict } = await worker.request({ type: 'decide', spec }, 'verdict');
      const radius = e.steps + 2;
      opened = true;
      const first = await worker.request(
        { type: 'open', session, spec, input: '', radius },
        'snapshot'
      );
      let extent: [number, number];
      let found: Pick<Result, 'evidence' | 'mark'> = { evidence: '', mark: null };
      if (e.decider === 'loops') {
        // Every configuration of the excerpt, for the evidence.
        const frames = [frame(first.snapshot, spec.blank)];
        let last = first.snapshot;
        for (let t = 1; t <= e.steps && !destroyed; t++) {
          last = (await worker.request({ type: 'step', session, count: 1, radius }, 'snapshot'))
            .snapshot;
          frames.push(frame(last, spec.blank));
        }
        extent = last.extent;
        found = describe(e, frames);
      } else {
        const last = await worker.request(
          { type: 'step', session, count: e.steps, radius: 0 },
          'snapshot'
        );
        extent = last.snapshot.extent;
      }
      if (destroyed) return;
      results[key] = { spec, verdict, extent, ...found };
    } catch {
      if (!destroyed) failed[key] = true;
    } finally {
      if (opened) worker.call({ type: 'close', session });
    }
  }

  $effect(() => {
    if (seen) void study(choice);
  });

  onDestroy(() => {
    destroyed = true;
  });

  const verdictText = $derived.by(() => {
    const v = result?.verdict;
    if (!v) return '';
    switch (v.kind) {
      case 'non-halting':
        return 'Never halts';
      case 'halts':
        return `Halts after ${formatInteger(v.steps)} steps`;
      case 'unknown':
        return 'Not decided';
      case 'unsupported':
        return `Not supported: ${v.reason}`;
    }
    return '';
  });

  const deciderText = $derived(result?.verdict.kind === 'non-halting' ? result.verdict.label : '');

  const description = $derived(
    result
      ? `Space-time diagram of ${example.machine} for its first ${formatInteger(example.steps)} ` +
          `steps from the blank tape. Time runs downwards.`
      : 'Space-time diagram, not computed yet.'
  );
</script>

<div class="deciders" {@attach inView(() => (seen = true))}>
  <div class="controls">
    <Segmented
      legend="Decider"
      name="{id}-decider"
      options={EXAMPLES.map((e) => ({ value: e.key, label: e.label }))}
      bind:value={choice}
    />
  </div>

  <div class="layout">
    <Plate
      spec={result?.spec ?? null}
      mode="exact"
      last={example.steps}
      extent={result?.extent ?? [0, 0]}
      label={description}
      mark={result?.mark ?? null}
      waiting={seen ? 'Computing' : ''}
      failed={failed[choice] ?? false}
    />

    <dl class="readout" aria-live="polite" aria-busy={pending}>
      <div>
        <dt>Machine</dt>
        <dd class="mono machine">{example.machine}</dd>
      </div>
      <div>
        <dt>Verdict in this browser</dt>
        <dd>
          {#if result}{verdictText}{#if deciderText}<span class="by">{deciderText}</span>{/if}
          {:else}<span class="muted">{failed[choice] ? 'Not available' : ''}</span>{/if}
        </dd>
      </div>
      <div>
        <dt>Evidence</dt>
        <dd class="evidence">
          {#if result?.evidence}
            {result.evidence}
          {:else if result?.verdict.kind === 'non-halting' && result.verdict.decider === example.decider}
            The decider found {example.looksFor}
          {:else if result}
            The verdict does not come from the decider this example is meant to show.
          {/if}
        </dd>
      </div>
    </dl>
  </div>

  <p class="reading">
    Each row is the tape after one more step, from the blank tape at step 0 to step
    {formatInteger(example.steps)}; filled cells hold a 1 and the blue cell is the head. The run
    continues below the excerpt.
  </p>
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
    line-height: 1.45;
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
  }

  .machine {
    overflow-wrap: anywhere;
  }

  .by {
    display: block;
    color: var(--muted);
  }

  .evidence {
    text-wrap: pretty;
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
      --plate-height: 24rem;
    }
  }
</style>
