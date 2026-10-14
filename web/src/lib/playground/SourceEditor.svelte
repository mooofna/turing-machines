<!--
  The description of the machine as text. CodeMirror loads when this tab first opens; edits are
  parsed 150 ms after the last key, errors appear in the text and in a list below it.
-->
<script lang="ts">
  import { onMount } from 'svelte';
  import type { SourceView } from './codemirror';
  import type { Playground } from './state.svelte';

  interface Props {
    pg: Playground;
  }

  let { pg }: Props = $props();
  const id = $props.id();

  let host: HTMLElement | null = $state(null);
  let view: SourceView | null = $state(null);
  let timer: ReturnType<typeof setTimeout> | undefined;
  let pending = $state(false);

  function changed(text: string) {
    pending = true;
    clearTimeout(timer);
    timer = setTimeout(() => {
      pending = false;
      pg.setSource(text);
    }, 150);
  }

  onMount(() => {
    let alive = true;
    void import('./codemirror').then(({ createSourceView }) => {
      if (!alive || !host) return;
      view = createSourceView(host, pg.source, changed, 'Machine description');
    });
    return () => {
      alive = false;
      clearTimeout(timer);
      if (pending && view) pg.setSource(view.text);
      view?.destroy();
      view = null;
    };
  });

  // Text that changed elsewhere (table, diagram, undo, a new machine) replaces the editor's.
  $effect(() => {
    const source = pg.source;
    if (view && !pending && view.text !== source) view.setText(source);
  });

  $effect(() => {
    const diagnostics = pg.diagnostics;
    if (view && !pending && view.text === pg.source) view.setDiagnostics(diagnostics);
  });

  $effect(() => {
    const index = pg.snapshot?.nextTransition;
    const span = index === undefined ? undefined : pg.spec?.transitions[index]?.span;
    const valid = !pg.hasErrors && !pending && view?.text === pg.source;
    view?.setNext(valid && span ? span.from : null);
  });

  const problems = $derived(pg.diagnostics);
</script>

<div class="source">
  <div class="host" bind:this={host}></div>
  {#if !view}
    <pre class="placeholder mono" aria-hidden="true">{pg.source}</pre>
  {/if}
  <div id={`${id}-problems`} class="problems" aria-live="polite">
    {#if problems.length > 0}
      <ul>
        {#each problems as d, i (i)}
          <li>
            <span class="where mono"
              >{d.line !== undefined ? `Line ${d.line}${d.column ? `:${d.column}` : ''}` : ''}</span
            >
            <span class="kind">{d.severity === 'error' ? 'Error' : 'Warning'} {d.code}</span>
            <span class="text">{d.message}{d.help ? `. ${d.help}` : ''}</span>
          </li>
        {/each}
      </ul>
    {:else}
      <p class="muted">
        Transitions are written δ(q, a) = (q', b, D) with D one of L, R or N. Comments start with
        //.
      </p>
    {/if}
  </div>
</div>

<style>
  .placeholder {
    margin: 0;
    min-height: 22rem;
    max-height: 36rem;
    overflow: auto;
    padding: 0.6rem 0.75rem 0.6rem 3.75rem;
    border: 1px solid var(--rule-strong);
    background: var(--field);
    font-size: 0.9375rem;
    line-height: 1.65;
    white-space: pre-wrap;
  }

  .problems {
    margin-top: 0.6rem;
    font-size: var(--text-sm);
  }

  ul {
    margin: 0;
    padding: 0;
    list-style: none;
    display: grid;
    gap: 0.35rem;
  }

  li {
    display: grid;
    grid-template-columns: 5.5rem 7.5rem 1fr;
    gap: 0.75rem;
  }

  .where {
    color: var(--muted);
  }

  .kind {
    font-weight: 600;
  }

  @media (max-width: 47.99rem) {
    li {
      grid-template-columns: auto 1fr;
    }

    .text {
      grid-column: 1 / -1;
    }
  }
</style>
