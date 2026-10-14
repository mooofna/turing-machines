<!--
  The line chart of Fig. 1.3: steps of M*2 against n, for the one-tape machine (ink) and the
  two-tape machine (grey), on one linear axis. Hairline grid, mono tick labels, both lines
  labelled at their right end. The selected n is a vertical accent hairline with a dot on each
  line; pointing at the chart shows a ghost line at the nearest n, and clicking selects it.
-->
<script lang="ts">
  import { count } from '#lib/machine/describe.ts';

  interface Props {
    /** Steps of the one-tape machine for n = 0, 1, ...; null while measuring. */
    one: number[] | null;
    /** Steps of the two-tape machine. */
    two: number[] | null;
    /** The largest n. */
    last: number;
    selected: number;
    onselect: (n: number) => void;
  }

  let { one, two, last, selected, onselect }: Props = $props();

  let width = $state(0);
  let hover = $state<number | null>(null);

  const narrow = $derived(width > 0 && width < 520);
  const height = $derived(narrow ? 200 : 236);
  const pad = $derived({ top: 26, right: narrow ? 74 : 92, bottom: 30, left: 44 });
  const plotW = $derived(Math.max(1, width - pad.left - pad.right));
  const plotH = $derived(height - pad.top - pad.bottom);

  const max = $derived(Math.max(1, ...(one ?? []), ...(two ?? [])));
  /** Clean tick spacing: 1, 2 or 5 times a power of ten, about four ticks. */
  const tick = $derived.by(() => {
    const raw = max / 4;
    const p = 10 ** Math.floor(Math.log10(raw));
    return [1, 2, 5, 10].map((k) => k * p).find((s) => s >= raw) ?? p * 10;
  });
  const top = $derived(Math.ceil(max / tick) * tick);
  const yTicks = $derived(Array.from({ length: Math.round(top / tick) + 1 }, (_, i) => i * tick));
  const xTicks = $derived(Array.from({ length: Math.floor(last / 5) + 1 }, (_, i) => i * 5));

  const x = (n: number) => pad.left + (n / last) * plotW;
  const y = (v: number) => pad.top + plotH - (v / top) * plotH;

  function path(values: number[] | null) {
    if (!values || values.length === 0) return '';
    return values
      .map((v, n) => `${n === 0 ? 'M' : 'L'}${x(n).toFixed(1)},${y(v).toFixed(1)}`)
      .join('');
  }

  /** The end labels, moved apart if they would overlap. */
  const ends = $derived.by(() => {
    if (!one || !two) return null;
    let a = y(one[last] ?? 0);
    let b = y(two[last] ?? 0);
    const gap = 30;
    if (b - a < gap) {
      const mid = (a + b) / 2;
      a = mid - gap / 2;
      b = mid + gap / 2;
    }
    return { one: a, two: b };
  });

  function nearest(event: PointerEvent): number {
    const box = (event.currentTarget as SVGElement).getBoundingClientRect();
    const px = event.clientX - box.left;
    return Math.max(0, Math.min(last, Math.round(((px - pad.left) / plotW) * last)));
  }

  const shown = $derived(hover ?? selected);
</script>

