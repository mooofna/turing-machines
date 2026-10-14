<!--
  Fig. 1.3: M*2 on one tape and on two tapes.

  When the figure scrolls into view, the engine runs both machines on x^n for every n from 0 to
  20 and the chart plots the measured numbers of steps. Below it, the slider selects n, and both
  runs are shown side by side; Play (the primary action) steps them in sync at a calm speed, so
  that the two-tape machine is done while the one-tape machine is still walking back and forth.
  Under reduced motion, Play shows the final configurations at once.
-->
<script lang="ts">
  import type { MachineSpecDto, SnapshotDto } from '#lib/engine/pkg/turing_wasm.js';
  import { count } from '#lib/machine/describe.ts';
  import { SessionClient } from '#lib/machine/session.ts';
  import TapeView from '#lib/machine/TapeView.svelte';
  import type { FigureProps } from '../types.ts';
  import DoublingChart from './DoublingChart.svelte';
  import { errorText, gallerySpec, inView, reducedMotion } from './kit.ts';
  import Plate from './Plate.svelte';

  let { id }: FigureProps = $props();
  const uid = $props.id();

  const LAST = 20;
  const BUDGET = 1_000_000;
  const NONE = { transitions: [], states: [] };

  const LANES = [
    { key: 'one', title: 'One tape', tapes: 1 },
    { key: 'two', title: 'Two tapes', tapes: 2 }
  ] as const;

  let n = $state(6);
  let one = $state.raw<number[] | null>(null);
  let two = $state.raw<number[] | null>(null);
  /** The current configuration of each lane. */
  let snapshots = $state.raw<(SnapshotDto | null)[]>([null, null]);
  // Not reactive: specs are sent to the worker, which cannot clone reactive proxies.
  const specs: (MachineSpecDto | null)[] = [null, null];
  const sessions: (SessionClient | null)[] = [null, null];
  const radii = [10, 10];
  let t = $state(0);
  let playing = $state(false);
  let failure = $state<string | null>(null);
  let frame = 0;
  let token = 0;
  const visible = inView(start);

  const totals = $derived(one && two ? [one[n] ?? 0, two[n] ?? 0] : null);
  const end = $derived(totals ? Math.max(...totals) : 0);
  const measuring = $derived(!failure && (!one || !two || !snapshots[0] || !snapshots[1]));

  /** False once the figure is removed, so that a measurement in progress stops. */
  let alive = true;

  async function measure(spec: MachineSpecDto): Promise<number[]> {
    const probe = new SessionClient();
    const steps: number[] = [];
    try {
      for (let k = 0; k <= LAST; k++) {
        if (!alive) throw new Error('the figure was removed');
        await probe.open(spec, 'x'.repeat(k), 0);
        const outcome = await probe.run(BUDGET, NONE, 0);
        if (!outcome || outcome.result.snapshot.status !== 'halted') {
          throw new Error(`M*2 did not halt on n = ${k}`);
        }
        steps.push(outcome.result.snapshot.step);
      }
    } finally {
      probe.close();
    }
    return steps;
  }

  async function start() {
    try {
      const [a, b] = await Promise.all([
        gallerySpec('doubling'),
        gallerySpec('doubling-two-tapes')
      ]);
      specs[0] = a;
      specs[1] = b;
      sessions[0] = new SessionClient();
      sessions[1] = new SessionClient();
      await openAll();
      one = await measure(a);
      two = await measure(b);
    } catch (error) {
      failure = errorText(error);
    }
  }

  $effect(() => () => {
    alive = false;
    stop();
    for (const session of sessions) session?.close();
  });

  function show(i: number, s: SnapshotDto | null) {
    if (s) snapshots = snapshots.with(i, s);
  }

  async function openAll() {
    stop();
    t = 0;
    const word = 'x'.repeat(n);
    await Promise.all(
      LANES.map(async (_, i) => {
        const spec = specs[i];
        const session = sessions[i];
        if (!spec || !session) return;
        show(i, await session.open(spec, word, radii[i]!));
      })
    ).catch((error: unknown) => (failure = errorText(error)));
  }

  function choose(value: number) {
    const v = Math.max(0, Math.min(LAST, Math.round(value)));
    if (v === n) return;
    n = v;
    void openAll();
  }

  async function seekAll(target: number) {
    await Promise.all(
      LANES.map(async (_, i) => {
        const session = sessions[i];
        if (!session || !totals) return;
        const goal = Math.min(target, totals[i] ?? 0);
        if (snapshots[i]?.step === goal) return;
        show(i, await session.seek(goal, radii[i]!));
      })
    ).catch((error: unknown) => (failure = errorText(error)));
  }

  function stop() {
    token++;
    cancelAnimationFrame(frame);
    playing = false;
  }

  async function play() {
    if (playing) {
      stop();
      return;
    }
    if (!totals) return;
    if (reducedMotion()) {
      t = end;
      await seekAll(end);
      return;
    }
    const from = t >= end ? 0 : t;
    t = from;
    await seekAll(from);
    // About six seconds for the one-tape run, but never faster than it can be followed.
    const rate = Math.max(8, Math.min(200, end / 6));
    const mine = ++token;
    playing = true;
    const t0 = performance.now();
    let busy = false;
    const tick = (now: number) => {
      if (mine !== token) return;
      const target = Math.min(end, from + Math.floor(((now - t0) / 1000) * rate));
      if (!busy && target !== t) {
        busy = true;
        t = target;
        void seekAll(target).then(() => {
          busy = false;
          if (mine === token && target >= end) stop();
        });
      }
      if (mine === token) frame = requestAnimationFrame(tick);
    };
    frame = requestAnimationFrame(tick);
  }

  function resize(i: number, r: number) {
    if (r === radii[i]) return;
    radii[i] = r;
    const session = sessions[i];
    if (session && snapshots[i] && !playing) void session.snapshot(r).then((s) => show(i, s));
  }

  function laneStatus(i: number): string {
    const s = snapshots[i];
    if (!s || !totals) return '';
    const total = totals[i] ?? 0;
    return s.status === 'halted'
      ? `halted after ${count(total)} steps`
      : `step ${count(s.step)} of ${count(total)}`;
  }
