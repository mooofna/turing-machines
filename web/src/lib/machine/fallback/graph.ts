// DEVELOPMENT FALLBACK, see ./index.ts. Layout and edge routing for the state diagram, used only
// while the WebAssembly module is the stub that places states on a line and draws straight
// edges. turing-render is the reference and replaces this file entirely.

import type {
  EdgeRouteDto,
  EdgeRoutesDto,
  GraphEdgeDto,
  GraphLayoutDto,
  MachineSpecDto
} from '#lib/engine/pkg/turing_wasm.js';

const R = 22;

function tuple(items: string[]) {
  return items.length === 1 ? items[0]! : `(${items.join(', ')})`;
}

export function edges(spec: MachineSpecDto): GraphEdgeDto[] {
  const index = new Map(spec.states.map((s, i) => [s.name, i]));
  const out: GraphEdgeDto[] = [];
  spec.transitions.forEach((t, i) => {
    const from = index.get(t.from) ?? 0;
    const to = index.get(t.to) ?? 0;
    const label = `${tuple(t.read)} ↦ ${tuple(t.write)}, ${tuple(t.dir)}`;
    const e = out.find((e) => e.from === from && e.to === to);
    if (e) {
      e.labels.push(label);
      e.transitions.push(i);
    } else out.push({ from, to, labels: [label], transitions: [i] });
  });
  return out;
}

export function layout(spec: MachineSpecDto): GraphLayoutDto {
  const n = spec.states.length;
  const es = edges(spec);
  const xs: number[] = [];
  const ys: number[] = [];
  const dense = es.filter((e) => e.from !== e.to).length > 1.4 * n && n > 3;
  if (dense) {
    const rad = Math.max(90, (n * 120) / (2 * Math.PI));
    for (let i = 0; i < n; i++) {
      const a = Math.PI + (2 * Math.PI * i) / n;
      xs.push(rad + 80 + rad * Math.cos(a));
      ys.push(rad + 70 + rad * Math.sin(a));
    }
  } else {
    // Layers by breadth-first distance from the start state.
    const index = new Map(spec.states.map((s, i) => [s.name, i]));
    const level = new Array<number>(n).fill(-1);
    const start = index.get(spec.start) ?? 0;
    if (n) level[start] = 0;
    const queue = n ? [start] : [];
    while (queue.length) {
      const v = queue.shift()!;
      for (const e of es) {
        if (e.from === v && level[e.to] === -1) {
          level[e.to] = level[v]! + 1;
          queue.push(e.to);
        }
      }
    }
    let max = Math.max(0, ...level);
    for (let i = 0; i < n; i++) if (level[i] === -1) level[i] = ++max;
    const rows = new Map<number, number[]>();
    for (let i = 0; i < n; i++) rows.set(level[i]!, [...(rows.get(level[i]!) ?? []), i]);
    const tallest = Math.max(1, ...[...rows.values()].map((r) => r.length));
    for (let i = 0; i < n; i++) {
      const row = rows.get(level[i]!)!;
      const k = row.indexOf(i);
      xs[i] = 70 + level[i]! * 150;
      ys[i] = 80 + ((tallest - row.length) * 110) / 2 + k * 110 + (level[i]! % 2) * 24;
    }
  }
  const nodes = spec.states.map((s, i) => ({
    name: s.name,
    isStart: s.name === spec.start,
    isFinal: s.isFinal,
    x: xs[i]!,
    y: ys[i]!
  }));
  return {
    nodes,
    edges: es,
    width: Math.max(0, ...xs) + 80,
    height: Math.max(0, ...ys) + 80,
    radius: R
  };
}

export function route(spec: MachineSpecDto, xs: Float64Array, ys: Float64Array): EdgeRoutesDto {
  const es = edges(spec);
  const routes: EdgeRouteDto[] = es.map((e, edge) => {
    const x1 = xs[e.from] ?? 0;
    const y1 = ys[e.from] ?? 0;
    const x2 = xs[e.to] ?? 0;
    const y2 = ys[e.to] ?? 0;
    if (e.from === e.to) {
      // Self loop in the widest gap between the other edges at this node.
      const angles = es
        .filter((o) => o !== e && (o.from === e.from || o.to === e.from))
        .map((o) => {
          const other = o.from === e.from ? o.to : o.from;
          return Math.atan2((ys[other] ?? 0) - y1, (xs[other] ?? 0) - x1);
        })
        .concat(spec.states[e.from]?.name === spec.start ? [Math.PI] : [])
        .sort((a, b) => a - b);
      let theta = -Math.PI / 2;
      if (angles.length) {
        let best = -1;
        angles.forEach((a, i) => {
          const b = i + 1 < angles.length ? angles[i + 1]! : angles[0]! + 2 * Math.PI;
          if (b - a > best) {
            best = b - a;
            theta = a + (b - a) / 2;
          }
        });
      }
      const p = (a: number, d: number) => [x1 + d * Math.cos(a), y1 + d * Math.sin(a)] as const;
      const [sx, sy] = p(theta - 0.45, R);
      const [ex, ey] = p(theta + 0.45, R);
      const [c1x, c1y] = p(theta - 0.75, R * 2.9);
      const [c2x, c2y] = p(theta + 0.75, R * 2.9);
      const [lx, ly] = p(theta, R * 3.1);
      const anchor = Math.cos(theta) > 0.4 ? 'start' : Math.cos(theta) < -0.4 ? 'end' : 'middle';
      return {
        edge,
        path: `M ${sx} ${sy} C ${c1x} ${c1y} ${c2x} ${c2y} ${ex} ${ey}`,
        tipX: ex,
        tipY: ey,
        tipAngle: Math.atan2(ey - c2y, ex - c2x),
        labelX: lx,
        labelY: ly + (Math.sin(theta) < -0.4 ? -8 * (e.labels.length - 1) : 0),
        labelAnchor: anchor
      };
    }
    const twin = es.some((o) => o.from === e.to && o.to === e.from);
    const dx = x2 - x1;
    const dy = y2 - y1;
    const len = Math.hypot(dx, dy) || 1;
    const nx = -dy / len;
    const ny = dx / len;
    const bend = twin ? 26 : 0;
    const cx = (x1 + x2) / 2 + nx * bend;
    const cy = (y1 + y2) / 2 + ny * bend;
    const a1 = Math.atan2(cy - y1, cx - x1);
    const a2 = Math.atan2(cy - y2, cx - x2);
    const sx = x1 + R * Math.cos(a1);
    const sy = y1 + R * Math.sin(a1);
    const ex = x2 + R * Math.cos(a2);
    const ey = y2 + R * Math.sin(a2);
    const mx = 0.25 * sx + 0.5 * cx + 0.25 * ex;
    const my = 0.25 * sy + 0.5 * cy + 0.25 * ey;
    const side = twin ? 1 : -1;
    const off = 14 + 7.5 * (e.labels.length - 1);
    const lx = mx + nx * off * side;
    const ly = my + ny * off * side;
    const horizontal = Math.abs(dx) > Math.abs(dy);
    const anchor = horizontal ? 'middle' : nx * side > 0 ? 'start' : 'end';
    return {
      edge,
      path: `M ${sx} ${sy} Q ${cx} ${cy} ${ex} ${ey}`,
      tipX: ex,
      tipY: ey,
      tipAngle: Math.atan2(ey - cy, ex - cx),
      labelX: lx,
      labelY: ly,
      labelAnchor: anchor
    };
  });
  return { routes };
}
