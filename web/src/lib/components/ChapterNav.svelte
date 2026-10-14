<!--
  The six chapters.

  variant="list" (default): a numbered list of all chapters, with the current one marked; with
  `summaries` each entry carries its one-sentence summary (the landing page).
  variant="pager": links to the previous and the next chapter, for the bottom of a chapter.
  Usage: <ChapterNav current="machine" /> and <ChapterNav current="machine" variant="pager" />.
-->
<script lang="ts">
  import { resolve } from '$app/paths';
  import type { RouteId } from '$app/types';
  import { chapters } from '#lib/content/chapters.ts';
  import type { Chapter } from '#lib/content/types.ts';

  interface Props {
    /** Slug of the current chapter, if any. */
    current?: string;
    variant?: 'list' | 'pager';
    /** Show the summary under each title (list only). */
    summaries?: boolean;
    /** Accessible name of the navigation. */
    label?: string;
  }

  let { current, variant = 'list', summaries = false, label }: Props = $props();

  const href = (c: Chapter) => resolve(`/${c.slug}` as RouteId);
  const index = $derived(chapters.findIndex((c) => c.slug === current));
  const previous = $derived(index > 0 ? chapters[index - 1] : undefined);
  const next = $derived(
    index >= 0 && index < chapters.length - 1 ? chapters[index + 1] : undefined
  );
</script>

{#if variant === 'list'}
  <nav class="list" class:summaries aria-label={label ?? 'Chapters'}>
    <ol>
      {#each chapters as chapter (chapter.slug)}
        {@const here = chapter.slug === current}
        <li>
          <a href={href(chapter)} aria-current={here ? 'page' : undefined}>
            <span class="n mono">{chapter.n}</span>
            <span class="title">{chapter.title}</span>
            {#if summaries}
              <span class="summary">{chapter.summary}</span>
            {/if}
          </a>
        </li>
      {/each}
    </ol>
  </nav>
{:else if previous || next}
  <nav class="pager page" aria-label={label ?? 'Previous and next chapter'}>
    {#if previous}
      <a class="prev" href={href(previous)} rel="prev">
        <span class="dir">Previous chapter</span>
        <span class="title"><span class="n mono">{previous.n}</span> {previous.title}</span>
      </a>
    {/if}
    {#if next}
      <a class="next" href={href(next)} rel="next">
        <span class="dir">Next chapter</span>
        <span class="title"><span class="n mono">{next.n}</span> {next.title}</span>
      </a>
    {/if}
  </nav>
{/if}

<style>
  ol {
    margin: 0;
    padding: 0;
    list-style: none;
  }

  /* Compact list, for a label column or a menu. */
  .list a {
    display: grid;
    grid-template-columns: 1.5rem minmax(0, 1fr);
    column-gap: 0.5rem;
    padding-block: 0.3rem;
    font-size: var(--text-sm);
    line-height: 1.4;
    text-decoration: none;
    color: var(--muted);
  }

  .list a:hover {
    color: var(--text);
  }

  .list a:hover .title {
    text-decoration: underline;
    text-decoration-color: var(--accent);
    text-decoration-thickness: 2px;
    text-underline-offset: 0.18em;
  }

  .list .n {
    font-size: var(--text-xs);
    line-height: inherit;
  }

  .list a[aria-current='page'] {
    color: var(--text);
    font-weight: 600;
  }

  .list a[aria-current='page'] .n {
    color: var(--accent);
    font-weight: 500;
  }

  /* With summaries: one row per chapter between hairlines, as on the landing page. */
  .summaries li {
    border-top: 1px solid var(--rule);
  }

  @media (max-width: 47.99rem) {
    .summaries a {
      grid-template-columns: 2rem minmax(0, 1fr);
    }
  }

  .summaries a {
    grid-template-columns: 2.5rem minmax(0, 1fr);
    align-items: baseline;
    row-gap: 0.25rem;
    padding-block: var(--space-2) calc(var(--space-2) + 0.1rem);
    font-size: var(--text-base);
    line-height: 1.45;
    color: var(--text);
  }

  .summaries .n {
    font-size: var(--text-sm);
    font-weight: 500;
    color: var(--accent);
  }

  .summaries .title {
    font-size: var(--text-lg);
    font-weight: 650;
    letter-spacing: -0.01em;
    line-height: 1.2;
  }

  .summaries .summary {
    grid-column: 2;
    max-width: var(--measure);
    color: var(--muted);
    font-size: var(--text-sm);
  }

  .summaries a:hover .title {
    text-decoration: underline;
  }

  /* Pager. */
  .pager {
    display: grid;
    grid-template-columns: repeat(2, minmax(0, 1fr));
    gap: var(--gutter);
    padding-block: var(--space-3) 0;
    border-top: 1px solid var(--rule-strong);
  }

  .pager a {
    display: flex;
    flex-direction: column;
    gap: 0.2rem;
    text-decoration: none;
  }

  .pager .next {
    grid-column: 2;
    text-align: right;
  }

  .pager .dir {
    font-size: var(--text-sm);
    color: var(--muted);
  }

  .pager .title {
    font-size: var(--text-lg);
    font-weight: 650;
    letter-spacing: -0.01em;
    line-height: 1.2;
  }

  .pager .n {
    font-size: 0.7em;
    font-weight: 500;
    color: var(--accent);
    margin-right: 0.3rem;
  }

  .pager a:hover .title {
    text-decoration: underline;
    text-decoration-color: var(--accent);
    text-decoration-thickness: 2px;
    text-underline-offset: 0.18em;
  }

  @media (max-width: 47.99rem) {
    .pager {
      grid-template-columns: minmax(0, 1fr);
      row-gap: var(--space-3);
    }

    .pager .next {
      grid-column: 1;
      text-align: left;
    }
  }
</style>
