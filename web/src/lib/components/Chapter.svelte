<!--
  The layout of a chapter, from the segments that the content pipeline (render.server.ts)
  produced at build time.

  The grid is the one of the homepage: the chapter number sits large in the label column next to
  the title and the lede; every section is a row under a hairline with its heading in the label
  column and its text (at most 38rem wide) in the content column; a figure fills the content
  column with its number and caption in the label column, and below the figure on phones. At the
  end come the sources cited on the page, numbered in order of first citation, the previous and
  next chapter and the list of all chapters.

  Usage: <Chapter chapter={chapterBySlug('machine')} content={data.content} />.
-->
<script lang="ts">
  import { onMount } from 'svelte';
  import ChapterNav from '#lib/components/ChapterNav.svelte';
  import Figure from '#lib/components/Figure.svelte';
  import type { RenderedChapter } from '#lib/content/render.server.ts';
  import { fillLive } from '#lib/content/live.svelte.ts';
  import type { Chapter } from '#lib/content/types.ts';
  import { figureEntry } from '#lib/figures/registry.ts';

  interface Props {
    chapter: Chapter;
    content: RenderedChapter;
  }

  let { chapter, content }: Props = $props();

  let article = $state<HTMLElement>();

  // Display formulas and tables wider than the column scroll sideways; only those that actually
  // overflow become focusable, so that keyboard users can scroll them.
  onMount(() => {
    if (!article) return;
    const regions = [...article.querySelectorAll<HTMLElement>('.math-display, .table-wrap')];
    const check = () => {
      for (const region of regions) {
        const scrolls = region.scrollWidth > region.clientWidth + 1;
        if (scrolls) {
          region.tabIndex = 0;
          region.setAttribute('role', 'region');
          region.setAttribute(
            'aria-label',
            region.classList.contains('table-wrap')
              ? 'Table, scrolls sideways'
              : 'Formula, scrolls sideways'
          );
        } else {
          region.removeAttribute('tabindex');
          region.removeAttribute('role');
          region.removeAttribute('aria-label');
        }
      }
    };
    const observer = new ResizeObserver(check);
    for (const region of regions) observer.observe(region);
    return () => observer.disconnect();
  });
</script>

<svelte:head>
  <title>{chapter.title}, Turing machines</title>
  <meta name="description" content={chapter.summary} />
</svelte:head>

