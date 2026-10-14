<!--
  A menu of actions in a popover: arrow keys move, Enter or Space choose, Escape closes.
-->
<script lang="ts" module>
  export interface MenuItem {
    label: string;
    onselect: () => void;
    disabled?: boolean;
    /** Shown as a checkbox item when defined. */
    checked?: boolean;
    /** A short hint on the right, such as a key. */
    hint?: string;
    /** A second line in muted text. */
    detail?: string;
  }
</script>

<script lang="ts">
  import Popover from './Popover.svelte';

  interface Props {
    open: boolean;
    anchor: HTMLElement | DOMRect | null;
    label: string;
    items: MenuItem[];
    onclose: () => void;
  }

  let { open, anchor, label, items, onclose }: Props = $props();
  let panel: HTMLElement | null = $state(null);

  function entries(): HTMLElement[] {
    return Array.from(
      panel?.querySelectorAll<HTMLElement>('[role="menuitem"], [role="menuitemcheckbox"]') ?? []
    );
  }

  function keydown(event: KeyboardEvent) {
    const all = entries();
    const at = all.indexOf(document.activeElement as HTMLElement);
    const move = (i: number) => {
      event.preventDefault();
      all[(i + all.length) % all.length]?.focus();
    };
    if (event.key === 'ArrowDown') move(at + 1);
    else if (event.key === 'ArrowUp') move(at - 1);
    else if (event.key === 'Home') move(0);
    else if (event.key === 'End') move(all.length - 1);
    else if (event.key === 'Tab') onclose();
  }

  function choose(item: MenuItem) {
    if (item.disabled) return;
    // Props read the parent's state, which closing clears: take the anchor first.
    const target = anchor;
    onclose();
    if (target instanceof HTMLElement) target.focus();
    item.onselect();
  }
</script>

<Popover {open} {anchor} {label} {onclose} role="menu" width={176} bind:panel>
  <!-- svelte-ignore a11y_no_static_element_interactions -->
  <div class="items" onkeydown={keydown}>
    {#each items as item (item.label)}
      <button
        type="button"
        role={item.checked === undefined ? 'menuitem' : 'menuitemcheckbox'}
        aria-checked={item.checked === undefined ? undefined : item.checked}
        aria-disabled={item.disabled ? 'true' : undefined}
        tabindex="-1"
        class="item"
        onclick={() => choose(item)}
      >
        <span class="mark" aria-hidden="true">{item.checked ? '✓' : ''}</span>
        <span class="text"
          >{item.label}{#if item.detail}<span class="detail">{item.detail}</span>{/if}</span
        >
        {#if item.hint}<span class="hint mono" aria-hidden="true">{item.hint}</span>{/if}
      </button>
    {/each}
  </div>
</Popover>

<style>
  .items {
    display: grid;
    padding-block: 0.25rem;
  }

  .item {
    display: grid;
    grid-template-columns: 1rem 1fr auto;
    align-items: center;
    gap: 0.4rem;
    width: 100%;
    padding: 0.45rem 0.75rem 0.45rem 0.5rem;
    border: 0;
    background: none;
    text-align: left;
    font-size: var(--text-sm);
    cursor: pointer;
    white-space: nowrap;
  }

  .item:hover,
  .item:focus-visible {
    background: color-mix(in srgb, var(--text) 7%, transparent);
  }

  .item:focus-visible {
    outline: 2px solid var(--accent);
    outline-offset: -2px;
  }

  .item[aria-disabled='true'] {
    color: var(--muted);
    cursor: default;
  }

  .mark {
    align-self: start;
    font-size: var(--text-xs);
    line-height: 1.55rem;
  }

  .detail {
    display: block;
    color: var(--muted);
    font-size: var(--text-xs);
    white-space: normal;
  }

  .hint {
    color: var(--muted);
    font-size: var(--text-xs);
  }
</style>
