<!--
  A section of a page: a grid row under a hairline, its name in the label column and its content
  in the content column, as on the author's homepage.
  Usage: <Section name="Configurations" id="configurations">...</Section>.
-->
<script lang="ts">
  import type { Snippet } from 'svelte';

  interface Props {
    /** Name of the section, shown in the label column. */
    name: string;
    /** Anchor id of the section. */
    id?: string;
    /** Heading level of the name. */
    level?: 2 | 3;
    /** Content below the name in the label column, for example a link. */
    aside?: Snippet;
    /** Let the content use the full width of the content column instead of the text measure. */
    wide?: boolean;
    class?: string;
    children: Snippet;
  }

  let {
    name,
    id,
    level = 2,
    aside,
    wide = false,
    class: className = '',
    children
  }: Props = $props();

  const uid = $props.id();
  const headingId = $derived(id ? `${id}-title` : `section-${uid}`);
</script>

<section class="page row section {className}" {id} aria-labelledby={headingId}>
  <div class="label">
    <svelte:element this={`h${level}`} id={headingId} class="section-name">{name}</svelte:element>
    {#if aside}
      <div class="aside">{@render aside()}</div>
    {/if}
  </div>
  <div class="content" class:prose={!wide}>
    {@render children()}
  </div>
</section>

<style>
  .label {
    margin-bottom: var(--space-3);
  }

  .label .section-name {
    margin-bottom: 0;
  }

  .aside {
    margin-top: 0.35rem;
    font-size: var(--text-sm);
    color: var(--muted);
  }

  .prose > :global(p),
  .prose > :global(ul),
  .prose > :global(ol) {
    max-width: var(--measure);
  }
</style>
