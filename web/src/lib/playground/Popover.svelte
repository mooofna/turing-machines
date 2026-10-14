<!--
  A small floating panel in the top layer, anchored below (or above) an element or a rectangle.
  Escape and a pointer press outside close it; focus moves into it when it opens and back to
  the anchor element when it closes.
-->
<script lang="ts">
  import type { Snippet } from 'svelte';
  import { tick } from 'svelte';

  interface Props {
    open: boolean;
    anchor: HTMLElement | DOMRect | null;
    onclose: () => void;
    label: string;
    role?: 'dialog' | 'menu';
    /** Minimum width in pixels. */
    width?: number;
    /** Below (or above) the anchor, or beside it, which keeps a diagram node visible. */
    placement?: 'below' | 'side';
    children: Snippet;
    panel?: HTMLElement | null;
  }

  let {
    open,
    anchor,
    onclose,
    label,
    role = 'dialog',
    width = 0,
    placement = 'below',
    children,
    panel = $bindable(null)
  }: Props = $props();

  let left = $state(0);
  let top = $state(0);

  function rect(): DOMRect | null {
    if (!anchor) return null;
    return anchor instanceof HTMLElement ? anchor.getBoundingClientRect() : anchor;
  }

  function place() {
    const r = rect();
    if (!r || !panel) return;
    const margin = 8;
    const w = panel.offsetWidth;
    const h = panel.offsetHeight;
    const vw = document.documentElement.clientWidth;
    const vh = window.innerHeight;
    const fits = (y: number) => y >= margin && y + h + margin <= vh;
    if (placement === 'side') {
      const right = r.right + 8;
      left = right + w + margin <= vw ? right : Math.max(margin, r.left - w - 8);
      const centred = r.top + r.height / 2 - h / 2;
      top = Math.max(margin, Math.min(centred, vh - h - margin));
      return;
    }
    // Anchors on the right side of the page get panels that end where they end.
    const preferred = r.left + r.width / 2 > vw * 0.6 ? r.right - w : r.left;
    left = Math.max(margin, Math.min(preferred, vw - w - margin));
    const below = r.bottom + 4;
    const above = r.top - h - 4;
    top = fits(below) ? below : fits(above) ? above : Math.max(margin, vh - h - margin);
  }

  $effect(() => {
    const el = panel;
    if (!el) return;
    if (open) {
      if (!el.matches(':popover-open')) el.showPopover();
      place();
      void tick().then(() => {
        place();
        const target =
          el.querySelector<HTMLElement>('[data-autofocus]') ??
          el.querySelector<HTMLElement>(
            '[role="menuitem"]:not([aria-disabled="true"]), [role="menuitemcheckbox"], input, select, button:not([disabled])'
          );
        target?.focus();
      });
    } else if (el.matches(':popover-open')) {
      el.hidePopover();
    }
  });

  function close(returnFocus = true) {
    // Props read the parent's state, which closing clears: take the anchor first.
    const target = anchor;
    onclose();
    if (returnFocus && target instanceof HTMLElement) target.focus();
  }

  function pointerdown(event: PointerEvent) {
    if (!open || !panel) return;
    const target = event.target as Node;
    if (panel.contains(target)) return;
    const a = anchor;
    if (a instanceof HTMLElement && a.contains(target)) return;
    close(false);
  }

  function keydown(event: KeyboardEvent) {
    if (event.key === 'Escape') {
      event.preventDefault();
      event.stopPropagation();
      close();
    }
  }
</script>

<svelte:window
  onpointerdowncapture={pointerdown}
  onresize={() => open && place()}
  onscrollcapture={() => open && place()}
/>

<div
  bind:this={panel}
  popover="manual"
  class="popover"
  {role}
  aria-label={label}
  style:left={`${left}px`}
  style:top={`${top}px`}
  style:min-width={width ? `${width}px` : undefined}
  onkeydown={keydown}
  tabindex="-1"
>
  {#if open}{@render children()}{/if}
</div>

<style>
  .popover {
    position: fixed;
    inset: auto;
    margin: 0;
    padding: 0;
    max-width: min(24rem, calc(100vw - 16px));
    max-height: calc(100vh - 16px);
    overflow: auto;
    border: 1px solid var(--rule-strong);
    background: var(--bg);
    color: var(--text);
    font-size: var(--text-sm);
  }

  .popover:focus {
    outline: none;
  }
</style>
