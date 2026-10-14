<!--
  Fig. 2.1: one machine in three linked views, a line in the standard text format, a transition
  table and a state diagram.

  The line is parsed by the engine (fromStandard); while it has errors, the other views keep the
  last valid machine, dimmed, and the error is shown below the line. A table cell holds one
  three-character entry of the line; editing it rewrites the line through the engine
  (fromStandard, then toStandard). Selecting a cell outlines its transition in the diagram and
  writes it in the course notation.
-->
<script lang="ts">
  import { resolve } from '$app/paths';
  import type { MachineSpecDto } from '#lib/engine/pkg/turing_wasm.js';
  import { services, type EditorServices } from '#lib/machine/services.ts';
  import { buildHash } from '#lib/share.ts';
  import type { FigureProps } from '../types.ts';
  import { errorText, inView } from '../ch1/kit.ts';
  import Plate from '../ch1/Plate.svelte';
  import MiniDiagram from './MiniDiagram.svelte';

  let { id }: FigureProps = $props();
  const uid = $props.id();

  /** The two-state champion that chapter 2 describes in words. */
  const ORIGINAL = '1RB1LB_1LA1RZ';
  const ENTRY = /^([0-9][LR][A-Z]|---)$/;

  let svc = $state.raw<EditorServices | null>(null);
  let spec = $state.raw<MachineSpecDto | null>(null);
  let text = $state(ORIGINAL);
  let problem = $state<string | null>(null);
  let drafts = $state<Record<string, string>>({});
  let cellProblem = $state<{ key: string; text: string } | null>(null);
  let focus = $state<{ row: number; col: number } | null>(null);
  let failure = $state<string | null>(null);
  const visible = inView(start);

  async function start() {
    try {
      const s = await services();
      const parsed = s.fromStandard(ORIGINAL);
      if (!parsed.spec) throw new Error('the example machine does not parse');
      svc = s;
      spec = parsed.spec;
    } catch (error) {
      failure = errorText(error);
    }
  }

  /** The canonical line of the last valid machine. */
  const standard = $derived((svc && spec ? svc.toStandard(spec) : undefined) ?? ORIGINAL);
  /** Entries of the line, one row per state. */
  const rows = $derived(standard.split('_').map((row) => row.match(/.{3}/g) ?? []));
  const symbols = $derived(rows[0]?.length ?? 0);
  const letter = (i: number) => String.fromCharCode(65 + i);
  const key = (row: number, col: number) => `${row}:${col}`;
  const stale = $derived(problem !== null);
  const changed = $derived(text.trim() !== ORIGINAL || standard !== ORIGINAL);
  const playground = $derived(
    `${resolve('/playground')}${buildHash({ machine: { kind: 'standard', text: standard } })}`
  );

  function transitionIndex(row: number, col: number): number | null {
    if (!spec) return null;
    const from = letter(row);
    const i = spec.transitions.findIndex((t) => t.from === from && t.read[0] === String(col));
    return i < 0 ? null : i;
  }

  const highlight = $derived(focus ? transitionIndex(focus.row, focus.col) : null);

  function course(row: number, col: number): string {
    const i = transitionIndex(row, col);
    const t = i === null ? undefined : spec?.transitions[i];
    const lhs = `δ(${letter(row)}, ${col})`;
    if (!t) return `${lhs} is undefined; the machine halts there`;
    return `${lhs} = (${t.to}, ${t.write[0]}, ${t.dir[0]})`;
  }

  function editText(value: string) {
    text = value;
    if (!svc) return;
    const parsed = svc.fromStandard(value.trim());
    if (parsed.spec) {
      spec = parsed.spec;
      problem = null;
      drafts = {};
      cellProblem = null;
    } else {
      const d = parsed.diagnostics[0];
      problem = d
        ? `${d.message.replace(/^invalid standard format: /, '')}.`
        : 'This is not a machine in the standard format.';
      problem = problem.charAt(0).toUpperCase() + problem.slice(1);
    }
  }

  function draft(row: number, col: number, value: string) {
    drafts = { ...drafts, [key(row, col)]: value };
    if (cellProblem?.key === key(row, col)) cellProblem = null;
  }

  /** Writes the draft of a cell into the line; false if the entry is not valid. */
  function commit(row: number, col: number): boolean {
    const k = key(row, col);
    const value = drafts[k];
    if (value === undefined || !svc) return true;
    const entry = value.trim().toUpperCase();
    if (!ENTRY.test(entry)) {
      cellProblem = {
        key: k,
        text: 'Write the symbol to write, L or R and the next state, as in 1RB, or --- for none.'
      };
      return false;
    }
    const next = rows.map((r, i) => r.map((c, j) => (i === row && j === col ? entry : c)).join(''));
    const parsed = svc.fromStandard(next.join('_'));
    if (!parsed.spec) {
      cellProblem = { key: k, text: parsed.diagnostics[0]?.message ?? 'This entry is not valid.' };
      return false;
    }
    const { [k]: _done, ...rest } = drafts;
    drafts = rest;
    cellProblem = null;
    spec = parsed.spec;
    problem = null;
    text = svc.toStandard(parsed.spec) ?? next.join('_');
    return true;
  }

  function cancel(row: number, col: number) {
    const { [key(row, col)]: _gone, ...rest } = drafts;
    drafts = rest;
    if (cellProblem?.key === key(row, col)) cellProblem = null;
  }

  function cellKeydown(event: KeyboardEvent, row: number, col: number) {
    const input = event.currentTarget as HTMLInputElement;
    if (event.key === 'Enter') {
      event.preventDefault();
      if (commit(row, col)) input.select();
    } else if (event.key === 'Escape') {
      event.preventDefault();
      cancel(row, col);
      requestAnimationFrame(() => input.select());
    }
  }

  function reset() {
    editText(ORIGINAL);
    focus = null;
  }

  const message = $derived(
    cellProblem?.text ??
      (focus && rows[focus.row]?.[focus.col] !== undefined
        ? `${rows[focus.row]![focus.col]} in row ${letter(focus.row)}, column ${focus.col}: ${course(focus.row, focus.col)}.`
        : null)
  );
