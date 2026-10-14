<!--
  A numbered figure. On wide screens the number and caption sit in the label column next to the
  figure, which fills the content column; with `wide` the figure spans all twelve columns and
  the caption follows below. On phones the caption always follows the figure.

  Place it between sections, at the level of the page. Inside a section's content column it
  falls back to a figure with the caption below.
  Usage: <Figure n="1.1" caption="The binary increment machine on 1011.">...</Figure>.
-->
<script lang="ts">
  import type { Snippet } from 'svelte';

  interface Props {
    /** Figure number, for example `1.1`; omit for an unnumbered figure. */
    n?: string | number;
    /** One or two plain sentences: what is shown and how to read it. */
    caption: string | Snippet;
    /** Span all twelve columns. */
    wide?: boolean;
    id?: string;
    class?: string;
    children: Snippet;
  }

  let { n, caption, wide = false, id, class: className = '', children }: Props = $props();
</script>

<figure class="page row figure {className}" class:wide {id}>
  <div class="body">
    {@render children()}
  </div>
  <figcaption>
    {#if n !== undefined}<span class="number">Figure {n}</span>{/if}
    <span class="text"
      >{#if typeof caption === 'string'}{caption}{:else}{@render caption()}{/if}</span
    >
  </figcaption>
</figure>

<style>
  .figure {
    margin-block: 0;
    padding-block: 0 var(--space-5);
  }

  .body {
    grid-column: 4 / -1;
    grid-row: 1;
    min-width: 0;
  }

  figcaption {
    grid-column: 1 / span 3;
    grid-row: 1;
    font-size: var(--text-sm);
    line-height: 1.45;
  }

  .number {
    display: block;
    margin-bottom: 0.35rem;
    font-weight: 650;
  }

  .text {
    display: block;
    color: var(--muted);
    text-wrap: pretty;
  }

  .wide .body {
    grid-column: 1 / -1;
  }

  .wide figcaption {
    grid-column: 1 / -1;
    grid-row: 2;
    display: grid;
    grid-template-columns: subgrid;
    margin-top: var(--space-2);
  }

  .wide .number {
    grid-column: 1 / span 3;
    margin: 0;
  }

  .wide .text {
    grid-column: 4 / -1;
    max-width: var(--measure);
  }

  @media (max-width: 47.99rem) {
    .body,
    figcaption,
    .wide figcaption {
      grid-column: 1 / -1;
      grid-row: auto;
    }

    figcaption,
    .wide figcaption {
      display: block;
      margin-top: var(--space-2);
    }

    .number {
      display: inline;
      margin-right: 0.4rem;
    }

    .text {
      display: inline;
    }
  }

  /* Inside a section's content column: a plain figure with the caption below. */
  :global(.content) .figure {
    display: block;
    width: auto;
    margin-block: var(--space-3);
    padding: 0;
  }

  :global(.content) figcaption {
    display: block;
    margin-top: var(--space-2);
    max-width: var(--measure);
  }

  :global(.content) .number {
    display: inline;
    margin-right: 0.4rem;
  }

  :global(.content) .text {
    display: inline;
  }
</style>
