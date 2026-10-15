<!--
  A framed plate with the space-time diagram of a run from the blank tape, sized to fit.

  `exact` shows one row per step from step 0 to `last` over the cells the head visited plus one
  blank cell on either side: short runs get square cells (with hairlines between them when they
  are large), long and narrow runs get cells wider than high, so every step stays visible. The
  overview samples a run of `last` steps, one row per pixel. Step labels at the left mark the
  first and the last row; `mark` brackets a range of rows at the right, for example a period.
-->
<script module lang="ts">
  /** How the diagram maps the run onto the plate, for the reading line under a figure. */
  export type PlateLayout =
    | { kind: 'exact'; rows: number; cols: number; cellWidth: number; rowHeight: number }
    | { kind: 'overview'; rows: number; stride: number; cellsPerColumn: number };
</script>

<script lang="ts">
  import { formatInteger } from '#lib/components/Live.svelte';
  import type { MachineSpecDto, SpaceTimeRequestDto } from '#lib/engine/pkg/turing_wasm.js';
  import Diagram from './Diagram.svelte';

  interface Props {
    /** The machine, or null while it is not known yet. */
    spec: MachineSpecDto | null;
    mode: 'exact' | 'overview';
    /** The last step shown: the halting step, or the end of the excerpt. */
    last: number;
    /** Leftmost and rightmost head position up to `last`. */
    extent: [number, number];
    /** Accessible description of the diagram. */
    label: string;
    /** Rows `from` to `to` bracketed at the right, with a short label. */
    mark?: { from: number; to: number; text: string } | null;
    /** Shown in the empty plate before the diagram is requested. */
    waiting?: string;
    /** Whether the diagram failed. */
    failed?: boolean;
    onlayout?: (layout: PlateLayout | null) => void;
  }

  let {
    spec,
    mode,
    last,
    extent,
    label,
    mark = null,
    waiting = '',
    failed = false,
    onlayout
  }: Props = $props();

  /** Largest cell size in CSS pixels. */
  const MAX_CELL = 28;
  /** Space around the diagram inside the plate, across and along time. */
  const PAD = 24;
  const PAD_Y = 16;
  /** Width of the bracket and its label at the right. */
  const MARK = 76;

  let plateWidth = $state(0);
  let plateHeight = $state(0);
  let drawnKey = $state('');

  const geometry = $derived.by(() => {
    if (!spec || plateWidth === 0 || plateHeight === 0) return null;
    // Room for the step labels, in the small mono type of the axis.
    const axis = Math.ceil(`step ${formatInteger(last)}`.length * 6.8) + 10;
    const innerWidth = Math.max(1, plateWidth - 2 * PAD - axis - (mark ? MARK : 0));
    const innerHeight = Math.max(1, plateHeight - 2 * PAD_Y);
    if (mode === 'exact') {
      const rows = last + 1;
      const cols = extent[1] - extent[0] + 3;
      // Whole CSS pixels per row where they fit; otherwise whole device pixels, at least one,
      // and the plate grows if even that does not fit.
      const dpr = Math.max(1, Math.round(window.devicePixelRatio || 1));
      const whole = Math.floor(innerHeight / rows);
      const rh =
        whole >= 1
          ? Math.min(MAX_CELL, whole)
          : Math.max(1, Math.floor((innerHeight * dpr) / rows)) / dpr;
      // Square cells while they are large; otherwise wider than high, at least 6 pixels.
      const wide = rh >= 8 ? rh : Math.min(16, Math.max(4 * rh, 6));
      const cw = Math.max(1, Math.min(wide, Math.floor(innerWidth / cols)));
      const request: SpaceTimeRequestDto = {
        width: cols,
        height: rows,
        mode: { kind: 'exact', firstStep: 0, cellPx: 1 }
      };
      return {
        request,
        width: cols * cw,
        height: rows * rh,
        rowHeight: rh,
        grid: cw >= 10 && rh >= 10 ? { cols, rows } : null,
        layout: { kind: 'exact', rows, cols, cellWidth: cw, rowHeight: rh } as PlateLayout
      };
    }
    const scale = Math.min(window.devicePixelRatio || 1, 2);
    const w = Math.round(innerWidth * scale);
    const h = Math.round(innerHeight * scale);
    const request: SpaceTimeRequestDto = {
      width: w,
      height: h,
      mode: { kind: 'overview', maxSteps: last }
    };
    // The renderer samples every `stride` steps and bins the tape into one column per pixel.
    const stride = Math.max(1, Math.ceil(last / (h - 1)));
    const rows = Math.ceil(last / stride) + 1;
    return {
      request,
      width: innerWidth,
      height: innerHeight,
      rowHeight: innerHeight / rows,
      grid: null,
      layout: {
        kind: 'overview',
        rows,
        stride,
        cellsPerColumn: (extent[1] - extent[0] + 1) / w
      } as PlateLayout
    };
  });

  const key = $derived(
    geometry && spec
      ? `${JSON.stringify(spec.transitions.map((t) => [t.from, t.read, t.to, t.write, t.dir]))}|${mode}|${last}|${geometry.width}x${geometry.height}`
      : ''
  );

  $effect(() => {
    onlayout?.(geometry?.layout ?? null);
  });

  const status = $derived(
    failed ? 'error' : !spec ? 'waiting' : key !== '' && drawnKey === key ? 'done' : 'computing'
  );
