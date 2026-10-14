<!--
  Mathematics as MathML, produced at prerender time (temml in +page.server.ts), so no
  mathematics code runs in the browser.
  Usage: <Math mathml={data.math.delta} display />.
-->
<script lang="ts">
  import { onMount } from 'svelte';

  interface Props {
    /** A complete <math> element, generated from trusted source text at build time. */
    mathml: string;
    /** A displayed formula on its own line, scrolling sideways if it is wider than the column. */
    display?: boolean;
    /** Name of the formula for screen readers, if it scrolls. */
    label?: string;
  }

  let { mathml, display = false, label = 'Formula' }: Props = $props();

  let block = $state<HTMLDivElement>();
  // A formula wider than the column scrolls; keyboard users can then focus it to scroll.
  let scrollable = $state(false);

  onMount(() => {
    if (!block) return;
    const element = block;
    const check = () => (scrollable = element.scrollWidth > element.clientWidth + 1);
    const observer = new ResizeObserver(check);
    observer.observe(element);
    return () => observer.disconnect();
  });
</script>

{#if display}
  <!-- svelte-ignore a11y_no_noninteractive_tabindex -->
  <div
    bind:this={block}
    class="math display"
    tabindex={scrollable ? 0 : undefined}
    role={scrollable ? 'group' : undefined}
    aria-label={scrollable ? label : undefined}
  >
    {@html mathml}
  </div>
{:else}
  <span class="math">{@html mathml}</span>
{/if}

<style>
  .display {
    max-width: 100%;
    margin-block: var(--space-2);
    overflow-x: auto;
    overflow-y: hidden;
  }

  /* Left-aligned with the text: block math would be centred in the column. Temml sets
     display: block math as an inline style, hence the !important. */
  .display :global(math) {
    display: math !important;
    margin: 0;
    math-style: normal;
  }
</style>
