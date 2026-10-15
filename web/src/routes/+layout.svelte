<script lang="ts">
  import '../app.css';
  import { resolve } from '$app/paths';
  import { page } from '$app/state';
  import ThemeToggle from '#lib/components/ThemeToggle.svelte';
  import { chapterByRoute } from '#lib/content/chapters.ts';

  let { children } = $props();

  const SITE = 'Turing machines';
  // Absolute addresses for link previews (Open Graph), which crawlers need in full.
  const ORIGIN = 'https://mooofna.github.io';
  const OG_IMAGE = `${ORIGIN}/turing-machines/og.png`;
  const DESCRIPTION =
    'Turing machines from Turing’s definition of 1936 to the open problems of the busy beaver function, simulated in the browser with Rust and WebAssembly.';

  // Titles of the pages that are not chapters. A page can override its title and description by
  // returning `title` and `description` from its load function.
  const TITLES: Record<string, string> = {
    '/playground': 'Playground',
    '/gallery': 'Gallery',
    '/about': 'About'
  };

  const chapter = $derived(chapterByRoute(page.route.id));
  const title = $derived.by(() => {
    const own = (page.data as { title?: string }).title;
    const name = own ?? chapter?.title ?? TITLES[page.route.id ?? ''];
    return name ? `${name} | ${SITE}` : SITE;
  });
  const description = $derived(
    (page.data as { description?: string }).description ?? chapter?.summary ?? DESCRIPTION
  );

  const links = [
    { href: `${resolve('/')}#chapters`, label: 'Chapters', chapters: true, route: '' },
    { href: resolve('/playground'), label: 'Playground', chapters: false, route: '/playground' },
    { href: resolve('/gallery'), label: 'Gallery', chapters: false, route: '/gallery' },
    { href: resolve('/about'), label: 'About', chapters: false, route: '/about' }
  ];

  function current(link: (typeof links)[number]): 'page' | 'true' | undefined {
    if (link.chapters) return chapter ? 'true' : undefined;
    return page.route.id === link.route ? 'page' : undefined;
  }
</script>

<svelte:head>
  <title>{title}</title>
  <meta name="description" content={description} />
  <meta name="author" content="Maurice Heide" />
  <link rel="canonical" href={`${ORIGIN}${page.url.pathname}`} />
  <meta property="og:type" content="website" />
  <meta property="og:site_name" content={SITE} />
  <meta property="og:title" content={title} />
  <meta property="og:description" content={description} />
  <meta property="og:url" content={`${ORIGIN}${page.url.pathname}`} />
  <meta property="og:image" content={OG_IMAGE} />
  <meta property="og:image:width" content="1200" />
  <meta property="og:image:height" content="675" />
  <meta
    property="og:image:alt"
    content="The start page of the site, with the five-state busy beaver champion drawn as a space-time diagram"
  />
  <meta name="twitter:card" content="summary_large_image" />
  <meta name="theme-color" content="#ffffff" media="(prefers-color-scheme: light)" />
  <meta name="theme-color" content="#0e0e0e" media="(prefers-color-scheme: dark)" />
</svelte:head>

<a class="skip-link" href="#main">Skip to content</a>

<header class="site-header">
  <div class="page bar">
    <a class="name" href={resolve('/')} aria-current={page.route.id === '/' ? 'page' : undefined}
      >Turing machines</a
    >
    <nav aria-label="Main">
      <ul>
        {#each links as link (link.label)}
          <li>
            <a href={link.href} aria-current={current(link)}>{link.label}</a>
          </li>
        {/each}
      </ul>
    </nav>
    <div class="tools">
      <a class="author" href="https://mooofna.github.io/">Maurice Heide</a>
      <ThemeToggle />
    </div>
  </div>
</header>

<main id="main" tabindex="-1">
  {@render children()}
</main>

<footer class="site-footer">
  <div class="page row">
    <p class="label who">
      Turing machines
      <br />
      <span class="muted">Maurice Heide, 2026</span>
    </p>
    <div class="content">
      <ul class="links">
        <li><a href="https://mooofna.github.io/">Homepage</a></li>
        <li><a href="https://github.com/mooofna/turing-machines">Code on GitHub</a></li>
        <li><a href={resolve('/about')}>About and citation</a></li>
      </ul>
      <p class="note muted">
        The machines on this site run in your browser, in a Rust engine compiled to WebAssembly. The
        site sets no cookies and makes no requests to other servers. Code licensed under MIT or
        Apache-2.0.
      </p>
    </div>
  </div>
</footer>

<style>
  .site-header {
    border-bottom: 1px solid var(--rule);
  }

  .bar {
    display: flex;
    align-items: center;
    gap: var(--space-2) clamp(1rem, 0.6rem + 1.4vw, 2rem);
    min-height: 3.75rem;
  }

  .name {
    margin-right: auto;
    font-weight: 650;
    text-decoration: none;
    white-space: nowrap;
  }

  ul {
    display: flex;
    align-items: center;
    gap: clamp(1rem, 0.6rem + 1.4vw, 2rem);
    margin: 0;
    padding: 0;
    list-style: none;
  }

  nav a,
  .author {
    position: relative;
    display: inline-block;
    font-size: var(--text-sm);
    color: var(--text);
    text-decoration: none;
    padding-block: 0.35rem;
    white-space: nowrap;
  }

  nav a[aria-current]::after,
  nav a:hover::after {
    content: '';
    position: absolute;
    left: 0;
    right: 0;
    bottom: 0;
    height: 2px;
    background: var(--accent);
  }

  .tools {
    display: flex;
    align-items: center;
    gap: clamp(1rem, 0.6rem + 1.4vw, 2rem);
  }

  /* The author's name, separated from the site navigation by a hairline. */
  .author {
    color: var(--muted);
    padding-left: clamp(1rem, 0.6rem + 1.4vw, 2rem);
    border-left: 1px solid var(--rule);
    line-height: 1.2;
  }

  .author:hover {
    color: var(--text);
  }

  @media (max-width: 63.99rem) {
    .author {
      display: none;
    }
  }

  /* Phones: name and theme on the first line, the navigation on the second. */
  @media (max-width: 39.99rem) {
    .bar {
      display: grid;
      grid-template-columns: minmax(0, 1fr) auto;
      grid-template-areas:
        'name tools'
        'nav nav';
      row-gap: 0;
      padding-top: 0.6rem;
    }

    .name {
      grid-area: name;
    }

    .tools {
      grid-area: tools;
    }

    nav {
      grid-area: nav;
    }

    ul {
      flex-wrap: wrap;
      gap: 0.25rem 1.25rem;
      padding-block: 0.25rem 0.35rem;
    }
  }

  main:focus {
    outline: none;
  }

  .site-footer {
    margin-top: var(--space-5);
    border-top: 1px solid var(--rule-strong);
    padding-block: var(--space-3) var(--space-5);
    font-size: var(--text-sm);
  }

  .who {
    margin: 0 0 var(--space-2);
    font-weight: 650;
  }

  .who span {
    font-weight: 400;
  }

  .note {
    max-width: var(--measure);
    margin-top: var(--space-2);
  }
</style>
