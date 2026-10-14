<!--
  The state diagram. Layout and edge routes come from the engine; states can be dragged, and
  edges follow on every animation frame. Double-click empty space to add a state, drag from the
  handle on a state's edge to another state (or to empty space for a new one) to add a
  transition, click a label to edit it. States are focusable: arrow keys move them, Enter opens
  their menu, Delete removes them. Drag the background to pan, pinch or Ctrl and scroll to zoom.
-->
<script lang="ts">
  import { tick } from 'svelte';
  import type { EdgeRouteDto, GraphEdgeDto } from '#lib/engine/pkg/turing_wasm.js';
  import { nextStateName } from './edit';
  import Menu from './Menu.svelte';
  import { stateMenu } from './menus';
  import RenameForm from './RenameForm.svelte';
  import type { Playground, Point, Positions } from './state.svelte';
  import TransitionForm from './TransitionForm.svelte';

  interface Props {
    pg: Playground;
    /** False on small screens: pan and zoom only. */
    editable?: boolean;
    /** Whether the diagram is on screen, so it can fit once it has a size. */
    visible?: boolean;
  }

  let { pg, editable = true, visible = true }: Props = $props();
  const id = $props.id();

  const spec = $derived(pg.spec!);
  const layout = $derived(pg.services.graphLayout(spec));
  const edges = $derived<GraphEdgeDto[]>(layout.edges);
  const r = $derived(layout.radius);

  let svg: SVGSVGElement | null = $state(null);
  let width = $state(0);
  let height = $state(0);
  let view = $state({ x: 0, y: 0, k: 1 });

  // A state being dragged, or a transition being drawn.
  let dragging = $state<{ name: string; x: number; y: number } | null>(null);
  let linking = $state<{ from: string; x: number; y: number; over: string | null } | null>(null);
  let panning = false;

  // Popovers.
  let menu = $state<{ name: string; anchor: DOMRect } | null>(null);
  let renaming = $state<{ name: string; anchor: DOMRect } | null>(null);
  let form = $state<{
    from: string;
    to: string;
    at: Point | null;
    index: number | null;
    anchor: DOMRect;
  } | null>(null);

  const positions = $derived.by<Positions>(() => {
    const p = pg.positions;
    if (!dragging) return p;
    return { ...p, [dragging.name]: { x: dragging.x, y: dragging.y } };
  });

  const nodes = $derived(
    spec.states.map((s) => ({
      name: s.name,
      isFinal: s.isFinal,
      isStart: s.name === spec.start,
      x: positions[s.name]?.x ?? 0,
      y: positions[s.name]?.y ?? 0,
      out: spec.transitions.filter((t) => t.from === s.name).length
    }))
  );

  let routes = $state.raw<EdgeRouteDto[]>([]);
  let frame = 0;

  // Route the edges again whenever the machine or a position changes, at most once per frame.
  $effect(() => {
    const s = spec;
    const xs = new Float64Array(nodes.map((n) => n.x));
    const ys = new Float64Array(nodes.map((n) => n.y));
    cancelAnimationFrame(frame);
    const compute = () => (routes = pg.services.graphRoute(s, xs, ys));
    if (dragging) frame = requestAnimationFrame(compute);
    else compute();
    return () => cancelAnimationFrame(frame);
  });

  const routeOf = $derived(new Map(routes.map((route) => [route.edge, route])));
  const current = $derived(pg.snapshot?.state ?? null);
  const last = $derived(pg.snapshot?.lastTransition);

  // Fit when a machine is loaded and whenever the diagram first gets a size.
  let fitted = -1;
  $effect(() => {
    const loads = pg.loads;
    if (!visible || width === 0 || height === 0) return;
    if (fitted === loads) return;
    fitted = loads;
    fit();
  });

  function bounds() {
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
    for (const n of nodes) {
      add(n.x - r - (n.isStart ? 30 : 0), n.y - r);
      add(n.x + r, n.y + r);
    }
    for (const route of routes) {
      const e = edges[route.edge];
      const lines = e?.labels.length ?? 1;
      const w = Math.max(...(e?.labels.map((l) => l.length) ?? [8])) * 7.3;
      const left = route.labelAnchor === 'start' ? 0 : route.labelAnchor === 'end' ? w : w / 2;
      add(route.labelX - left, route.labelY - lines * 8);
      add(route.labelX - left + w, route.labelY + lines * 8);
    }
    return { x0, y0, x1, y1 };
  }

  function fit() {
    if (!nodes.length || !width || !height) return;
    const b = bounds();
    const pad = 24;
    const bw = Math.max(1, b.x1 - b.x0);
    const bh = Math.max(1, b.y1 - b.y0);
    const k = Math.min(1.2, (width - 2 * pad) / bw, (height - 2 * pad) / bh);
    view = {
      k,
      x: width / 2 - ((b.x0 + b.x1) / 2) * k,
      y: height / 2 - ((b.y0 + b.y1) / 2) * k
    };
  }

  function zoom(factor: number, cx = width / 2, cy = height / 2) {
    const k = Math.max(0.25, Math.min(4, view.k * factor));
    const f = k / view.k;
    view = { k, x: cx - (cx - view.x) * f, y: cy - (cy - view.y) * f };
  }

  /** Screen point (client coordinates) to diagram coordinates. */
  function toWorld(clientX: number, clientY: number): Point {
    const box = svg!.getBoundingClientRect();
    return { x: (clientX - box.left - view.x) / view.k, y: (clientY - box.top - view.y) / view.k };
  }

  /** Diagram point to a small client rectangle, to anchor popovers. */
  function toClient(p: Point): DOMRect {
    const box = svg!.getBoundingClientRect();
    return new DOMRect(box.left + view.x + p.x * view.k, box.top + view.y + p.y * view.k, 0, 0);
  }

  function nodeRect(name: string): DOMRect {
    const n = nodes.find((x) => x.name === name);
    if (!n) return toClient({ x: 0, y: 0 });
    const box = svg!.getBoundingClientRect();
    const k = view.k;
    return new DOMRect(
      box.left + view.x + (n.x - r) * k,
      box.top + view.y + (n.y - r) * k,
      2 * r * k,
      2 * r * k
    );
  }

  function nodeAt(p: Point): string | null {
    for (const n of nodes) if (Math.hypot(n.x - p.x, n.y - p.y) <= r + 6) return n.name;
    return null;
  }

  // Pointers: one moves, links or pans; two pinch.
  const pointers = new Map<number, { x: number; y: number }>();
  let pinch: { distance: number; k: number } | null = null;
  let press: { x: number; y: number; moved: boolean; name: string | null; offset: Point } | null =
    null;

  function down(event: PointerEvent, name: string | null, mode: 'move' | 'link' | 'pan') {
    if (event.button !== 0) return;
    event.stopPropagation();
    svg?.setPointerCapture(event.pointerId);
    pointers.set(event.pointerId, { x: event.clientX, y: event.clientY });
    if (pointers.size === 2) {
      const [a, b] = [...pointers.values()] as [Point, Point];
      pinch = { distance: Math.hypot(a.x - b.x, a.y - b.y), k: view.k };
      dragging = null;
      linking = null;
      return;
    }
    const p = toWorld(event.clientX, event.clientY);
    const node = name ? nodes.find((n) => n.name === name) : null;
    press = {
      x: event.clientX,
      y: event.clientY,
      moved: false,
      name,
      offset: node ? { x: p.x - node.x, y: p.y - node.y } : { x: 0, y: 0 }
    };
    if (mode === 'link' && editable && name) {
      event.preventDefault();
      linking = { from: name, x: p.x, y: p.y, over: null };
    } else if (mode === 'pan' || !editable) {
      panning = true;
    }
  }

  function moveTo(event: PointerEvent) {
    if (!pointers.has(event.pointerId)) return;
    const prev = pointers.get(event.pointerId)!;
    pointers.set(event.pointerId, { x: event.clientX, y: event.clientY });
    if (pinch && pointers.size === 2) {
      const [a, b] = [...pointers.values()] as [Point, Point];
      const box = svg!.getBoundingClientRect();
      const d = Math.hypot(a.x - b.x, a.y - b.y);
      zoom(
        (pinch.k * d) / pinch.distance / view.k,
        (a.x + b.x) / 2 - box.left,
        (a.y + b.y) / 2 - box.top
      );
      return;
    }
    if (!press) return;
    if (!press.moved && Math.hypot(event.clientX - press.x, event.clientY - press.y) < 4) return;
    press.moved = true;
    const p = toWorld(event.clientX, event.clientY);
    if (linking) {
      const over = nodeAt(p);
      linking = { ...linking, x: p.x, y: p.y, over };
    } else if (panning) {
      view = { ...view, x: view.x + event.clientX - prev.x, y: view.y + event.clientY - prev.y };
    } else if (press.name && editable) {
      dragging = { name: press.name, x: p.x - press.offset.x, y: p.y - press.offset.y };
    }
  }

  function up(event: PointerEvent) {
    if (!pointers.has(event.pointerId)) return;
    pointers.delete(event.pointerId);
    if (pinch) {
      if (pointers.size < 2) pinch = null;
      press = null;
      return;
    }
    const p = toWorld(event.clientX, event.clientY);
    const wasPress = press;
    press = null;
    panning = false;
    if (linking) {
      const from = linking.from;
      const over = nodeAt(p);
      linking = null;
      if (!wasPress?.moved && !over) return;
      openForm(from, over, over ? null : p, over ? nodeRect(over) : toClient(p));
      return;
    }
    if (dragging) {
      const { name, x, y } = dragging;
      dragging = null;
      pg.movePositions({ ...pg.positions, [name]: { x, y } });
      return;
    }
    if (wasPress && !wasPress.moved) {
      if (wasPress.name) {
        pg.selection = { kind: 'state', name: wasPress.name };
        if (editable) menu = { name: wasPress.name, anchor: nodeRect(wasPress.name) };
      } else {
        pg.selection = null;
      }
    }
  }

  function openForm(from: string, to: string | null, at: Point | null, anchor: DOMRect) {
    form = { from, to: to ?? nextStateName(spec), at, index: null, anchor };
  }

  function dblclick(event: MouseEvent) {
    if (!editable) return;
    if ((event.target as Element).closest('.node, .label')) return;
    const p = toWorld(event.clientX, event.clientY);
    const name = pg.addState(undefined, p, 'diagram');
    if (name) void tick().then(() => focusNode(name));
  }

  function wheel(event: WheelEvent) {
    if (!event.ctrlKey && !event.metaKey) return;
    event.preventDefault();
    const box = svg!.getBoundingClientRect();
    zoom(Math.exp(-event.deltaY * 0.01), event.clientX - box.left, event.clientY - box.top);
  }

  function focusNode(name: string) {
    svg?.querySelector<SVGGElement>(`[data-node="${CSS.escape(name)}"]`)?.focus();
  }

  function nodeKeydown(event: KeyboardEvent, name: string) {
    const step = event.shiftKey ? 40 : 8;
    const moves: Record<string, [number, number]> = {
      ArrowLeft: [-step, 0],
      ArrowRight: [step, 0],
      ArrowUp: [0, -step],
      ArrowDown: [0, step]
    };
    const d = moves[event.key];
    if (d && editable) {
      event.preventDefault();
      const p = pg.positions[name] ?? { x: 0, y: 0 };
      pg.movePositions({ ...pg.positions, [name]: { x: p.x + d[0], y: p.y + d[1] } });
    } else if ((event.key === 'Enter' || event.key === ' ') && editable) {
      event.preventDefault();
      pg.selection = { kind: 'state', name };
      menu = { name, anchor: nodeRect(name) };
    } else if ((event.key === 'Delete' || event.key === 'Backspace') && editable) {
      event.preventDefault();
      pg.removeState(name, 'diagram');
    }
  }

  function editLabel(index: number, anchor: DOMRect) {
    if (!editable) return;
    const t = spec.transitions[index];
    if (!t) return;
    form = { from: t.from, to: t.to, at: null, index, anchor };
  }

  function labelRect(event: Event): DOMRect {
    const el = event.currentTarget as SVGGraphicsElement;
    return el.getBoundingClientRect();
  }

  function stateLabel(n: (typeof nodes)[number]) {
    const parts = [`State ${n.name}`];
    if (n.isStart) parts.push('start');
    if (n.isFinal) parts.push('final');
    if (current === n.name) parts.push('current');
    parts.push(`${n.out} outgoing transition${n.out === 1 ? '' : 's'}`);
    return parts.join(', ');
  }

  function arrow(x: number, y: number, angle: number) {
    const a = (angle * 180) / Math.PI;
    return `translate(${x} ${y}) rotate(${a})`;
  }

  function transitionText(i: number) {
    const t = spec.transitions[i];
    if (!t) return '';
    const tuple = (xs: string[]) => (xs.length === 1 ? xs[0] : `(${xs.join(', ')})`);
    return `δ(${t.from}, ${tuple(t.read)}) = (${t.to}, ${tuple(t.write)}, ${tuple(t.dir)})`;
  }

  const lineHeight = 15;

  const rubber = $derived.by(() => {
    if (!linking) return null;
    const from = nodes.find((n) => n.name === linking!.from);
    return from ? { x1: from.x, y1: from.y, x2: linking.x, y2: linking.y } : null;
  });

  function nodeMenu(name: string, anchor: DOMRect) {
    return stateMenu(
      pg,
      name,
      {
        rename: () => (renaming = { name, anchor }),
        addTransition: () => (form = { from: name, to: name, at: null, index: null, anchor })
      },
      'diagram'
    );
  }

  function closeMenu() {
    const name = menu?.name;
    menu = null;
    if (name) focusNode(name);
  }