<article class="chapter" bind:this={article} {@attach fillLive}>
  <header class="page row chapter-head">
    <p class="label chapter-number" aria-hidden="true">{chapter.n}</p>
    <div class="content">
      <p class="kicker">Chapter {chapter.n}</p>
      <h1>{content.title}</h1>
      {#if content.lede}
        <div class="lede-text prose">{@html content.lede}</div>
      {/if}
    </div>
  </header>

  {#each content.segments as segment, i (segment.kind === 'figure' ? segment.anchor : `${segment.id}-${i}`)}
    {#if segment.kind === 'section' && segment.heading !== null}
      <section class="page row section part" id={segment.id} aria-labelledby="{segment.id}-title">
        <div class="label">
          <h2 id="{segment.id}-title" class="section-name">{segment.heading}</h2>
        </div>
        <div class="content prose">{@html segment.html}</div>
      </section>
    {:else if segment.kind === 'section'}
      <div class="page row continuation">
        <div class="content prose">{@html segment.html}</div>
      </div>
    {:else}
      {@const entry = figureEntry(segment.id)}
      {#snippet caption()}{@html segment.caption}{/snippet}
      <Figure n={segment.number ?? undefined} id={segment.anchor} {caption} class="chapter-figure">
        <entry.component id={segment.id} aspect={entry.aspect} aspectPhone={entry.aspectPhone} />
      </Figure>
    {/if}
  {/each}

  {#if content.references.length > 0}
    <section class="page row section sources" aria-labelledby="sources-title">
      <div class="label">
        <h2 id="sources-title" class="section-name">Sources</h2>
        <p class="aside">Numbered in the order of first citation in this chapter.</p>
      </div>
      <ol class="content references">
        {#each content.references as reference (reference.id)}
          <li id="ref-{reference.n}">
            <span class="n mono">{reference.n}</span>
            <span class="entry">{@html reference.html}</span>
          </li>
        {/each}
      </ol>
    </section>
  {/if}

  <div class="pager">
    <ChapterNav current={chapter.slug} variant="pager" />
  </div>

  <section class="page row section all-chapters" aria-labelledby="chapters-title">
    <div class="label">
      <h2 id="chapters-title" class="section-name">Chapters</h2>
    </div>
    <div class="content">
      <ChapterNav current={chapter.slug} label="All chapters" />
    </div>
  </section>
</article>

<style>
  /* Head: the number large in the label column, title and lede in the content column. */
  .chapter-head {
    padding-block: var(--space-5) var(--space-4);
  }

  .chapter-number {
    margin: 0;
    font-size: var(--text-display);
    font-weight: 650;
    line-height: 0.82;
    letter-spacing: -0.04em;
    font-variant-numeric: tabular-nums;
  }

  .kicker {
    display: none;
    margin: 0 0 var(--space-1);
    font-size: var(--text-sm);
    font-weight: 650;
    color: var(--muted);
  }

  h1 {
    font-size: var(--text-xl);
    max-width: var(--measure);
  }

  .lede-text {
    margin-top: var(--space-3);
    font-size: clamp(1.125rem, 1rem + 0.4vw, 1.3rem);
    line-height: 1.5;
    letter-spacing: -0.003em;
  }

  .lede-text :global(p) {
    margin-bottom: 0.6em;
  }

  @media (max-width: 47.99rem) {
    .chapter-head {
      padding-block: var(--space-4) var(--space-3);
    }

    .chapter-number {
      display: none;
    }

    .kicker {
      display: block;
    }

    .lede-text {
      font-size: 1.125rem;
    }
  }

  /* Sections. The heading stays in view while its section scrolls past on wide screens. */
  .part .label {
    min-width: 0;
  }

  .part .section-name {
    font-size: var(--text-base);
    line-height: 1.3;
    letter-spacing: -0.01em;
    text-wrap: balance;
  }

  @media (min-width: 48rem) {
    .part .section-name {
      position: sticky;
      top: var(--space-3);
      padding-right: var(--space-2);
    }
  }

  .continuation {
    padding-bottom: var(--space-5);
  }

  @media (max-width: 47.99rem) {
    .part {
      padding-block: var(--space-3) var(--space-4);
    }

    .continuation {
      padding-bottom: var(--space-4);
    }
  }

  .part:has(+ :global(.chapter-figure)) {
    padding-bottom: var(--space-4);
  }

  /* Running text. */
  .prose :global(:is(p, ul, ol, details, .math-display, .table-wrap)) {
    max-width: var(--measure);
  }

  .prose :global(ul),
  .prose :global(ol) {
    margin: 0 0 1em;
    padding-left: 1.3em;
  }

  .prose :global(li) {
    margin-bottom: 0.45em;
    padding-left: 0.15em;
  }

  .prose :global(li::marker) {
    color: var(--muted);
    font-variant-numeric: tabular-nums;
  }

  .prose :global(strong) {
    font-weight: 650;
  }

  .prose :global(code) {
    font-size: 0.84em;
    white-space: nowrap;
  }

  .prose :global(:last-child) {
    margin-bottom: 0;
  }

  /* Facts, citations and live values. */
  .chapter :global(.fact.short) {
    white-space: nowrap;
  }

  .chapter :global(.fact sup),
  .chapter :global(.live sup) {
    font-size: 0.72em;
  }

  .chapter :global(sup.cite) {
    margin-left: 0.08em;
    font-family: var(--font-mono);
    font-size: 0.66em;
    font-variant-numeric: tabular-nums;
    vertical-align: super;
    line-height: 0;
    white-space: nowrap;
  }

  .chapter :global(sup.cite a) {
    color: var(--muted);
    text-decoration: none;
    padding-inline: 0.05em;
  }

  .chapter :global(sup.cite a:hover) {
    color: var(--accent);
  }

  .chapter :global(sup.cite .sep) {
    color: var(--muted);
  }

  .chapter :global(.live),
  .chapter :global(.nobr) {
    white-space: nowrap;
  }

  .chapter :global(.live[data-state='pending']) {
    color: var(--muted);
  }

  /* Mathematics: inline formulas sit on the baseline; display formulas are centred in the
     measure and scroll sideways when they are wider than the column. */
  /* Temml's font for primes, so that q' is not raised twice (from the temml package). */
  @font-face {
    font-family: 'Temml';
    src: url('../../../node_modules/temml/dist/Temml.woff2') format('woff2');
    font-display: swap;
  }

  .chapter :global(mo.tml-prime) {
    font-family: 'Temml';
  }

  .chapter :global(math) {
    font-family: 'STIX Two Math', 'STIXTwoMath-Regular', 'Cambria Math', 'Latin Modern Math', math;
    font-size: 1.08em;
  }

  .chapter :global(.math-display) {
    display: flex;
    margin-block: 1.1em;
    overflow-x: auto;
    overflow-y: hidden;
    padding-block: 0.15em;
  }

  .chapter :global(.math-display math) {
    flex: none;
    margin: 0 auto;
  }

  /* Tables. */
  .chapter :global(.table-wrap) {
    margin-block: var(--space-3);
    overflow-x: auto;
  }

  .chapter :global(.table-wrap .table) {
    min-width: 100%;
    width: auto;
  }

  .chapter :global(.table caption) {
    caption-side: top;
    padding-bottom: var(--space-1);
    text-align: left;
    color: var(--muted);
    font-size: var(--text-sm);
  }

  .chapter :global(.table td),
  .chapter :global(.table th) {
    padding-right: 1.25rem;
  }

  .chapter :global(.table thead th.num) {
    font-family: inherit;
    font-size: inherit;
  }

  /* Details: between two hairlines, with a plus or minus at the right. */
  .chapter :global(details) {
    margin-block: var(--space-3);
    border-block: 1px solid var(--rule);
  }

  .chapter :global(details + details) {
    margin-top: calc(-1 * var(--space-3) - 1px);
  }

  .chapter :global(summary) {
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

  .chapter :global(summary::-webkit-details-marker) {
    display: none;
  }

  .chapter :global(summary::after) {
    content: '+';
    flex: none;
    font-family: var(--font-mono);
    font-weight: 400;
    color: var(--muted);
  }

  .chapter :global(details[open] > summary::after) {
    content: '\2212';
  }

  .chapter :global(summary:hover),
  .chapter :global(summary:hover::after) {
    color: var(--accent);
  }

  .chapter :global(details[open] > summary) {
    margin-bottom: var(--space-1);
  }

  .chapter :global(details > :last-child) {
    margin-bottom: var(--space-3);
  }

  /* Sources. */
  .references {
    max-width: calc(var(--measure) + 2.25rem);
    margin: 0;
    padding: 0;
    list-style: none;
    font-size: var(--text-sm);
    line-height: 1.5;
  }

  .references li {
    display: grid;
    grid-template-columns: 2.25rem minmax(0, 1fr);
    padding-block: 0.6rem;
    border-top: 1px solid var(--rule);
    scroll-margin-top: var(--space-4);
  }

  .references li:first-child {
    border-top: 0;
    padding-top: 0;
  }

  .references li:target {
    border-top-color: var(--accent);
  }

  .references li:target .n {
    color: var(--accent);
  }

  .references .n {
    color: var(--muted);
  }

  .references .entry {
    max-width: var(--measure);
    overflow-wrap: anywhere;
  }

  .references :global(.ref-id),
  .references :global(.ref-note) {
    color: var(--muted);
  }

  .aside {
    margin-top: 0.35rem;
    font-size: var(--text-sm);
    color: var(--muted);
    max-width: 14rem;
  }

  .pager {
    padding-block: var(--space-4) var(--space-5);
  }

  .all-chapters .content {
    max-width: var(--measure);
  }
</style>
