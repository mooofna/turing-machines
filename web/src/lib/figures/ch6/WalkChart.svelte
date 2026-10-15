<!--
  The counter a of Antihydra's rule over the iterations, as one ink line on hairline axes.

  The x axis spans the iterations asked for, so the line grows to the right while the walk is
  computed. The baseline is a = 0, the only level at which the machine can halt; it is drawn in
  ink and labelled. A crosshair follows the pointer, or the arrow keys when the chart has focus,
  and the value under it is read out below the chart.
-->
<script lang="ts">
  import { formatInteger } from '#lib/components/Live.svelte';

  interface Sample {
    iteration: number;
    balance: number;
  }

  interface Props {
    samples: readonly Sample[];
    /** The last iteration on the x axis. */
    total: number;
    /** Accessible name of the chart. */
    label: string;
  }

  let { samples, total, label }: Props = $props();

  let width = $state(0);
  const height = 240;
  const margin = { top: 16, right: 16, bottom: 30, left: 60 };
  let active = $state<number | null>(null);

  /** Ticks at 1, 2 or 5 times a power of ten, at least `count` of them up to `max`. */
  function ticks(max: number, count: number): number[] {
    if (max <= 0) return [0];
    const raw = max / count;
    const power = 10 ** Math.floor(Math.log10(raw));
    const step = [1, 2, 5, 10].map((f) => f * power).find((s) => s >= raw) ?? 10 * power;
    // The last tick is the first at or above `max`, so the axis covers the data.
    const out = [];
    for (let v = 0; out.at(-1) === undefined || out.at(-1)! < max - 1e-9; v += step) out.push(v);
    return out;
  }

  const top = $derived(Math.max(10, ...samples.map((s) => s.balance)));
  const empty = $derived(samples.length === 0);
  const yTicks = $derived(empty ? [0] : ticks(top, 4));
  const yMax = $derived(Math.max(top, yTicks.at(-1) ?? top));
  const xTicks = $derived(ticks(total, 4).filter((t) => t <= total));
  const plot = $derived({
    w: Math.max(1, width - margin.left - margin.right),
    h: height - margin.top - margin.bottom
  });
  const x = (i: number) => margin.left + (i / Math.max(1, total)) * plot.w;
  const y = (a: number) => margin.top + plot.h - (a / yMax) * plot.h;

  const path = $derived(
    samples.length > 1
      ? samples
          .map(
            (s, i) =>
              `${i === 0 ? 'M' : 'L'}${x(s.iteration).toFixed(1)},${y(s.balance).toFixed(1)}`
          )
          .join('')
      : ''
  );

  const point = $derived(active !== null ? (samples[active] ?? null) : null);
  const valueText = $derived.by(() => {
    const p = point ?? samples.at(-1);
    return p
      ? `iteration ${formatInteger(p.iteration)}, a = ${formatInteger(p.balance)}`
      : 'no values yet';
  });

  function nearest(px: number): number | null {
    if (samples.length === 0) return null;
    const target = ((px - margin.left) / plot.w) * total;
    let best = 0;
    for (let i = 1; i < samples.length; i++) {
      if (Math.abs(samples[i]!.iteration - target) < Math.abs(samples[best]!.iteration - target)) {
        best = i;
      }
    }
    return best;
  }

  function move(event: PointerEvent) {
    const box = (event.currentTarget as SVGElement).getBoundingClientRect();
    active = nearest(event.clientX - box.left);
  }

  function key(event: KeyboardEvent) {
    if (samples.length === 0) return;
    const last = samples.length - 1;
    const step = event.shiftKey ? 10 : 1;
    const current = active ?? last;
    const next: Record<string, number> = {
      ArrowLeft: Math.max(0, current - step),
      ArrowRight: Math.min(last, current + step),
      Home: 0,
      End: last
    };
    if (!(event.key in next)) return;
    event.preventDefault();
    active = next[event.key]!;
  }
</script>