</script>

<Plate {id} busy={!spec && !failure} attach={visible}>
  <div class="line">
    <label class="fig-label" for="{uid}-line">Standard format</label>
    <div class="line-row">
      <input
        id="{uid}-line"
        class="fig-field standard"
        value={text}
        autocomplete="off"
        spellcheck="false"
        autocapitalize="characters"
        disabled={!svc}
        aria-invalid={problem ? 'true' : undefined}
        aria-describedby="{uid}-line-help"
        oninput={(e) => editText(e.currentTarget.value)}
      />
      <button class="button quiet" type="button" onclick={reset} disabled={!changed}>Reset</button>
      <a class="open" href={playground} rel="external">Open in the playground</a>
    </div>
    <p id="{uid}-line-help" class="fig-hint" class:problem aria-live="polite">
      {#if failure}
        The engine could not read this machine: {failure}
      {:else if problem}
        {problem} The table and the diagram show the last valid machine.
      {:else}
        One block of three characters per state and symbol: the symbol to write, the move and the
        next state.
      {/if}
    </p>
  </div>

  <div class="views">
    <div class="view diagram">
      <p class="fig-label">State diagram</p>
      {#if svc && spec}
        <MiniDiagram {svc} {spec} {highlight} {stale} />
      {:else}
        <div class="empty"></div>
      {/if}
    </div>

    <div class="view table" class:stale>
      <p class="fig-label" id="{uid}-table">Transition table</p>
      <table aria-labelledby="{uid}-table">
        <thead>
          <tr>
            <th scope="col" class="corner"
              ><span aria-hidden="true">δ</span><span class="visually-hidden">State</span></th
            >
            {#each { length: symbols } as _, col (col)}
              <th scope="col" class="mono">{col}</th>
            {/each}
          </tr>
        </thead>
        <tbody>
          {#each rows as row, r (r)}
            <tr class:current={focus?.row === r}>
              <th scope="row" class="mono state"
                ><span class="start" aria-hidden="true">{r === 0 ? '→' : ''}</span>{letter(r)}</th
              >
              {#each row as entry, c (c)}
                {@const k = key(r, c)}
                <td
                  class:on={focus?.row === r && focus.col === c}
                  class:invalid={cellProblem?.key === k}
                >
                  <input
                    class="cell mono"
                    value={drafts[k] ?? entry}
                    maxlength="3"
                    autocomplete="off"
                    spellcheck="false"
                    autocapitalize="characters"
                    disabled={!svc}
                    aria-label="Entry for state {letter(r)} reading {c}"
                    aria-invalid={cellProblem?.key === k ? 'true' : undefined}
                    aria-describedby="{uid}-cell-help"
                    onfocus={(e) => {
                      focus = { row: r, col: c };
                      e.currentTarget.select();
                    }}
                    onblur={() => {
                      commit(r, c);
                      focus = null;
                    }}
                    oninput={(e) => draft(r, c, e.currentTarget.value)}
                    onkeydown={(e) => cellKeydown(e, r, c)}
                  />
                </td>
              {/each}
            </tr>
          {/each}
        </tbody>
      </table>
      <p
        id="{uid}-cell-help"
        class="fig-hint cell-help"
        class:problem={cellProblem}
        aria-live="polite"
      >
        {#if message}
          <span class="mono">{message}</span>
        {:else}
          Select an entry to see it in the notation of chapter 1.
        {/if}
      </p>
    </div>
  </div>
</Plate>

<style>
  .line-row {
    display: flex;
    flex-wrap: wrap;
    align-items: center;
    gap: var(--space-1) 0.75rem;
  }

  .standard {
    flex: 0 1 22rem;
    width: 22rem;
    letter-spacing: 0.04em;
  }

  .button.quiet {
    min-height: 2.25rem;
    padding: 0 0.8rem;
    border-color: var(--rule);
  }

  .button.quiet:hover:not(:disabled) {
    border-color: var(--accent);
  }

  .open {
    margin-left: auto;
    font-size: var(--text-sm);
  }

  .line .fig-hint {
    min-height: 1.2rem;
    margin: 0.4rem 0 0;
    max-width: 38rem;
  }

  .views {
    display: grid;
    grid-template-columns: minmax(0, 1.25fr) minmax(0, 1fr);
    gap: var(--space-3);
    margin-top: var(--space-3);
    padding-top: var(--space-2);
    border-top: 1px solid var(--rule);
  }

  .view {
    min-width: 0;
  }

  .empty {
    min-height: 8rem;
  }

  .table.stale table {
    opacity: 0.45;
  }

  table {
    border-collapse: collapse;
    font-size: var(--text-sm);
  }

  th,
  td {
    height: 2.25rem;
    padding: 0;
    vertical-align: middle;
    border: 1px solid var(--rule);
  }

  thead th {
    border-top: 0;
    border-bottom-color: var(--rule-strong);
    font-weight: 400;
    color: var(--text);
    text-align: center;
  }

  th:first-child {
    border-left: 0;
  }

  td:last-child,
  th:last-child {
    border-right: 0;
  }

  .corner {
    width: 3rem;
    padding-left: 0.2rem;
    text-align: left;
    color: var(--muted);
    font-family: var(--font-mono);
    border-right-color: var(--rule-strong);
  }

  .state {
    padding: 0 0.6rem 0 0.2rem;
    white-space: nowrap;
    font-weight: 400;
    text-align: left;
    border-right-color: var(--rule-strong);
    font-size: var(--text-sm);
  }

  .start {
    display: inline-block;
    width: 1.3rem;
    color: var(--muted);
    font-family: var(--font-sans);
  }

  .current .state {
    color: var(--accent);
  }

  .cell {
    display: block;
    width: 4.25rem;
    height: 2.25rem;
    padding: 0;
    border: 0;
    background: transparent;
    color: var(--text);
    font-size: var(--text-sm);
    text-align: center;
    text-transform: uppercase;
    letter-spacing: 0.04em;
  }

  .cell:focus-visible {
    outline: none;
  }

  td.on {
    outline: 2px solid var(--accent);
    outline-offset: -2px;
  }

  td.on .cell {
    color: var(--accent);
  }

  td.invalid .cell {
    text-decoration: underline wavy;
    text-decoration-thickness: 1px;
    text-underline-offset: 0.3em;
  }

  .cell-help {
    min-height: 2.6rem;
    margin: var(--space-1) 0 0;
  }

  .cell-help .mono {
    font-size: var(--text-xs);
    color: var(--text);
  }

  @media (max-width: 47.99rem) {
    .standard {
      flex: 1 1 12rem;
      width: auto;
    }

    .open {
      margin-left: 0;
    }

    .views {
      grid-template-columns: minmax(0, 1fr);
      gap: var(--space-2);
    }

    .diagram {
      order: 2;
    }
  }
</style>
