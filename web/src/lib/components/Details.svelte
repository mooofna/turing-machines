<!--
  A collapsible block for definitions, proofs and other detail, between two hairlines.
  Usage: <Details summary="Formal definition">...</Details>.
-->
<script lang="ts">
  import type { Snippet } from 'svelte';

  interface Props {
    /** The always visible line. */
    summary: string | Snippet;
    /** Expanded; bindable. */
    open?: boolean;
    id?: string;
    children: Snippet;
  }

  let { summary, open = $bindable(false), id, children }: Props = $props();
</script>

<details class="details" {id} bind:open>
  <summary>
    <span class="text">
      {#if typeof summary === 'string'}{summary}{:else}{@render summary()}{/if}
    </span>
    <span class="marker mono" aria-hidden="true">{open ? '−' : '+'}</span>
  </summary>
  <div class="body">
    {@render children()}
  </div>
</details>

<style>
  .details {
    max-width: var(--measure);
    margin-block: var(--space-3);
    border-block: 1px solid var(--rule);
  }

  .details + :global(.details) {
    margin-top: calc(-1 * var(--space-3) - 1px);
  }

  summary {
    display: flex;
    align-items: baseline;
    justify-content: space-between;
    gap: var(--space-2);
    padding-block: 0.7rem;
    font-size: var(--text-sm);
    font-weight: 600;
    cursor: pointer;
    list-style: none;
  }

  summary::-webkit-details-marker {
    display: none;
  }

  summary:hover .marker,
  summary:hover .text {
    color: var(--accent);
  }

  .marker {
    flex: none;
    color: var(--muted);
    font-weight: 400;
  }

  .body {
    padding-bottom: var(--space-2);
  }

  .body :global(> :last-child) {
    margin-bottom: 0;
  }
</style>
