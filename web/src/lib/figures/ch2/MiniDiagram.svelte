<!--
  A compact, read-only state diagram for Fig. 2.1, drawn from the engine's layout and edge
  routes (the same ones the playground uses). It scales to the width of its column. The
  transition with index `highlight` is drawn in the accent, with its source state.
-->
<script lang="ts">
  import type { EdgeRouteDto, MachineSpecDto } from '#lib/engine/pkg/turing_wasm.js';
  import type { EditorServices } from '#lib/machine/services.ts';

  interface Props {
    svc: EditorServices;
    spec: MachineSpecDto;
    /** Spec index of the transition to emphasise. */
    highlight?: number | null;
    /** The machine shown is the last valid one while the text has errors. */
    stale?: boolean;
    /** Largest height in CSS pixels. */
    maxHeight?: number;
  }

  let { svc, spec, highlight = null, stale = false, maxHeight = 280 }: Props = $props();

  const layout = $derived(svc.graphLayout(spec));
  const r = $derived(layout.radius);
  const routes = $derived.by<EdgeRouteDto[]>(() => {
    const xs = new Float64Array(layout.nodes.map((n) => n.x));
    const ys = new Float64Array(layout.nodes.map((n) => n.y));
    return svc.graphRoute(spec, xs, ys);
  });
  const routeOf = $derived(new Map(routes.map((route) => [route.edge, route])));

  const LINE = 15;
  const CHAR = 7.3;

  /** The box around states, start arrow and labels, with a margin. */
  const box = $derived.by(() => {
    let x0 = Infinity;
    let y0 = Infinity;
    let x1 = -Infinity;
    let y1 = -Infinity;
    const add = (x: number, y: number) => {
      x0 = Math.min(x0, x);
      y0 = Math.min(y0, y);
      x1 = Math.max(x1, x);
      y1 = Math.max(y1, y);
    };
    for (const n of layout.nodes) {
      add(n.x - r - (n.isStart ? 30 : 0), n.y - r);
      add(n.x + r, n.y + r);
    }
    for (const route of routes) {
      const edge = layout.edges[route.edge];
      const lines = edge?.labels.length ?? 1;
      const w = Math.max(...(edge?.labels.map((l) => l.length) ?? [8])) * CHAR;
      const left = route.labelAnchor === 'start' ? 0 : route.labelAnchor === 'end' ? w : w / 2;
      add(route.labelX - left, route.labelY - (lines * LINE) / 2 - 2);
      add(route.labelX - left + w, route.labelY + (lines * LINE) / 2 + 2);
    }
    const m = 8;
    return { x: x0 - m, y: y0 - m, w: x1 - x0 + 2 * m, h: y1 - y0 + 2 * m };
  });

  let width = $state(0);
  /** Never larger than 1:1, so that labels keep their size on wide screens. */
  const scale = $derived(width > 0 ? Math.min(1, width / box.w, maxHeight / box.h) : 1);

  const active = $derived(highlight === null ? null : spec.transitions[highlight]);

  function arrow(x: number, y: number, angle: number) {
    return `translate(${x} ${y}) rotate(${(angle * 180) / Math.PI})`;
  }

  /** "A to B on 0 and 1" for every edge, for screen readers. */
  const description = $derived.by(() => {
    const names = layout.nodes.map((n) => n.name);
    const parts = layout.edges.map((e) => {
      const reads = e.transitions.map((i) => spec.transitions[i]?.read[0] ?? '');
      const on =
        reads.length > 1 ? `${reads.slice(0, -1).join(', ')} and ${reads.at(-1)}` : reads[0];
      return e.from === e.to
        ? `${names[e.from]} to itself on ${on}`
        : `${names[e.from]} to ${names[e.to]} on ${on}`;
    });
    return `State diagram with start state ${spec.start}: ${parts.join('; ')}.`;
  });
</script>

<div class="diagram" class:stale bind:clientWidth={width}>
  {#if width > 0}
    <svg
      width={box.w * scale}
      height={box.h * scale}
      viewBox="{box.x} {box.y} {box.w} {box.h}"
      role="img"
      aria-label={description}
    >
      {#each layout.edges as edge, e (e)}
        {@const route = routeOf.get(e)}
        {#if route}
          {@const on = highlight !== null && edge.transitions.includes(highlight)}
          <g class="edge" class:on>
            <path d={route.path} class="line" />
            <path
              d="M -9 -4.5 L 0 0 L -9 4.5 z"
              class="head"
              transform={arrow(route.tipX, route.tipY, route.tipAngle)}
            />
          </g>
        {/if}
      {/each}

      {#each layout.nodes as n (n.name)}
        {#if n.isStart}
          <g class="start">
            <line x1={n.x - r - 28} y1={n.y} x2={n.x - r - 2} y2={n.y} />
            <path d="M -8 -4 L 0 0 L -8 4 z" transform={arrow(n.x - r - 1, n.y, 0)} />
          </g>
        {/if}
        <g class="node" class:on={active?.from === n.name} transform="translate({n.x} {n.y})">
          <circle class="body" {r} />
          {#if n.isFinal}<circle class="inner" r={r - 4} />{/if}
          <text class="name" dy="0.35em">{n.name}</text>
        </g>
      {/each}

      {#each layout.edges as edge, e (e)}
        {@const route = routeOf.get(e)}
        {#if route}
          {@const top = route.labelY - ((edge.labels.length - 1) * LINE) / 2}
          {#each edge.labels as label, j (j)}
            <text
              class="label"
              class:on={highlight === edge.transitions[j]}
              x={route.labelX}
              y={top + j * LINE}
              dy="0.35em"
              text-anchor={route.labelAnchor as 'start' | 'middle' | 'end'}>{label}</text
            >
          {/each}
        {/if}
      {/each}
    </svg>
  {/if}
</div>

<style>
  .diagram {
    display: grid;
    place-items: center;
    min-width: 0;
    min-height: 8rem;
    transition: opacity 120ms ease;
  }

  .diagram.stale {
    opacity: 0.45;
  }

  svg {
    display: block;
    overflow: visible;
  }

  .line {
    fill: none;
    stroke: var(--text);
    stroke-width: 1.25;
  }

  .head,
  .start path {
    fill: var(--text);
  }

  .start line {
    stroke: var(--text);
    stroke-width: 1.25;
  }

  .edge.on .line {
    stroke: var(--accent);
    stroke-width: 2;
  }

  .edge.on .head {
    fill: var(--accent);
  }

  .body {
    fill: var(--bg);
    stroke: var(--text);
    stroke-width: 1.25;
  }

  .inner {
    fill: none;
    stroke: var(--text);
    stroke-width: 1.25;
  }

  .node.on .body {
    stroke: var(--accent);
    stroke-width: 2;
  }

  .name {
    font-family: var(--font-mono);
    font-size: 13px;
    text-anchor: middle;
    fill: var(--text);
  }

  .node.on .name {
    fill: var(--accent);
  }

  .label {
    font-family: var(--font-mono);
    font-size: 12px;
    fill: var(--text);
    paint-order: stroke;
    stroke: var(--bg);
    stroke-width: 4px;
    stroke-linejoin: round;
  }

  .label.on {
    fill: var(--accent);
  }
</style>
