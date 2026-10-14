<!--
  The heading block of a page: the h1 in the content column, an optional label (for example
  "Chapter 1") in the label column and an optional lede below the title.
  Usage: <PageHeader title="The machine" label="Chapter 1" lede="..." />.
-->
<script lang="ts">
  import type { Snippet } from 'svelte';

  interface Props {
    title: string;
    /** Shown in the label column, level with the title. */
    label?: string;
    /** One short paragraph under the title. */
    lede?: string | Snippet;
    /** Further content under the lede, for example the primary action. */
    children?: Snippet;
  }

  let { title, label, lede, children }: Props = $props();
</script>

<header class="page row page-head">
  {#if label}
    <p class="label kicker">{label}</p>
  {/if}
  <h1 class="content">{title}</h1>
  {#if lede}
    <p class="content lede">
      {#if typeof lede === 'string'}{lede}{:else}{@render lede()}{/if}
    </p>
  {/if}
  {#if children}
    <div class="content extra">{@render children()}</div>
  {/if}
</header>

<style>
  .kicker {
    grid-row: 1;
    align-self: baseline;
    margin: 0;
    font-size: var(--text-sm);
    font-weight: 650;
  }

  h1 {
    grid-row: 1;
    align-self: baseline;
  }

  .extra {
    margin-top: var(--space-1);
  }

  @media (max-width: 47.99rem) {
    .kicker,
    h1 {
      grid-row: auto;
    }

    .kicker {
      margin-bottom: calc(-1 * var(--space-1));
      color: var(--muted);
    }
  }
</style>
