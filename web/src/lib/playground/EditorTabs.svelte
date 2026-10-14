<!--
  Three synchronised views of the machine: the state diagram, the transition table and the text.
  A tab is mounted the first time it opens and then kept, so the text editor keeps its history.
-->
<script lang="ts" module>
  export type Tab = 'diagram' | 'table' | 'text';
</script>

<script lang="ts">
  import type { KindDto } from '#lib/engine/pkg/turing_wasm.js';
  import SourceEditor from './SourceEditor.svelte';
  import StateDiagram from './StateDiagram.svelte';
  import type { Playground } from './state.svelte';
  import TransitionList from './TransitionList.svelte';
  import TransitionTable from './TransitionTable.svelte';

  interface Props {
    pg: Playground;
    tab: Tab;
    ontab: (tab: Tab) => void;
    /** Small screens: the diagram is view-only. */
    narrow: boolean;
  }

  let { pg, tab, ontab, narrow }: Props = $props();
  const id = $props.id();

  const tabs: { id: Tab; label: string }[] = [
    { id: 'diagram', label: 'Diagram' },
    { id: 'table', label: 'Table' },
    { id: 'text', label: 'Text' }
  ];

  const kinds: { kind: KindDto; label: string }[] = [
    { kind: 'acceptor', label: 'Acceptor' },
    { kind: 'transducer', label: 'Transducer' },
    { kind: 'enumerator', label: 'Enumerator' },
    { kind: 'beaver', label: 'Busy beaver' }
  ];

  // svelte-ignore state_referenced_locally
  let opened = $state<Record<Tab, boolean>>({
    diagram: false,
    table: false,
    text: false,
    [tab]: true
  });
  $effect(() => {
    if (!opened[tab]) opened = { ...opened, [tab]: true };
  });

  function keydown(event: KeyboardEvent) {
    const i = tabs.findIndex((t) => t.id === tab);
    let next = -1;
    if (event.key === 'ArrowRight') next = (i + 1) % tabs.length;
    else if (event.key === 'ArrowLeft') next = (i - 1 + tabs.length) % tabs.length;
    else if (event.key === 'Home') next = 0;
    else if (event.key === 'End') next = tabs.length - 1;
    if (next < 0) return;
    event.preventDefault();
    const target = tabs[next]!;
    ontab(target.id);
    document.getElementById(`${id}-tab-${target.id}`)?.focus();
  }

  const spec = $derived(pg.spec);
  const blocked = $derived(spec === null);
</script>

