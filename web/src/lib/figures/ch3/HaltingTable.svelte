<!--
  Fig. 3.1: small machines (rows) on the inputs x^n for n = 1 to 5 (columns), each run for at
  most a step budget chosen on a logarithmic slider, the one interaction of the figure.

  When the figure scrolls into view, the engine runs every cell once up to the largest budget and
  records whether the machine halted (and at which step), was proved never to halt (and from
  which step the proof holds), or was still running. A run with a smaller budget B sees exactly
  the first B steps of the same deterministic run, so each cell shows "halted at t" if t <= B,
  "never halts" with the engine's proof if the proof holds by step B, and "still running"
  otherwise. Non-halting is claimed only where the engine proved it.
-->
<script lang="ts">
  import type { MachineSpecDto } from '#lib/engine/pkg/turing_wasm.js';
  import { count } from '#lib/machine/describe.ts';
  import { SessionClient } from '#lib/machine/session.ts';
  import type { FigureProps } from '../types.ts';
  import { errorText, gallerySpec, inView, scrollRegion, sourceSpec } from '../ch1/kit.ts';
  import Plate from '../ch1/Plate.svelte';
  import { BUDGETS, INPUTS, ROWS, type HaltingRow } from './machines.ts';

  let { id }: FigureProps = $props();
  const uid = $props.id();

  const MAX = BUDGETS.at(-1)!;
  const NONE = { transitions: [], states: [] };

  type Outcome =
    | { kind: 'halted'; at: number }
    | { kind: 'never'; from: number; reason: string }
    | { kind: 'running' };

  let index = $state(1);
  let outcomes = $state.raw<Outcome[][] | null>(null);
  let started = $state(false);
  let failure = $state<string | null>(null);
  const visible = inView(start);
  const wide = scrollRegion('Halting table, scrolls sideways');

  const budget = $derived(BUDGETS[index] ?? MAX);

  async function specOf(row: HaltingRow): Promise<MachineSpecDto> {
    if (row.gallery) return gallerySpec(row.gallery);
    return sourceSpec(row.source ?? '');
  }

  /** The first step at which the session reports a proof of non-halting. */
  async function proofStep(session: SessionClient, last: number): Promise<number> {
    let lo = 0;
    let hi = last;
    while (lo < hi) {
      const mid = Math.floor((lo + hi) / 2);
      const s = await session.seek(mid, 0);
      if (s?.status === 'never-halts') hi = mid;
      else lo = mid + 1;
    }
    return lo;
  }

  async function runCell(session: SessionClient, spec: MachineSpecDto, input: string) {
    await session.open(spec, input, 0);
    const outcome = await session.run(MAX, NONE, 0);
    if (!outcome) throw new Error('the run was interrupted');
    const s = outcome.result.snapshot;
    if (s.status === 'halted') return { kind: 'halted', at: s.step } as const;
    if (s.status === 'never-halts') {
      return {
        kind: 'never',
        from: await proofStep(session, s.step),
        reason: s.reason ?? 'the engine proved it'
      } as const;
    }
    return { kind: 'running' } as const;
  }

  let session: SessionClient | null = null;

  async function start() {
    started = true;
    const own = new SessionClient();
    session = own;
    try {
      const specs = await Promise.all(ROWS.map(specOf));
      const table: Outcome[][] = [];
      for (const spec of specs) {
        const row: Outcome[] = [];
        for (const input of INPUTS) {
          // Stop if the figure was removed meanwhile.
          if (session !== own) return;
          row.push(await runCell(own, spec, input));
        }
        table.push(row);
      }
      outcomes = table;
    } catch (error) {
      if (session === own) failure = errorText(error);
    } finally {
      own.close();
    }
  }

  $effect(() => () => {
    session?.close();
    session = null;
  });

  /** The proofs that occur, numbered as notes below the table. */
  const notes = $derived.by(() => {
    const reasons: string[] = [];
    for (const row of outcomes ?? []) {
      for (const cell of row) {
        if (cell.kind === 'never' && cell.from <= budget && !reasons.includes(cell.reason)) {
          reasons.push(cell.reason);
        }
      }
    }
    return reasons;
  });

  function shown(cell: Outcome): 'halted' | 'never' | 'running' {
    if (cell.kind === 'halted' && cell.at <= budget) return 'halted';
    if (cell.kind === 'never' && cell.from <= budget) return 'never';
    return 'running';
  }

  const tally = $derived.by(() => {
    const t = { halted: 0, never: 0, running: 0 };
    for (const row of outcomes ?? []) for (const cell of row) t[shown(cell)]++;
    return t;
  });

  const runs = (k: number) => (k === 0 ? 'no run' : k === 1 ? 'one run' : `${k} runs`);
  const budgetText = $derived(`${count(budget)} ${budget === 1 ? 'step' : 'steps'}`);
  const exponent = (b: number) => Math.round(Math.log10(b));
  const decades = BUDGETS.map((b, i) => ({ b, i })).filter(({ b }) => 10 ** exponent(b) === b);
