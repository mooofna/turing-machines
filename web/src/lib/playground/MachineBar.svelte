<!--
  The machine bar: the machine's name (editable), a picker of example machines, New and Share.
-->
<script lang="ts">
  import { groups, machines, type GalleryMachine } from '#lib/gallery.ts';
  import Menu, { type MenuItem } from './Menu.svelte';
  import Popover from './Popover.svelte';
  import type { Playground } from './state.svelte';

  interface Props {
    pg: Playground;
    onpick: (machine: GalleryMachine) => void;
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
    groups.flatMap((group) =>
      machines
        .filter((m) => m.group === group.id)
        .map((m) => ({
          label: m.title,
          detail: m.description,
          section: group.title,
          checked: pg.galleryId === m.id,
          onselect: () => onpick(m)
        }))
    )
  );
  const entry = $derived(pg.galleryId ? machines.find((m) => m.id === pg.galleryId) : undefined);
  /** Busy beaver machines also show their standard format, as on bbchallenge.org. */
  const standard = $derived(
    !entry && pg.spec?.kind === 'beaver' && pg.ready ? pg.services.toStandard(pg.spec) : undefined
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
    {#if entry}
      <p class="about">{entry.description} {entry.provenance}.</p>
    {:else if standard}
      <p class="about">
        <span class="visually-hidden">Standard format </span><span class="mono">{standard}</span>
      </p>
    {/if}
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
  label="Machines from the gallery"
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
    align-items: flex-start;
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

  .about {
    margin: 0.2rem 0 0;
    max-width: var(--measure);
    font-size: var(--text-sm);
    color: var(--muted);
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
