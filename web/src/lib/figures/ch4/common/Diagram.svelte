<!--
  A space-time diagram drawn by the engine worker at a size the figure chooses.

  Unlike `lib/machine/SpaceTime.svelte`, which keeps cells square, the image and its size on the
  page are set separately: an exact diagram can be rendered with one image pixel per cell and
  step and then shown with cells wider than they are high, so that a long, narrow run still
  shows every step. Exact images are scaled without smoothing; `grid` draws hairlines between
  cells when they are large enough to read one by one. The diagram recolours on a theme change
  without simulating again and releases the worker's job when it is removed or replaced.
-->
<script lang="ts">
  import { engine } from '#lib/engine/client.ts';
  import { onThemeChange, readPalette } from '#lib/engine/palette.ts';
  import type {
    MachineSpecDto,
    SpaceTimeProgressDto,
    SpaceTimeRequestDto
  } from '#lib/engine/pkg/turing_wasm.js';
  import type { Response } from '#lib/engine/protocol.ts';

  interface Props {
    spec: MachineSpecDto;
    /** Image size in pixels and the row mapping. */
    request: SpaceTimeRequestDto;
    /** Size on the page in CSS pixels. */
    width: number;
    height: number;
    /** Accessible description of the image. */
    label: string;
    /** Hairlines between `cols` x `rows` cells, for exact diagrams with large cells. */
    grid?: { cols: number; rows: number } | null;
    /** Called once the image is complete. */
    ondone?: (progress: SpaceTimeProgressDto) => void;
  }

  let { spec, request, width, height, label, grid = null, ondone }: Props = $props();

  let canvas = $state<HTMLCanvasElement>();
  let status = $state<'computing' | 'done' | 'error'>('computing');

  const pixelated = $derived(request.mode.kind === 'exact');

  function draw(frame: Extract<Response, { type: 'spacetime' }>) {
    if (!canvas || frame.width === 0 || frame.height === 0) return;
    if (canvas.width !== frame.width || canvas.height !== frame.height) {
      canvas.width = frame.width;
      canvas.height = frame.height;
    }
    canvas
      .getContext('2d')
      ?.putImageData(new ImageData(frame.pixels, frame.width, frame.height), 0, 0);
  }

  $effect(() => {
    const job = {
      spec: $state.snapshot(spec) as MachineSpecDto,
      request: $state.snapshot(request) as SpaceTimeRequestDto
    };
    let stopped = false;
    let finished = false;
    let stale = false;
    status = 'computing';
    const call = engine().call(
      {
        type: 'spacetime',
        spec: job.spec,
        input: '',
        request: job.request,
        palette: readPalette()
      },
      (response) => {
        if (!stopped && response.type === 'spacetime') draw(response);
      }
    );
    const recolor = () => {
      engine()
        .request({ type: 'recolor', job: call.id, palette: readPalette() }, 'spacetime')
        .then((frame) => {
          if (!stopped) draw(frame);
        })
        .catch(() => undefined);
    };
    call.done
      .then((response) => {
        finished = true;
        if (stopped) return;
        status = 'done';
        if (stale) recolor();
        if (response.type === 'spacetime') ondone?.(response.progress);
      })
      .catch(() => {
        finished = true;
        if (!stopped) status = 'error';
      });
    const unsubscribe = onThemeChange(() => {
      if (finished) recolor();
      else stale = true;
    });
    return () => {
      stopped = true;
      unsubscribe();
      if (!finished) call.cancel();
      const release = () => engine().call({ type: 'release', job: call.id });
      call.done.then(release, release);
    };
  });

  const lines = $derived.by(() => {
    if (!grid) return null;
    const cw = width / grid.cols;
    const rh = height / grid.rows;
    return {
      x: Array.from({ length: grid.cols - 1 }, (_, i) => Math.round((i + 1) * cw)),
      y: Array.from({ length: grid.rows - 1 }, (_, i) => Math.round((i + 1) * rh))
    };
  });
</script>

<div
  class="diagram"
  class:pixelated
  style:width="{width}px"
  style:height="{height}px"
  data-status={status}
  role="img"
  aria-label={label}
>
  <canvas bind:this={canvas} aria-hidden="true"></canvas>
  {#if lines}
    <svg class="grid" viewBox="0 0 {width} {height}" aria-hidden="true">
      {#each lines.x as x (x)}<line x1={x} x2={x} y1="0" y2={height} />{/each}
      {#each lines.y as y (y)}<line x1="0" x2={width} y1={y} y2={y} />{/each}
    </svg>
  {/if}
  {#if status === 'error'}
    <p class="note mono">No diagram</p>
  {/if}
</div>

<style>
  .diagram {
    position: relative;
    flex: none;
  }

  canvas {
    display: block;
    width: 100%;
    height: 100%;
  }

  .pixelated canvas {
    image-rendering: pixelated;
  }

  .grid {
    position: absolute;
    inset: 0;
    width: 100%;
    height: 100%;
    overflow: visible;
  }

  .grid line {
    stroke: var(--rule);
    stroke-width: 1;
    shape-rendering: crispEdges;
    vector-effect: non-scaling-stroke;
  }

  .note {
    position: absolute;
    inset: 0;
    display: grid;
    place-items: center;
    margin: 0;
    color: var(--muted);
    font-size: var(--text-xs);
  }
</style>
