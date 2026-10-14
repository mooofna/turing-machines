<!--
  The space-time diagram of the playground's machine: every row is the tape at one step. Short
  runs are drawn step by step, long runs as an even overview up to the step the run reached.
-->
<script lang="ts">
  import type { MachineSpecDto, SpaceTimeModeDto } from '#lib/engine/pkg/turing_wasm.js';
  import { count } from '#lib/machine/describe.ts';
  import SpaceTime from '#lib/machine/SpaceTime.svelte';

  interface Props {
    spec: MachineSpecDto | null;
    input: string;
    /** Steps of the last run; 0 before the first run. */
    steps: number;
  }

  let { spec, input, steps }: Props = $props();

  let width = $state(0);

  /** Rows drawn one per step up to this length; longer runs are sampled. */
  const EXACT = 400;
  const exact = $derived(steps <= EXACT);
  const cellPx = $derived(
    exact ? Math.max(2, Math.min(6, Math.floor(360 / Math.max(steps, 60)))) : 1
  );
  const rows = $derived(exact ? Math.max(steps, 60) + 1 : 0);
  const height = $derived(exact ? Math.min(420, rows * cellPx) : 360);
  const mode = $derived<SpaceTimeModeDto>(
    exact ? { kind: 'exact', firstStep: 0, cellPx } : { kind: 'overview', maxSteps: steps }
  );
  const label = $derived(
    `Space-time diagram of ${spec?.name || 'the machine'} on ${
      input === '' ? 'the blank tape' : `input ${input}`
    }, ${exact ? `the first ${count(rows)} steps` : `${count(steps)} steps sampled evenly`}`
  );
</script>

<div class="slot" bind:clientWidth={width}>
  {#if !spec}
    <p class="muted">There is no machine to draw.</p>
  {:else if spec.tapes !== 1}
    <p class="muted">Space-time diagrams show machines with one tape.</p>
  {:else if width > 0}
    <SpaceTime {spec} {input} {mode} aspect={width / height} {label} />
    <p class="caption">
      Each row is the tape at one step, from step 0 at the top. {exact
        ? `One row per step, ${count(rows)} rows.`
        : `The rows sample the ${count(steps)} steps of the last run evenly.`}
    </p>
  {/if}
</div>

<style>
  .slot {
    min-height: 4rem;
  }

  .caption {
    margin: 0.6rem 0 0;
    max-width: var(--measure);
    font-size: var(--text-xs);
    color: var(--muted);
  }

  p.muted {
    font-size: var(--text-sm);
  }
</style>