<div class="chart" bind:clientWidth={width}>
  <!-- The chart is a slider over the samples: the arrow keys move the crosshair. -->
  <div
    class="focus"
    role="slider"
    tabindex={samples.length > 0 ? 0 : -1}
    aria-label={label}
    aria-valuemin={0}
    aria-valuemax={Math.max(0, samples.length - 1)}
    aria-valuenow={active ?? Math.max(0, samples.length - 1)}
    aria-valuetext={valueText}
    aria-disabled={samples.length === 0}
    onkeydown={key}
    onblur={() => (active = null)}
  >
    {#if width > 0}
      <svg
        {width}
        {height}
        viewBox="0 0 {width} {height}"
        aria-hidden="true"
        onpointermove={move}
        onpointerleave={() => (active = null)}
      >
        {#each yTicks as t (t)}
          {#if t > 0}
            <line class="grid" x1={margin.left} x2={margin.left + plot.w} y1={y(t)} y2={y(t)} />
          {/if}
          <text class="tick" x={margin.left - 8} y={y(t)} dy="0.32em" text-anchor="end"
            >{formatInteger(t)}</text
          >
        {/each}
        {#each xTicks as t, i (t)}
          <text
            class="tick"
            x={x(t)}
            y={margin.top + plot.h + 18}
            text-anchor={i === 0
              ? 'start'
              : i === xTicks.length - 1 && x(t) > width - 40
                ? 'end'
                : 'middle'}>{formatInteger(t)}</text
          >
        {/each}
        <line class="zero" x1={margin.left} x2={margin.left + plot.w} y1={y(0)} y2={y(0)} />
        <text class="note" x={margin.left + plot.w} y={y(0) - 6} text-anchor="end"
          >{width < 520
            ? 'a = 0, where it can halt'
            : 'a = 0, the only level at which the machine can halt'}</text
        >
        {#if path}<path class="line" d={path} />{/if}
        {#if empty}
          <text
            class="note"
            x={margin.left + plot.w / 2}
            y={margin.top + plot.h / 2}
            text-anchor="middle">No values yet</text
          >
        {/if}
        {#if point}
          <line
            class="cross"
            x1={x(point.iteration)}
            x2={x(point.iteration)}
            y1={margin.top}
            y2={margin.top + plot.h}
          />
          <circle class="dot" cx={x(point.iteration)} cy={y(point.balance)} r="4" />
        {/if}
      </svg>
    {/if}
  </div>
  <p class="readout mono" aria-hidden="true">
    {#if point}
      iteration {formatInteger(point.iteration)}: a = {formatInteger(point.balance)}
    {:else if !empty}
      <span class="hint">Point at the line to read a value.</span>
    {/if}
  </p>
</div>

<style>
  .chart {
    width: 100%;
  }

  .focus {
    display: block;
  }

  svg {
    display: block;
    overflow: visible;
    touch-action: pan-y;
  }

  .grid {
    stroke: var(--rule);
    stroke-width: 1;
    shape-rendering: crispEdges;
  }

  .zero {
    stroke: var(--rule-strong);
    stroke-width: 1;
    shape-rendering: crispEdges;
  }

  .tick {
    fill: var(--muted);
    font-family: var(--font-mono);
    font-size: 11px;
    font-variant-numeric: tabular-nums;
  }

  .note {
    fill: var(--muted);
    font-size: 12px;
  }

  .line {
    fill: none;
    stroke: var(--text);
    stroke-width: 1.5;
    stroke-linejoin: round;
    stroke-linecap: round;
  }

  .cross {
    stroke: var(--muted);
    stroke-width: 1;
    shape-rendering: crispEdges;
  }

  .dot {
    fill: var(--text);
    stroke: var(--bg);
    stroke-width: 2;
  }

  .readout {
    min-height: 1.5em;
    margin: 0.4rem 0 0 60px;
    font-size: var(--text-xs);
    color: var(--text);
  }

  .hint {
    font-family: var(--font-sans);
    font-variant-numeric: normal;
    color: var(--muted);
  }
</style>
