<!--
  A space-time diagram of a single-tape machine, rendered by the engine worker.

  Each row of the image is a moment of the run and each column a tape cell (in `exact` mode), or
  an even sample of the run and the density of written cells (in `overview` mode). The worker
  streams frames while it simulates; this component draws them with putImageData. It starts
  only when it scrolls into view, recolours without simulating again when the theme changes, and
  releases the worker's job when it is removed.

  Size: pass `width` and `height` in CSS pixels, or only `aspect` (width / height) to fill the
  width of the parent. Overview images are rendered at the device pixel ratio; exact images at
  one image pixel per CSS pixel and scaled without smoothing.
-->
<script module lang="ts">
  import type { SpaceTimeProgressDto } from '#lib/engine/pkg/turing_wasm.js';

  /** Progress of a diagram, with the time since the request. */
  export type SpaceTimeProgress = SpaceTimeProgressDto & { elapsedMs: number };
</script>

<script lang="ts">
  import { onMount } from 'svelte';
  import { engine } from '#lib/engine/client.ts';
  import { onThemeChange, readPalette } from '#lib/engine/palette.ts';
  import type { Response } from '#lib/engine/protocol.ts';
  import type { MachineSpecDto, SpaceTimeModeDto } from '#lib/engine/pkg/turing_wasm.js';

  interface Props {
    /** The machine; must have one tape. */
    spec: MachineSpecDto;
    /** Input word on the tape at step 0. */
    input?: string;
    /** Row mapping: exact steps or an overview of a long run. */
    mode: SpaceTimeModeDto;
    /** Width in CSS pixels; omit to fill the parent. */
    width?: number;
    /** Height in CSS pixels; omit to derive it from `aspect`. */
    height?: number;
    /** Width divided by height, used when `width` or `height` is omitted. */
    aspect?: number;
    /** Accessible description of the image. */
    label: string;
    /** Called with every frame's progress. */
    onprogress?: (progress: SpaceTimeProgress) => void;
    /** Extra class on the outer element. */
    class?: string;
  }

  let {
    spec,
    input = '',
    mode,
    width,
    height,
    aspect = 4 / 3,
    label,
    onprogress,
    class: className = ''
  }: Props = $props();

  type Status = 'waiting' | 'computing' | 'done' | 'error';

  let wrapper: HTMLDivElement;
  let canvas: HTMLCanvasElement;
  let visible = $state(false);
  let status = $state<Status>('waiting');
  let drawn = $state(false);
  let message = $state('');

  const exact = $derived(mode.kind === 'exact');
  const ratio = $derived(width && height ? width / height : aspect);
  const fixedWidth = $derived(width && height ? `${width}px` : undefined);

  function draw(frame: Extract<Response, { type: 'spacetime' }>) {
    if (frame.width === 0 || frame.height === 0) return;
    if (canvas.width !== frame.width || canvas.height !== frame.height) {
      canvas.width = frame.width;
      canvas.height = frame.height;
    }
    const context = canvas.getContext('2d');
    if (!context) return;
    context.putImageData(new ImageData(frame.pixels, frame.width, frame.height), 0, 0);
    drawn = true;
  }

  /** Requests a diagram and returns a function that cancels it and frees the job. */
  function render(request: {
    spec: MachineSpecDto;
    input: string;
    mode: SpaceTimeModeDto;
  }): () => void {
    const cssWidth = width ?? Math.max(1, Math.round(wrapper.clientWidth));
    const cssHeight = height ?? Math.max(1, Math.round(cssWidth / aspect));
    const scale = request.mode.kind === 'exact' ? 1 : Math.min(window.devicePixelRatio || 1, 2);
    const px = (n: number) => Math.max(1, Math.round(n * scale));

    let finished = false;
    let stale = false;
    let stopped = false;
    status = 'computing';
    drawn = false;
    message = '';

    const call = engine().call(
      {
        type: 'spacetime',
        spec: request.spec,
        input: request.input,
        request: { width: px(cssWidth), height: px(cssHeight), mode: request.mode },
        palette: readPalette()
      },
      (response) => {
        if (stopped || response.type !== 'spacetime') return;
        draw(response);
        onprogress?.({ ...response.progress, elapsedMs: response.elapsedMs });
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
      .then(() => {
        finished = true;
        if (stopped) return;
        status = 'done';
        if (stale) recolor();
      })
      .catch((error: unknown) => {
        finished = true;
        if (stopped) return;
        status = 'error';
        message = error instanceof Error ? error.message : String(error);
      });

    const unsubscribe = onThemeChange(() => {
      // Frames still streaming carry the old palette; recolour once the job has finished.
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
  }

  onMount(() => {
    if (!('IntersectionObserver' in window)) {
      visible = true;
      return;
    }
    const observer = new IntersectionObserver(
      (entries) => {
        if (entries.some((e) => e.isIntersecting)) {
          visible = true;
          observer.disconnect();
        }
      },
      { rootMargin: '200px 0px' }
    );
    observer.observe(wrapper);
    return () => observer.disconnect();
  });

  $effect(() => {
    if (!visible) return;
    // Snapshots: the worker cannot clone reactive proxies, and reading them here makes the
    // diagram follow changes of the machine, the input and the mode.
    const request = {
      spec: $state.snapshot(spec) as MachineSpecDto,
      input,
      mode: $state.snapshot(mode) as SpaceTimeModeDto
    };
    return render(request);
  });
</script>

<div
  bind:this={wrapper}
  class="spacetime {className}"
  class:exact
  style:aspect-ratio={ratio}
  style:width={fixedWidth}
  data-status={status}
>
  <div class="image" role="img" aria-label={label}>
    <canvas bind:this={canvas} aria-hidden="true"></canvas>
  </div>
  {#if !drawn}
    <p class="state mono" aria-hidden="true">
      {#if status === 'error'}
        No diagram
      {:else if status === 'computing'}
        Computing
      {/if}
    </p>
  {/if}
  {#if status === 'error'}
    <span class="visually-hidden">The diagram could not be computed: {message}</span>
  {/if}
</div>

<style>
  .spacetime {
    position: relative;
    max-width: 100%;
    background: var(--bg);
    overflow: hidden;
  }

  .image {
    width: 100%;
    height: 100%;
  }

  canvas {
    display: block;
    width: 100%;
    height: 100%;
  }

  .exact canvas {
    image-rendering: pixelated;
  }

  .state {
    position: absolute;
    inset: 0;
    display: grid;
    place-items: center;
    margin: 0;
    color: var(--muted);
    font-size: var(--text-xs);
  }

  [data-status='computing'] .state {
    animation: breathe 1.6s ease-in-out infinite alternate;
  }

  @keyframes breathe {
    from {
      color: var(--muted);
    }

    to {
      color: var(--text);
    }
  }

  @media (prefers-reduced-motion: reduce) {
    [data-status='computing'] .state {
      animation: none;
    }
  }
</style>
