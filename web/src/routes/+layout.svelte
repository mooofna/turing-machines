<script lang="ts">
  import '../app.css';
  import { resolve } from '$app/paths';
  import { page } from '$app/state';
  import ThemeToggle from '#lib/components/ThemeToggle.svelte';

  let { children } = $props();

  const links = [
    { href: resolve('/machine'), prefix: '/machine', label: 'Chapters' },
    { href: resolve('/playground'), prefix: '/playground', label: 'Playground' },
    { href: resolve('/gallery'), prefix: '/gallery', label: 'Gallery' },
    { href: resolve('/about'), prefix: '/about', label: 'About' }
  ];

  const current = (prefix: string) => page.route.id?.startsWith(prefix) ?? false;
</script>

<svelte:head>
  <title>Turing machines</title>
  <meta
    name="description"
    content="Turing machines from Turing's definition of 1936 to the open problems of the busy beaver function, simulated in the browser with Rust and WebAssembly."
  />
</svelte:head>

<a class="skip-link" href="#main">Skip to content</a>

<header class="site-header">
  <div class="page bar">
    <a class="name" href={resolve('/')}>Turing machines</a>
    <nav aria-label="Main">
      <ul>
        {#each links as link (link.href)}
          <li>
            <a href={link.href} aria-current={current(link.prefix) ? 'page' : undefined}
              >{link.label}</a
            >
          </li>
        {/each}
        <li><ThemeToggle /></li>
      </ul>
    </nav>
  </div>
</header>

<main id="main">
  {@render children()}
</main>

<footer class="site-footer">
  <div class="page foot">
    <p>
      By <a href="https://mooofna.github.io/">Maurice Heide</a>. Code on
      <a href="https://github.com/mooofna/turing-machines">GitHub</a>, MIT or Apache-2.0.
    </p>
    <p class="muted">Everything on this site is computed in your browser.</p>
  </div>
</footer>

<style>
  .site-header {
    border-bottom: 1px solid var(--rule);
  }

  .bar {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: var(--space-2);
    min-height: 3.75rem;
  }

  .name {
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

  nav a {
    position: relative;
    font-size: var(--text-sm);
    color: var(--text);
    text-decoration: none;
    padding-block: 0.35rem;
  }

  nav a[aria-current='page']::after,
  nav a:hover::after {
    content: '';
    position: absolute;
    left: 0;
    right: 0;
    bottom: 0;
    height: 2px;
    background: var(--accent);
  }

  .site-footer {
    border-top: 1px solid var(--rule);
    margin-top: var(--space-5);
  }

  .foot {
    display: flex;
    flex-wrap: wrap;
    justify-content: space-between;
    gap: var(--space-1) var(--space-4);
    padding-block: var(--space-3) var(--space-4);
    font-size: var(--text-xs);
  }

  .foot p {
    margin: 0;
  }
</style>
