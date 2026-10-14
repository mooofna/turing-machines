<!--
  The machine bar: the machine's name (editable), a picker of example machines, New and Share.
-->
<script lang="ts">
  import Menu, { type MenuItem } from './Menu.svelte';
  import Popover from './Popover.svelte';
  import { samples, type Sample } from './samples';
  import type { Playground } from './state.svelte';

  interface Props {
    pg: Playground;
    onpick: (sample: Sample) => void;
    onshare: () => Promise<{ url: string; copied: boolean } | null>;
  }

  let { pg, onpick, onshare }: Props = $props();
  const id = $props.id();

  let picker: HTMLButtonElement | null = $state(null);
  let shareButton: HTMLButtonElement | null = $state(null);
  let picking = $state(false);
  let shared = $state<{ url: string; copied: boolean } | null>(null);
  let copiedTimer: ReturnType<typeof setTimeout> | undefined;

  const items = $derived<MenuItem[]>(
    samples.map((s) => ({
      label: s.name,
      detail: s.summary,
      checked: pg.galleryId === s.id,
      onselect: () => onpick(s)
    }))
  );

  function commitName(input: HTMLInputElement) {
    const name = input.value.trim().replace(/"/g, "'");
    if (pg.spec && name !== pg.spec.name) pg.setName(name);
    input.value = name;
  }

  async function share() {
    clearTimeout(copiedTimer);
    shared = await onshare();
    if (shared?.copied) copiedTimer = setTimeout(() => (shared = null), 2500);
  }
</script>

<div class="bar">
  <div class="name">
    <label class="visually-hidden" for={`${id}-name`}>Machine name</label>
    <input
      id={`${id}-name`}
      class="name-input"
      value={pg.spec?.name ?? ''}
      placeholder="Untitled machine"
      autocomplete="off"
      spellcheck="false"
      disabled={!pg.spec}
      onkeydown={(e) => {
        if (e.key === 'Enter') {
          e.preventDefault();
          e.currentTarget.blur();
        } else if (e.key === 'Escape') {
          e.currentTarget.value = pg.spec?.name ?? '';
          e.currentTarget.blur();
        }
      }}
      onblur={(e) => commitName(e.currentTarget)}
    />
  </div>
  <div class="actions">
    <button
      type="button"
      class="button"
      bind:this={picker}
      aria-haspopup="menu"
      aria-expanded={picking}
      onclick={() => (picking = !picking)}>Machines</button
    >
    <button type="button" class="button" onclick={() => pg.newMachine()}>New</button>
    <button type="button" class="button" bind:this={shareButton} onclick={share} disabled={!pg.spec}
      >{shared?.copied ? 'Link copied' : 'Share'}</button
    >
  </div>
</div>
<p class="visually-hidden" aria-live="polite">{shared?.copied ? 'Link copied' : ''}</p>

<Menu
  open={picking}
  anchor={picker}
  label="Example machines"
  {items}
  onclose={() => (picking = false)}
/>

{#if shared && !shared.copied}
  <Popover
    open={true}
    anchor={shareButton}
    label="Share link"
    onclose={() => (shared = null)}
    width={320}
  >
    <div class="share">
      <label for={`${id}-url`}>Copy this link to share the machine and its input.</label>
      <input
        id={`${id}-url`}
        class="mono"
        readonly
        value={shared.url}
        data-autofocus
        onfocus={(e) => e.currentTarget.select()}
      />
    </div>
  </Popover>
{/if}

<style>
  .bar {
    display: flex;
    flex-wrap: wrap;
    align-items: center;
    justify-content: space-between;
    gap: var(--space-2);
  }

  .name {
    flex: 1 1 14rem;
    min-width: 0;
  }

  .name-input {
    width: 100%;
    max-width: 28rem;
    padding: 0.2rem 0;
    border: 0;
    border-bottom: 1px solid transparent;
    background: transparent;
    font-size: var(--text-lg);
    font-weight: 650;
    letter-spacing: -0.01em;
    color: var(--text);
  }

  .name-input:hover:not(:disabled) {
    border-bottom-color: var(--rule);
  }

  .name-input:focus-visible {
    outline: none;
    border-bottom: 1px solid var(--accent);
  }

  .name-input::placeholder {
    color: var(--muted);
    opacity: 1;
  }

  .actions {
    display: flex;
    gap: 0.5rem;
  }

  .share {
    display: grid;
    gap: 0.5rem;
    padding: 0.75rem;
  }

  .share label {
    font-size: var(--text-xs);
    color: var(--muted);
  }

  .share input {
    height: 2.25rem;
    padding: 0 0.6rem;
    border: 1px solid var(--rule-strong);
    background: var(--field);
    font-size: var(--text-xs);
  }
</style>
