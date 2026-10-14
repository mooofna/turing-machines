<!--
  The numbered bibliography, from `#lib/content/sources.ts`. Each entry has the anchor that
  `<Cite>` links to. With `ids`, only those sources are listed, under their global numbers.
-->
<script lang="ts">
  import type { Source } from '#lib/content/types.ts';
  import { sourceAnchor, sourceNumber, sources } from './bibliography';

  interface Props {
    /** Only these sources; all if omitted. */
    ids?: readonly string[];
    /** Give each entry its anchor; turn off for a second list on the same page. */
    anchors?: boolean;
  }

  let { ids, anchors = true }: Props = $props();

  const entries = $derived(
    (ids ? sources.filter((s) => ids.includes(s.id)) : sources).map((s) => ({
      source: s,
      n: sourceNumber(s.id) ?? 0
    }))
  );

  const link = (s: Source) =>
    s.doi ? `https://doi.org/${s.doi}` : s.arxiv ? `https://arxiv.org/abs/${s.arxiv}` : s.url;
</script>

{#if entries.length > 0}
  <ol class="bibliography">
    {#each entries as { source, n } (source.id)}
      <li id={anchors ? sourceAnchor(source.id) : undefined}>
        <span class="n mono">{n}</span>
        <span class="entry">
          {source.authors}. {#if link(source)}<a href={link(source)}>{source.title}</a
            >{:else}{source.title}{/if}. {source.venue}, {source.year}.
          {#if source.doi}<span class="id mono">doi:{source.doi}</span>{/if}
          {#if source.arxiv}<span class="id mono">arXiv:{source.arxiv}</span>{/if}
        </span>
      </li>
    {/each}
  </ol>
{:else}
  <p class="muted empty">The bibliography is added together with the chapters.</p>
{/if}

<style>
  .bibliography {
    margin: 0;
    padding: 0;
    list-style: none;
    font-size: var(--text-sm);
    line-height: 1.5;
  }

  li {
    display: grid;
    grid-template-columns: 2.25rem minmax(0, 1fr);
    padding-block: 0.6rem;
    border-top: 1px solid var(--rule);
    scroll-margin-top: var(--space-4);
  }

  li:target {
    border-top-color: var(--accent);
  }

  li:target .n {
    color: var(--accent);
  }

  .n {
    color: var(--muted);
  }

  .entry {
    max-width: var(--measure);
  }

  .id {
    color: var(--muted);
    overflow-wrap: anywhere;
  }

  .empty {
    font-size: var(--text-sm);
  }
</style>