</script>

<div
  class="plate frame"
  style:min-height={geometry && geometry.height > plateHeight - 2 * PAD_Y
    ? `${Math.ceil(geometry.height + 2 * PAD_Y + 2)}px`
    : undefined}
  bind:clientWidth={plateWidth}
  bind:clientHeight={plateHeight}
  data-status={status}
>
  {#if spec && geometry}
    {#key key}
      <div class="stage">
        <div class="axis mono" aria-hidden="true">
          <span>step 0</span>
          <span>step {formatInteger(last)}</span>
        </div>
        <Diagram
          {spec}
          request={geometry.request}
          width={geometry.width}
          height={geometry.height}
          grid={geometry.grid}
          {label}
          ondone={() => (drawnKey = key)}
        />
        {#if mark}
          {@const top = mark.from * geometry.rowHeight}
          {@const bottom = (mark.to + 1) * geometry.rowHeight}
          <div class="mark" aria-hidden="true">
            <span class="bracket" style:top="{top}px" style:height="{bottom - top}px"></span>
            <span class="mark-text mono" style:top="{(top + bottom) / 2}px">{mark.text}</span>
          </div>
        {/if}
      </div>
    {/key}
  {:else}
    <p class="waiting mono" aria-hidden="true">{failed ? 'No diagram' : waiting}</p>
  {/if}
</div>

<style>
  .plate {
    position: relative;
    height: var(--plate-height, 30rem);
    display: grid;
    place-items: center;
    overflow: hidden;
  }

  .stage {
    position: relative;
    display: flex;
    align-items: stretch;
    gap: 0.6rem;
  }

  .axis {
    display: flex;
    flex-direction: column;
    justify-content: space-between;
    align-items: flex-end;
    font-size: 0.6875rem;
    line-height: 1;
    color: var(--muted);
    white-space: nowrap;
  }

  .mark {
    position: relative;
    width: 4rem;
  }

  .bracket {
    position: absolute;
    left: 0;
    width: 0.4rem;
    border: 1px solid var(--muted);
    border-left: 0;
  }

  .mark-text {
    position: absolute;
    left: 0.8rem;
    transform: translateY(-50%);
    font-size: 0.6875rem;
    line-height: 1.2;
    color: var(--muted);
    white-space: nowrap;
  }

  .waiting {
    margin: 0;
    color: var(--muted);
    font-size: var(--text-xs);
  }
</style>
