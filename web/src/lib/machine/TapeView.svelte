<!--
  The tapes around the heads. Each tape is a row of cells centred on its head; when the head
  moves the row slides under it (only at low speeds and without reduced motion). The window
  radius follows the available width and is reported through `onradius`.
-->
<script lang="ts">
  import type { TapeWindowDto } from '#lib/engine/pkg/turing_wasm.js';

  interface Props {
    tapes: TapeWindowDto[] | null;
    /** The current state, shown above the first head. */
    current?: string | null;
    blank?: string;
    /** Slide duration in milliseconds; 0 disables the animation. */
    slide?: number;
    /** Cells needed on either side of the head to fill the width. */
    onradius?: (radius: number) => void;
    /** Number of tapes to reserve space for while there is no snapshot. */
    rows?: number;
  }

  let { tapes, current = null, blank = '␣', slide = 0, onradius, rows = 1 }: Props = $props();

  let width = $state(0);
  let cellPx = $state(28);
  let strips: HTMLElement[] = $state([]);
  let previous: number[] = [];

  const reduced =
    typeof window !== 'undefined' && window.matchMedia('(prefers-reduced-motion: reduce)').matches;

  const radius = $derived(width > 0 ? Math.ceil(width / cellPx / 2) + 2 : 0);
  /** Cells on either side of the head that are fully visible; the others stay empty. */
  const visible = $derived(width > 0 ? Math.floor((width / cellPx - 1) / 2) : Infinity);

  $effect(() => {
    if (radius > 0) onradius?.(radius);
  });

  // Slide each strip from where its old head was to the centre.
  $effect(() => {
    const heads = (tapes ?? []).map((t) => t.head);
    const before = previous;
    previous = heads;
    if (!slide || reduced) return;
    heads.forEach((head, i) => {
      const el = strips[i];
      const old = before[i];
      if (!el || old === undefined) return;
      const d = head - old;
      if (d === 0 || Math.abs(d) > 2) return;
      el.style.transition = 'none';
      el.style.transform = `translateX(${d * cellPx}px)`;
      void el.offsetWidth;
      el.style.transition = `transform ${slide}ms cubic-bezier(0.25, 0.7, 0.3, 1)`;
      el.style.transform = '';
    });
  });

  /** The blank ␣ is an empty cell; a written blank such as 0 stays, in grey. */
  function shown(g: string) {
    return g === blank && (g === '␣' || g === '_') ? '' : g;
  }

  function describe(t: TapeWindowDto, i: number) {
    const read = t.cells[t.head - t.start] ?? blank;
    const name = (tapes?.length ?? 1) > 1 ? `Tape ${i + 1}` : 'Tape';
    return `${name}, head at position ${t.head} reading ${read === '␣' ? 'blank' : read}`;
  }

  const count = $derived(tapes?.length ?? rows);
</script>

<div class="tapes" class:multi={count > 1} bind:clientWidth={width}>
  <div class="probe" bind:clientWidth={cellPx} aria-hidden="true"></div>
  <div class="state" aria-hidden="true">
    <span class="mono">{current ?? ''}</span>
  </div>
  {#each { length: count } as _, i (i)}
    {@const tape = tapes?.[i]}
    <div class="tape" role="img" aria-label={tape ? describe(tape, i) : 'Tape'}>
      {#if count > 1}<span class="index mono" aria-hidden="true">{i + 1}</span>{/if}
      {#if tape}
        <div
          class="strip"
          bind:this={strips[i]}
          style:left={`calc(50% - ${tape.head - tape.start + 0.5} * var(--cell))`}
        >
          {#each tape.cells as glyph, j (tape.start + j)}
            {@const pos = tape.start + j}
            {@const inside = Math.abs(pos - tape.head) <= visible}
            <span class="cell mono" class:blank={glyph === blank}>
              {inside ? shown(glyph) : ''}
              {#if inside && pos % 5 === 0}<span class="tick">{pos}</span>{/if}
            </span>
          {/each}
        </div>
      {/if}
      <span class="head" aria-hidden="true"></span>
    </div>
  {/each}
</div>

<style>
  .tapes {
    position: relative;
    overflow: hidden;
    padding-top: 1.6rem;
    user-select: none;
  }

  .probe {
    position: absolute;
    width: var(--cell);
    height: 0;
    visibility: hidden;
  }

  .state {
    position: absolute;
    top: 0;
    left: 50%;
    transform: translateX(-50%);
    color: var(--accent);
    font-size: var(--text-sm);
    font-weight: 500;
    line-height: 1.4rem;
    white-space: nowrap;
  }

  .state .mono {
    font-size: 1em;
  }

  .tape {
    position: relative;
    height: calc(var(--cell) + 1.25rem);
  }

  .multi .tape + .tape {
    margin-top: 0.25rem;
  }

  .index {
    position: absolute;
    left: 0;
    top: 0;
    z-index: 2;
    display: grid;
    place-items: center;
    width: 1.5rem;
    height: var(--cell);
    background: var(--bg);
    color: var(--muted);
    font-size: var(--text-xs);
    border-right: 1px solid var(--rule);
  }

  .strip {
    position: absolute;
    top: 0;
    display: flex;
    border-block: 1px solid var(--rule-strong);
    will-change: transform;
  }

  .cell {
    position: relative;
    display: grid;
    place-items: center;
    width: var(--cell);
    height: calc(var(--cell) - 2px);
    border-right: 1px solid var(--rule);
    font-size: 1rem;
    line-height: 1;
    color: var(--text);
  }

  .cell.blank {
    color: var(--muted);
  }

  .cell:first-child {
    border-left: 1px solid var(--rule);
  }

  .tick {
    position: absolute;
    top: calc(100% + 0.3rem);
    left: 50%;
    transform: translateX(-50%);
    font-size: 0.6875rem;
    color: var(--muted);
    line-height: 1;
  }

  .head {
    position: absolute;
    top: -1px;
    left: calc(50% - var(--cell) / 2 - 1px);
    width: calc(var(--cell) + 2px);
    height: calc(var(--cell) + 0px);
    border: 2px solid var(--head);
    pointer-events: none;
  }
</style>