</script>

<div
  class="diagram"
  class:editable
  bind:clientWidth={width}
  bind:clientHeight={height}
  data-own-keys
>
  <div class="tools">
    <button type="button" class="tool" onclick={() => zoom(1.25)} aria-label="Zoom in">+</button>
    <button type="button" class="tool" onclick={() => zoom(0.8)} aria-label="Zoom out"
      >&minus;</button
    >
    <button type="button" class="tool text" onclick={fit}>Fit</button>
    {#if editable}
      <button type="button" class="tool text" onclick={() => pg.arrange()}>Arrange</button>
    {/if}
  </div>
  <!-- svelte-ignore a11y_no_noninteractive_element_interactions -->
  <svg
    bind:this={svg}
    role="group"
    aria-label="State diagram"
    aria-describedby={`${id}-help`}
    {width}
    {height}
    class:linking={linking !== null}
    onpointerdown={(e) => down(e, null, 'pan')}
    onpointermove={moveTo}
    onpointerup={up}
    onpointercancel={up}
    ondblclick={dblclick}
    onwheel={wheel}
  >
    <defs>
      <marker
        id={`${id}-tip`}
        viewBox="0 0 10 10"
        refX="9"
        refY="5"
        markerWidth="7"
        markerHeight="7"
        orient="auto-start-reverse"
      >
        <path d="M 0 0 L 10 5 L 0 10 z" class="tip" />
      </marker>
    </defs>
    <g transform={`translate(${view.x} ${view.y}) scale(${view.k})`}>
      <!-- Edges -->
      {#each edges as edge, e (e)}
        {@const route = routeOf.get(e)}
        {#if route}
          {@const active = last !== undefined && edge.transitions.includes(last)}
          <g class="edge" class:active>
            <path d={route.path} class="line" />
            <path
              d="M -9 -4.5 L 0 0 L -9 4.5 z"
              class="head"
              transform={arrow(route.tipX, route.tipY, route.tipAngle)}
            />
          </g>
        {/if}
      {/each}

      <!-- Start arrows -->
      {#each nodes as n (n.name)}
        {#if n.isStart}
          <g class="start" aria-hidden="true">
            <line x1={n.x - r - 28} y1={n.y} x2={n.x - r - 2} y2={n.y} />
            <path d="M -8 -4 L 0 0 L -8 4 z" transform={arrow(n.x - r - 1, n.y, 0)} />
          </g>
        {/if}
      {/each}

      <!-- States -->
      {#each nodes as n (n.name)}
        <g
          class="node"
          class:active={current === n.name}
          class:selected={pg.selection?.kind === 'state' && pg.selection.name === n.name}
          class:target={linking?.over === n.name}
          data-node={n.name}
          transform={`translate(${n.x} ${n.y})`}
          tabindex="0"
          role="button"
          aria-haspopup={editable ? 'menu' : undefined}
          aria-label={stateLabel(n)}
          onkeydown={(e) => nodeKeydown(e, n.name)}
          onfocus={() => (pg.selection = { kind: 'state', name: n.name })}
        >
          <circle class="focus-ring" r={r + 5} />
          <!-- svelte-ignore a11y_no_static_element_interactions -->
          <circle class="body" {r} onpointerdown={(e) => down(e, n.name, 'move')} />
          {#if n.isFinal}<circle class="inner" r={r - 4} />{/if}
          <text class="name" dy="0.35em">{n.name}</text>
          {#if editable}
            <!-- svelte-ignore a11y_no_static_element_interactions -->
            <circle class="rim" r={r + 1} onpointerdown={(e) => down(e, n.name, 'link')} />
            <!-- svelte-ignore a11y_no_static_element_interactions -->
            <rect
              class="handle"
              x={r - 4}
              y={-4}
              width="8"
              height="8"
              onpointerdown={(e) => down(e, n.name, 'link')}
            />
          {/if}
        </g>
      {/each}

      <!-- Labels, above everything so that they stay readable -->
      {#each edges as edge, e (e)}
        {@const route = routeOf.get(e)}
        {#if route}
          {@const top = route.labelY - ((edge.labels.length - 1) * lineHeight) / 2}
          {#each edge.labels as label, j (j)}
            {@const index = edge.transitions[j]!}
            <g
              class="label"
              class:active={last === index}
              role="button"
              tabindex={editable ? 0 : -1}
              aria-label={editable ? `Edit ${transitionText(index)}` : transitionText(index)}
              onpointerdown={(e) => e.stopPropagation()}
              onclick={(e) => editLabel(index, labelRect(e))}
              onkeydown={(e) => {
                if (e.key === 'Enter' || e.key === ' ') {
                  e.preventDefault();
                  editLabel(index, labelRect(e));
                }
              }}
            >
              <text
                x={route.labelX}
                y={top + j * lineHeight}
                dy="0.35em"
                text-anchor={route.labelAnchor}>{label}</text
              >
            </g>
          {/each}
        {/if}
      {/each}

      {#if rubber}
        <line
          class="rubber"
          x1={rubber.x1}
          y1={rubber.y1}
          x2={rubber.x2}
          y2={rubber.y2}
          marker-end={`url(#${id}-tip)`}
        />
      {/if}
    </g>
  </svg>
  <p class="help" id={`${id}-help`}>
    {#if editable}
      Double-click to add a state. Drag from the square on a state to connect it. Click a label to
      edit it. Ctrl and scroll to zoom.
    {:else}
      Drag to pan, pinch to zoom. Edit the machine in the table.
    {/if}
  </p>
</div>

{#if menu}
  <Menu
    open={true}
    anchor={menu.anchor}
    label={`State ${menu.name}`}
    items={nodeMenu(menu.name, menu.anchor)}
    onclose={closeMenu}
  />
{/if}

{#if renaming}
  <RenameForm
    {pg}
    name={renaming.name}
    anchor={renaming.anchor}
    origin="diagram"
    onclose={() => (renaming = null)}
  />
{/if}

{#if form}
  <TransitionForm
    {pg}
    from={form.from}
    to={form.to}
    at={form.at}
    index={form.index}
    anchor={form.anchor}
    onclose={() => (form = null)}
  />
{/if}

<style>
  .diagram {
    position: relative;
    height: 28rem;
    border: 1px solid var(--rule);
    background: var(--bg);
    touch-action: none;
    overflow: hidden;
  }

  svg {
    display: block;
    cursor: grab;
    user-select: none;
    -webkit-user-select: none;
  }

  svg:active {
    cursor: grabbing;
  }

  svg.linking {
    cursor: crosshair;
  }

  .tools {
    position: absolute;
    top: 0.5rem;
    right: 0.5rem;
    z-index: 1;
    display: flex;
  }

  .tool {
    display: grid;
    place-items: center;
    min-width: 2rem;
    height: 2rem;
    margin-left: -1px;
    padding: 0 0.5rem;
    border: 1px solid var(--rule-strong);
    background: var(--bg);
    font-size: var(--text-sm);
    cursor: pointer;
  }

  .tool:hover {
    color: var(--accent);
    border-color: var(--accent);
    position: relative;
  }

  .tool:focus-visible {
    position: relative;
  }

  .help {
    position: absolute;
    left: 0.75rem;
    bottom: 0.5rem;
    right: 0.75rem;
    margin: 0;
    font-size: var(--text-xs);
    color: var(--muted);
    pointer-events: none;
  }

  .edge .line {
    fill: none;
    stroke: var(--text);
    stroke-width: 1.25;
  }

  .edge .head {
    fill: var(--text);
  }

  .edge.active .line {
    stroke: var(--accent);
    stroke-width: 2;
  }

  .edge.active .head {
    fill: var(--accent);
  }

  .start line {
    stroke: var(--text);
    stroke-width: 1.25;
  }

  .start path {
    fill: var(--text);
  }

  .node {
    cursor: pointer;
    outline: none;
  }

  .editable .node .body {
    cursor: move;
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
    pointer-events: none;
  }

  .name {
    font-family: var(--font-mono);
    font-size: 13px;
    text-anchor: middle;
    fill: var(--text);
    pointer-events: none;
  }

  .node.active .body,
  .node.active .inner {
    stroke: var(--accent);
    stroke-width: 2;
  }

  .node.active .name {
    fill: var(--accent);
    font-weight: 500;
  }

  .node.target .body {
    stroke-width: 2.5;
  }

  .focus-ring {
    fill: none;
    stroke: transparent;
    stroke-width: 2;
    pointer-events: none;
  }

  .node:focus-visible .focus-ring {
    stroke: var(--accent);
  }

  .node.selected .focus-ring {
    stroke: var(--rule-strong);
    stroke-width: 1;
    stroke-dasharray: 3 3;
  }

  .node.selected:focus-visible .focus-ring {
    stroke: var(--accent);
    stroke-width: 2;
    stroke-dasharray: none;
  }

  .rim {
    fill: none;
    stroke: transparent;
    stroke-width: 8;
    pointer-events: stroke;
    cursor: crosshair;
  }

  .handle {
    fill: var(--bg);
    stroke: var(--text);
    stroke-width: 1.25;
    opacity: 0;
    cursor: crosshair;
    transition: opacity 120ms ease;
  }

  .node:hover .handle,
  .node:focus-visible .handle,
  .node.selected .handle {
    opacity: 1;
  }

  .label {
    cursor: pointer;
    outline: none;
  }

  .label text {
    font-family: var(--font-mono);
    font-size: 12px;
    fill: var(--text);
    paint-order: stroke;
    stroke: var(--bg);
    stroke-width: 4px;
    stroke-linejoin: round;
  }

  .label:hover text {
    text-decoration: underline;
  }

  .label.active text {
    fill: var(--accent);
  }

  .label:focus-visible text {
    fill: var(--accent);
    text-decoration: underline;
  }

  .rubber {
    stroke: var(--text);
    stroke-width: 1.25;
    stroke-dasharray: 4 3;
    pointer-events: none;
  }

  .tip {
    fill: var(--text);
  }

  @media (max-width: 47.99rem) {
    .diagram {
      height: 22rem;
    }
  }
</style>
