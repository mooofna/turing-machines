<!--
  The place of the space-time diagram in the playground.

  INTEGRATION: replace the placeholder below with A5's lib/machine/SpaceTime.svelte once it is
  merged, passing the same props (spec, input, step, onseek). Until then this shows where the
  diagram goes and what it will show.
-->
<script lang="ts">
  import type { MachineSpecDto } from '#lib/engine/pkg/turing_wasm.js';

  interface Props {
    spec: MachineSpecDto | null;
    input: string;
    /** The current step, marked in the diagram. */
    step: number;
    /** Called when a row of the diagram is chosen. */
    onseek?: (step: number) => void;
  }

  let { spec, input, step }: Props = $props();
  const single = $derived(spec?.tapes === 1);
</script>

<div class="slot" data-spacetime-slot>
  <p class="muted">
    {#if !spec}
      There is no machine to draw.
    {:else if !single}
      Space-time diagrams show machines with one tape.
    {:else}
      Each row of the diagram is the tape at one step, from step 0 at the top to step {step} on input
      {input === '' ? 'blank' : input}.
    {/if}
  </p>
</div>

<style>
  .slot {
    display: grid;
    place-items: center;
    min-height: 12rem;
    padding: var(--space-3);
    border: 1px solid var(--rule);
    font-size: var(--text-sm);
    text-align: center;
  }
</style>
