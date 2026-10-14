<script lang="ts">
  import { onMount } from 'svelte';

  type Theme = 'light' | 'dark';
  let theme = $state<Theme>('light');

  function systemTheme(): Theme {
    return window.matchMedia('(prefers-color-scheme: dark)').matches ? 'dark' : 'light';
  }

  onMount(() => {
    const stored = document.documentElement.dataset.theme as Theme | undefined;
    theme = stored ?? systemTheme();
    const media = window.matchMedia('(prefers-color-scheme: dark)');
    const sync = () => {
      if (!document.documentElement.dataset.theme) theme = systemTheme();
    };
    media.addEventListener('change', sync);
    return () => media.removeEventListener('change', sync);
  });

  function toggle() {
    theme = theme === 'dark' ? 'light' : 'dark';
    document.documentElement.dataset.theme = theme;
    try {
      localStorage.setItem('theme', theme);
    } catch {
      // Private mode or blocked storage: the choice lasts for this page only.
    }
    window.dispatchEvent(new CustomEvent('themechange', { detail: theme }));
  }

  const next = $derived(theme === 'dark' ? 'light' : 'dark');
</script>

<button class="theme" type="button" onclick={toggle} aria-label={`Switch to the ${next} theme`}>
  {next === 'dark' ? 'Dark' : 'Light'}
</button>

<style>
  .theme {
    border: 0;
    background: none;
    color: var(--muted);
    cursor: pointer;
    padding: 0.35rem 0;
    font-size: var(--text-sm);
  }

  .theme:hover {
    color: var(--text);
  }
</style>