<div class="chart" bind:clientWidth={width}>
  <div class="legend" aria-hidden="true">
    <span class="n mono">n = {shown}</span>
    <span class="key"
      ><span class="swatch ink"></span>One tape
      <span class="mono">{one ? `${count(one[shown] ?? 0)} steps` : ''}</span></span
    >
    <span class="key"
      ><span class="swatch grey"></span>Two tapes
      <span class="mono">{two ? `${count(two[shown] ?? 0)} steps` : ''}</span></span
    >
  </div>
  {#if width > 0}
    <!-- The slider below selects n by keyboard; the pointer may also pick it here. -->
    <!-- svelte-ignore a11y_no_noninteractive_element_interactions, a11y_click_events_have_key_events -->
    <svg
      {width}
      {height}
      viewBox="0 0 {width} {height}"
      role="img"
      aria-label={one && two
        ? `Steps of M*2 for n = 0 to ${last}: on one tape from ${count(one[0] ?? 0)} to ${count(one[last] ?? 0)}, on two tapes from ${count(two[0] ?? 0)} to ${count(two[last] ?? 0)}.`
        : 'Steps of M*2, being measured.'}
      onpointermove={(e) => (hover = nearest(e))}
      onpointerleave={() => (hover = null)}
      onclick={(e) => onselect(nearest(e as unknown as PointerEvent))}
    >
      <!-- Grid and axes -->
      {#each yTicks as v (v)}
        <line
          class="grid"
          class:base={v === 0}
          x1={pad.left}
          x2={pad.left + plotW}
          y1={y(v)}
          y2={y(v)}
        />
        <text class="tick" x={pad.left - 8} y={y(v)} dy="0.32em" text-anchor="end">{count(v)}</text>
      {/each}
      {#each xTicks as n (n)}
        <text class="tick" x={x(n)} y={pad.top + plotH + 18} text-anchor="middle">{n}</text>
      {/each}
      <text class="axis" x={pad.left + plotW + 10} y={pad.top + plotH + 18}>n</text>
      <text class="unit" x={0} y={pad.top - 14}>steps</text>

      <!-- Hover and selection -->
      {#if hover !== null && hover !== selected}
        <line class="ghost" x1={x(hover)} x2={x(hover)} y1={pad.top} y2={pad.top + plotH} />
      {/if}
      <line class="select" x1={x(selected)} x2={x(selected)} y1={pad.top} y2={pad.top + plotH} />

      <!-- Series -->
      <path class="series grey" d={path(two)} />
      <path class="series ink" d={path(one)} />

      {#if one && two && ends}
        <circle class="dot ink" cx={x(last)} cy={y(one[last] ?? 0)} r="4" />
        <circle class="dot grey" cx={x(last)} cy={y(two[last] ?? 0)} r="4" />
        <text class="label" x={x(last) + 10} y={ends.one} dy="0.32em">One tape</text>
        <text class="label" x={x(last) + 10} y={ends.two} dy="0.32em">Two tapes</text>

        <circle class="dot active" cx={x(selected)} cy={y(one[selected] ?? 0)} r="4" />
        <circle class="dot active" cx={x(selected)} cy={y(two[selected] ?? 0)} r="4" />
      {/if}
    </svg>
  {:else}
    <div style:height="{height}px"></div>
  {/if}
</div>

<style>
  .chart {
    position: relative;
    min-width: 0;
  }

  .legend {
    display: flex;
    flex-wrap: wrap;
    align-items: baseline;
    gap: 0.25rem 1.5rem;
    margin-bottom: var(--space-1);
    font-size: var(--text-sm);
  }

  .legend .n {
    min-width: 4.5rem;
    color: var(--accent);
    font-size: var(--text-sm);
  }

  .key {
    display: inline-flex;
    align-items: baseline;
    gap: 0.45rem;
    color: var(--muted);
  }

  .key .mono {
    min-width: 6.5rem;
    color: var(--text);
    font-size: var(--text-sm);
  }

  .swatch {
    display: inline-block;
    width: 1rem;
    height: 2px;
    transform: translateY(-0.3em);
  }

  .swatch.ink {
    background: var(--text);
  }

  .swatch.grey {
    background: var(--muted);
  }

  svg {
    display: block;
    overflow: visible;
    cursor: crosshair;
    touch-action: pan-y;
  }

  .grid {
    stroke: var(--rule);
    stroke-width: 1;
    shape-rendering: crispEdges;
  }

  .grid.base {
    stroke: var(--rule-strong);
  }

  .tick,
  .axis {
    font-family: var(--font-mono);
    font-size: 11px;
    fill: var(--muted);
  }

  .axis {
    font-family: var(--font-sans);
    font-size: 12px;
    font-style: italic;
  }

  .unit {
    font-size: 11px;
    fill: var(--muted);
  }

  .series {
    fill: none;
    stroke-width: 2;
    stroke-linejoin: round;
    stroke-linecap: round;
  }

  .series.ink {
    stroke: var(--text);
  }

  .series.grey {
    stroke: var(--muted);
  }

  .dot {
    stroke: var(--bg);
    stroke-width: 2;
  }

  .dot.ink {
    fill: var(--text);
  }

  .dot.grey {
    fill: var(--muted);
  }

  .dot.active {
    fill: var(--accent);
  }

  .label {
    font-size: 12px;
    fill: var(--text);
  }

  .select {
    stroke: var(--accent);
    stroke-width: 1;
    shape-rendering: crispEdges;
  }

  .ghost {
    stroke: var(--rule-strong);
    stroke-width: 1;
    shape-rendering: crispEdges;
    opacity: 0.5;
  }
</style>