<div class="toolbar">
  <div
    class="tablist"
    role="tablist"
    aria-label="Views of the machine"
    tabindex="-1"
    onkeydown={keydown}
  >
    {#each tabs as t (t.id)}
      <button
        type="button"
        role="tab"
        id={`${id}-tab-${t.id}`}
        aria-selected={tab === t.id}
        aria-controls={`${id}-panel-${t.id}`}
        tabindex={tab === t.id ? 0 : -1}
        class="tab"
        onclick={() => ontab(t.id)}>{t.label}</button
      >
    {/each}
  </div>
  <div class="tools">
    {#if spec}
      <label class="kind">
        <span>Kind</span>
        <select
          value={spec.kind}
          onchange={(e) => pg.setKind(e.currentTarget.value as KindDto)}
          disabled={pg.stale}
        >
          {#each kinds as k (k.kind)}<option value={k.kind}>{k.label}</option>{/each}
        </select>
      </label>
    {/if}
    <button
      type="button"
      class="tool"
      onclick={() => pg.undo()}
      disabled={!pg.canUndo}
      aria-keyshortcuts="Control+Z Meta+Z">Undo</button
    >
    <button
      type="button"
      class="tool"
      onclick={() => pg.redo()}
      disabled={!pg.canRedo}
      aria-keyshortcuts="Control+Shift+Z Meta+Shift+Z">Redo</button
    >
  </div>
</div>

<div class="notice" aria-live="polite">
  {#if pg.notice}
    <span>{pg.notice.text}</span>
    {#if pg.notice.undo && pg.canUndo}
      <button type="button" class="link" onclick={() => pg.undo()}>Undo</button>
    {/if}
  {:else if pg.stale && tab !== 'text'}
    <span>The text has errors, so this shows the last valid machine.</span>
    <button type="button" class="link" onclick={() => ontab('text')}>Show the errors</button>
    <button type="button" class="link" onclick={() => pg.discardText()}
      >Discard the text changes</button
    >
  {/if}
</div>

{#each tabs as t (t.id)}
  <div
    role="tabpanel"
    id={`${id}-panel-${t.id}`}
    aria-labelledby={`${id}-tab-${t.id}`}
    class="panel"
    hidden={tab !== t.id}
  >
    {#if opened[t.id]}
      {#if t.id === 'text'}
        <SourceEditor {pg} />
      {:else if blocked}
        <div class="blocked">
          <p>The description has errors, so there is no machine to show.</p>
          <button type="button" class="button" onclick={() => ontab('text')}>Open the text</button>
        </div>
      {:else}
        <div class="view" class:stale={pg.stale} inert={pg.stale}>
          {#if t.id === 'diagram'}
            <StateDiagram {pg} editable={!narrow} visible={tab === 'diagram'} />
          {:else if spec && spec.tapes > 1}
            <TransitionList {pg} />
          {:else}
            <TransitionTable {pg} />
          {/if}
        </div>
      {/if}
    {/if}
  </div>
{/each}

<style>
  .toolbar {
    display: flex;
    flex-wrap: wrap;
    align-items: end;
    justify-content: space-between;
    gap: 0.5rem var(--space-2);
    border-bottom: 1px solid var(--rule-strong);
  }

  .tablist {
    display: flex;
  }

  .tab {
    position: relative;
    padding: 0.55rem 0.9rem 0.6rem;
    border: 0;
    background: none;
    color: var(--muted);
    font-size: var(--text-sm);
    cursor: pointer;
  }

  .tab:first-child {
    padding-left: 0;
  }

  .tab:hover {
    color: var(--text);
  }

  .tab[aria-selected='true'] {
    color: var(--text);
    font-weight: 600;
  }

  .tab[aria-selected='true']::after {
    content: '';
    position: absolute;
    left: 0.9rem;
    right: 0.9rem;
    bottom: -1px;
    height: 2px;
    background: var(--text);
  }

  .tab:first-child[aria-selected='true']::after {
    left: 0;
  }

  .tab:focus-visible {
    outline: 2px solid var(--accent);
    outline-offset: -2px;
  }

  .tools {
    display: flex;
    align-items: center;
    gap: 0.25rem 0.75rem;
    padding-bottom: 0.4rem;
    font-size: var(--text-sm);
  }

  .kind {
    display: flex;
    align-items: center;
    gap: 0.4rem;
    color: var(--muted);
  }

  select {
    height: 2rem;
    padding: 0 0.4rem;
    border: 1px solid var(--rule);
    border-radius: 0;
    background: var(--field);
    color: var(--text);
    font-size: var(--text-sm);
  }

  select:focus-visible {
    outline: 2px solid var(--accent);
    outline-offset: 1px;
  }

  .tool {
    padding: 0.3rem 0.1rem;
    border: 0;
    background: none;
    color: var(--text);
    font-size: var(--text-sm);
    cursor: pointer;
  }

  .tool:hover:not(:disabled) {
    color: var(--accent);
  }

  .tool:disabled {
    color: var(--muted);
    opacity: 0.6;
    cursor: default;
  }

  .notice {
    display: flex;
    flex-wrap: wrap;
    align-items: baseline;
    gap: 0.25rem 0.9rem;
    min-height: 2.5rem;
    padding-block: 0.6rem;
    font-size: var(--text-sm);
  }

  .link {
    border: 0;
    background: none;
    padding: 0;
    color: var(--text);
    font-size: var(--text-sm);
    cursor: pointer;
    text-decoration: underline;
    text-decoration-color: color-mix(in srgb, currentColor 35%, transparent);
    text-underline-offset: 0.18em;
  }

  .link:hover {
    text-decoration-color: var(--accent);
    text-decoration-thickness: 2px;
  }

  .view.stale {
    opacity: 0.45;
    filter: grayscale(1);
  }

  .blocked {
    display: grid;
    justify-items: start;
    gap: 0.75rem;
    padding: var(--space-3) 0;
    font-size: var(--text-sm);
  }
</style>