</script>

<Plate {id} busy={!outcomes && !failure} attach={visible}>
  <div class="budget">
    <label class="fig-label" for="{uid}-budget">Step budget</label>
    <div class="slider-row">
      <div class="slider">
        <input
          id="{uid}-budget"
          type="range"
          min="0"
          max={BUDGETS.length - 1}
          step="1"
          bind:value={index}
          aria-valuetext={budgetText}
          disabled={!outcomes}
        />
        <div class="ticks" aria-hidden="true">
          {#each decades as { b, i } (b)}
            <span
              class="tick mono"
              class:first={i === 0}
              class:last={i === BUDGETS.length - 1}
              style:left="{(i / (BUDGETS.length - 1)) * 100}%">10<sup>{exponent(b)}</sup></span
            >
          {/each}
        </div>
      </div>
      <output class="value mono" for="{uid}-budget">{budgetText}</output>
    </div>
  </div>

  <div class="scroll" {@attach wide}>
    <table>
      <caption id="{uid}-caption" class="visually-hidden">
        Machines on the inputs x to xxxxx, each run for at most {budgetText}
      </caption>
      <thead>
        <tr>
          <th scope="col" class="corner"><span class="fig-label">Machine</span></th>
          {#each INPUTS as input (input)}
            <th scope="col" class="input">
              <span class="fig-label">n = {input.length}</span>
              <span class="mono word">{input}</span>
            </th>
          {/each}
        </tr>
      </thead>
      <tbody>
        {#each ROWS as row, r (row.key)}
          <tr>
            <th scope="row" class="machine">
              <span class="name"
                >{row.name}{#if row.sub}<sub>{row.sub}</sub>{/if}</span
              >
              <span class="note"
                >{#each row.note.split('`') as part, k (k)}{#if k % 2 === 1}<span class="mono"
                      >{part}</span
                    >{:else}{part}{/if}{/each}</span
              >
            </th>
            {#each INPUTS as input, c (input)}
              {@const cell = outcomes?.[r]?.[c]}
              {@const state = cell ? shown(cell) : null}
              <td class={state ?? 'pending'}>
                {#if !cell}
                  <span class="visually-hidden">{failure ? '' : 'not computed yet'}</span>
                {:else if state === 'halted' && cell.kind === 'halted'}
                  halted at <span class="mono">{count(cell.at)}</span>
                {:else if state === 'never' && cell.kind === 'never'}
                  never halts<sup class="mark mono" aria-hidden="true"
                    >{notes.indexOf(cell.reason) + 1}</sup
                  ><span class="visually-hidden">, proved: {cell.reason}</span>
                {:else}
                  still running
                {/if}
              </td>
            {/each}
          </tr>
        {/each}
      </tbody>
    </table>
  </div>

  {#if notes.length > 0}
    <ol class="notes">
      {#each notes as reason, i (reason)}
        <li>
          <span class="mark mono" aria-hidden="true">{i + 1}</span>
          Never halts, proved by the simulator: the machine {reason}.
        </li>
      {/each}
    </ol>
  {/if}

  <p class="fig-status summary" aria-live="polite">
    {#if failure}
      The engine could not run these machines: {failure}
    {:else if outcomes}
      With a budget of {budgetText}, {runs(tally.halted)} halted, {runs(tally.never)}
      {tally.never === 1 ? 'was' : 'were'} proved never to halt and {runs(tally.running)}
      {tally.running === 1 ? 'is' : 'are'} still running.
    {:else if started}
      Running the machines.
    {/if}
  </p>
</Plate>

<style>
  .budget {
    margin-bottom: var(--space-3);
  }

  .slider-row {
    display: flex;
    align-items: flex-start;
    gap: var(--space-2);
  }

  .slider {
    position: relative;
    flex: 1 1 auto;
    max-width: 30rem;
    padding-bottom: 1.1rem;
  }

  .slider input {
    display: block;
    width: 100%;
  }

  .ticks {
    position: absolute;
    left: 0.375rem;
    right: 0.375rem;
    bottom: 0;
    height: 1rem;
  }

  .tick.first {
    transform: translateX(-0.375rem);
  }

  .tick.last {
    transform: translateX(calc(-100% + 0.375rem));
  }

  .tick sup {
    font-size: 0.8em;
    line-height: 0;
  }

  .note .mono {
    font-size: 1em;
  }

  .tick {
    position: absolute;
    top: 0;
    transform: translateX(-50%);
    font-size: 0.6875rem;
    color: var(--muted);
    white-space: nowrap;
  }

  .value {
    min-width: 9rem;
    line-height: 2.25rem;
    font-size: var(--text-sm);
  }

  .scroll {
    overflow-x: auto;
  }

  .scroll:focus-visible {
    outline: 2px solid var(--accent);
    outline-offset: 2px;
  }

  table {
    width: 100%;
    border-collapse: collapse;
    font-size: var(--text-sm);
    line-height: 1.35;
  }

  th,
  td {
    padding: 0.6rem 0.9rem 0.6rem 0;
    border-bottom: 1px solid var(--rule);
    text-align: left;
    vertical-align: top;
  }

  th:last-child,
  td:last-child {
    padding-right: 0;
  }

  thead th {
    border-bottom-color: var(--rule-strong);
    font-weight: 400;
    vertical-align: bottom;
  }

  thead .fig-label {
    margin: 0 0 0.1rem;
  }

  .word {
    font-size: var(--text-sm);
    letter-spacing: 0.04em;
  }

  .machine {
    width: 32%;
    min-width: 11rem;
    font-weight: 400;
  }

  .name {
    display: block;
    font-weight: 600;
  }

  .name sub {
    font-size: 0.72em;
  }

  .note {
    display: block;
    margin-top: 0.15rem;
    font-size: var(--text-xs);
    color: var(--muted);
  }

  td {
    white-space: nowrap;
  }

  td .mono {
    font-size: 0.92em;
  }

  td.running,
  td.pending {
    color: var(--muted);
  }

  .mark {
    margin-left: 0.1em;
    font-size: 0.7em;
    line-height: 0;
    color: var(--muted);
  }

  .notes {
    margin: var(--space-2) 0 0;
    padding: 0;
    list-style: none;
    font-size: var(--text-xs);
    color: var(--muted);
  }

  .notes .mark {
    display: inline-block;
    width: 1rem;
    margin: 0;
    font-size: inherit;
  }

  .summary {
    margin-top: var(--space-2);
  }

  @media (max-width: 47.99rem) {
    .slider-row {
      flex-direction: column;
      gap: 0;
    }

    .slider {
      width: 100%;
      max-width: none;
    }

    .value {
      line-height: 1.6;
    }

    .machine,
    .corner {
      position: sticky;
      left: 0;
      z-index: 1;
      min-width: 8.5rem;
      width: 8.5rem;
      background: var(--bg);
    }

    .machine {
      padding-right: 0.75rem;
    }
  }
</style>