</script>

<Plate {id} busy={measuring} attach={visible}>
  <DoublingChart {one} {two} last={LAST} selected={n} onselect={choose} />

  <div class="controls">
    <button
      class="button primary play"
      type="button"
      onclick={play}
      disabled={!totals || !snapshots[0]}>{playing ? 'Pause' : 'Play'}</button
    >
    <label class="choose" for="{uid}-n">
      <span class="fig-label">Input x<sup>n</sup></span>
    </label>
    <input
      id="{uid}-n"
      class="slider"
      type="range"
      min="0"
      max={LAST}
      step="1"
      value={n}
      aria-valuetext="n = {n}"
      oninput={(e) => choose(Number(e.currentTarget.value))}
    />
    <span class="mono value">n = {n}</span>
  </div>

  <div class="lanes">
    {#each LANES as lane, i (lane.key)}
      {@const snapshot = snapshots[i]}
      <div class="lane" role="group" aria-label="{lane.title}, n = {n}">
        <p class="lane-head">
          <span class="lane-title">{lane.title}</span>
          <span class="mono lane-status" class:done={snapshot?.status === 'halted'}
            >{laneStatus(i)}</span
          >
        </p>
        <TapeView
          tapes={snapshot?.tapes ?? null}
          current={snapshot?.state ?? 'q0'}
          rows={lane.tapes}
          slide={0}
          onradius={(r) => resize(i, r)}
        />
      </div>
    {/each}
  </div>

  <p class="visually-hidden" aria-live="polite">
    {totals
      ? `n = ${n}: ${count(totals[0] ?? 0)} steps on one tape, ${count(totals[1] ?? 0)} on two tapes.`
      : ''}
  </p>

  {#if failure}
    <p class="fig-error">The engine could not run these machines: {failure}</p>
  {/if}

  <details class="numbers">
    <summary>Measured steps as a table</summary>
    <table class="table">
      <thead>
        <tr>
          <th scope="col" class="num">n</th>
          <th scope="col" class="num">One tape</th>
          <th scope="col" class="num">Two tapes</th>
        </tr>
      </thead>
      <tbody>
        {#each Array.from({ length: LAST + 1 }, (_, k) => k) as k (k)}
          <tr>
            <th scope="row" class="num">{k}</th>
            <td class="num">{one ? count(one[k] ?? 0) : ''}</td>
            <td class="num">{two ? count(two[k] ?? 0) : ''}</td>
          </tr>
        {/each}
      </tbody>
    </table>
  </details>
</Plate>

<style>
  .controls {
    display: flex;
    flex-wrap: wrap;
    align-items: center;
    gap: var(--space-1) 0.9rem;
    margin-top: var(--space-2);
    padding-top: var(--space-2);
    border-top: 1px solid var(--rule);
  }

  .play {
    min-width: 5.5rem;
    justify-content: center;
    margin-right: var(--space-1);
  }

  .choose .fig-label {
    margin: 0;
  }

  .slider {
    flex: 1 1 12rem;
    min-width: 8rem;
    max-width: 24rem;
  }

  .value {
    min-width: 4rem;
    font-size: var(--text-sm);
  }

  .lanes {
    display: grid;
    grid-template-columns: repeat(2, minmax(0, 1fr));
    gap: var(--space-3);
    margin-top: var(--space-3);
  }

  .lane {
    min-width: 0;
  }

  .lane-head {
    display: flex;
    justify-content: space-between;
    align-items: baseline;
    gap: var(--space-1);
    margin: 0 0 0.2rem;
    padding-bottom: 0.3rem;
    border-bottom: 1px solid var(--rule);
  }

  .lane-title {
    font-weight: 600;
  }

  .lane-status {
    color: var(--muted);
    font-size: var(--text-xs);
  }

  .lane-status.done {
    color: var(--text);
  }

  /* Inside the plate, after the chapter's style for details blocks. */
  :global(.plate) details.numbers {
    margin: var(--space-3) 0 0;
    border-bottom: 0;
  }

  .numbers .table {
    width: auto;
    margin-top: var(--space-1);
  }

  .numbers .table th,
  .numbers .table td {
    padding: 0.25rem 1.5rem 0.25rem 0;
  }

  @media (max-width: 47.99rem) {
    .lanes {
      grid-template-columns: minmax(0, 1fr);
      gap: var(--space-2);
    }

    .slider {
      max-width: none;
    }
  }
</style>
