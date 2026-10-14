<!--
  The transition table of a single-tape machine: one row per state, one column per symbol of Γ.
  A cell holds δ(q, a) as "q2, X, R" (or "1RB" for busy beaver machines) and is edited in place:
  type to replace, Enter or F2 to edit, Enter to save, Escape to cancel, arrow keys to move.
  Row and column headers open menus for states and symbols.
-->
<script lang="ts">
  import { tick } from 'svelte';
  import type { DirDto } from '#lib/engine/pkg/turing_wasm.js';
  import { glyph, parseCell, showCell } from './cell';
  import { findTransition, stateNameProblem } from './edit';
  import Menu from './Menu.svelte';
  import { stateMenu, symbolMenu } from './menus';
  import type { Playground } from './state.svelte';

  interface Props {
    pg: Playground;
  }

  let { pg }: Props = $props();

  const spec = $derived(pg.spec!);
  const compact = $derived(spec.kind === 'beaver');
  const states = $derived(spec.states);
  const symbols = $derived(spec.tapeAlphabet);
  const next = $derived.by(() => {
    const i = pg.snapshot?.nextTransition;
    return i === undefined ? undefined : spec.transitions[i];
  });
  const current = $derived(pg.snapshot?.state ?? null);

  let root: HTMLElement | null = $state(null);
  let active = $state({ r: 0, c: 0 });
  let editing = $state<string | null>(null);
  let drafts = $state<Record<string, string>>({});
  let problem = $state<{ key: string; text: string } | null>(null);
  let confirm = $state<{
    key: string;
    from: string;
    read: string;
    to: string;
    write: string;
    dir: DirDto;
    text: string;
  } | null>(null);
  let pointerFocus = false;

  // Menus.
  let menu = $state<{ kind: 'state' | 'symbol'; name: string; anchor: HTMLElement } | null>(null);
  let renaming = $state<{ name: string; draft: string } | null>(null);
  let addingSymbol = $state(false);
  let symbolDraft = $state('');

  const key = (r: number, c: number) => `${r}:${c}`;

  function transitionAt(state: string, read: string) {
    const i = findTransition(spec, state, [read]);
    return i < 0 ? undefined : spec.transitions[i];
  }

  function focusAt(r: number, c: number, select = true): boolean {
    const el = root?.querySelector<HTMLElement>(`[data-r="${r}"][data-c="${c}"]`);
    if (!el) return false;
    el.focus();
    if (select && el instanceof HTMLInputElement) el.select();
    return true;
  }

  function move(r: number, c: number) {
    if (focusAt(r, c)) return;
    // Rows without that column (the add-state row) fall back to the row header.
    if (focusAt(r, -1)) return;
  }

  function onfocus(event: FocusEvent, r: number, c: number) {
    active = { r, c };
    const input = event.currentTarget as HTMLInputElement;
    if (pointerFocus) {
      editing = key(r, c);
    } else {
      editing = null;
      input.select();
    }
    pointerFocus = false;
    const state = states[r];
    if (state) pg.selection = { kind: 'state', name: state.name };
  }

  function headerFocus(r: number, c: number) {
    active = { r, c };
  }

  function caretToEnd(input: HTMLInputElement) {
    const n = input.value.length;
    input.setSelectionRange(n, n);
  }

  function revert(k: string) {
    const { [k]: _gone, ...rest } = drafts;
    drafts = rest;
    if (problem?.key === k) problem = null;
    if (confirm?.key === k) confirm = null;
  }

  /** Saves the draft of cell (r, c); false if it needs an answer first. */
  function commit(r: number, c: number): boolean {
    const k = key(r, c);
    const draft = drafts[k];
    const state = states[r]?.name;
    const read = symbols[c];
    if (draft === undefined || state === undefined || read === undefined) return true;
    const shown = showCell(transitionAt(state, read), compact);
    if (draft === shown) {
      revert(k);
      editing = null;
      return true;
    }
    const result = parseCell(draft, {
      states: states.map((s) => s.name),
      gamma: symbols,
      blank: spec.blank,
      read,
      compact
    });
    if (result.kind === 'error') {
      problem = { key: k, text: result.message };
      return false;
    }
    if (result.kind === 'empty') {
      revert(k);
      editing = null;
      pg.setTransition(state, [read], null);
      return true;
    }
    const { to, write, dir, missing } = result;
    if (missing.state && stateNameProblem(spec, missing.state)) {
      problem = { key: k, text: stateNameProblem(spec, missing.state)! };
      return false;
    }
    if (missing.state || missing.symbol) {
      const what =
        missing.state && missing.symbol
          ? `state ${missing.state} and no symbol ${missing.symbol}`
          : missing.state
            ? `state ${missing.state}`
            : `symbol ${missing.symbol}`;
      confirm = { key: k, from: state, read, to, write, dir, text: `There is no ${what} yet.` };
      problem = null;
      void tick().then(() => root?.querySelector<HTMLElement>('[data-confirm]')?.focus());
      return false;
    }
    revert(k);
    editing = null;
    pg.setTransition(state, [read], { to, write: [write], dir: [dir] });
    return true;
  }

  function accept() {
    if (!confirm) return;
    const { key: k, from, read, to, write, dir } = confirm;
    const [r, c] = k.split(':').map(Number) as [number, number];
    revert(k);
    editing = null;
    pg.setTransition(from, [read], { to, write: [write], dir: [dir] });
    void tick().then(() => focusAt(r, c));
  }

  function decline() {
    if (!confirm) return;
    const [r, c] = confirm.key.split(':').map(Number) as [number, number];
    confirm = null;
    void tick().then(() => focusAt(r, c, false));
  }

  function cellKeydown(event: KeyboardEvent, r: number, c: number) {
    const k = key(r, c);
    const input = event.currentTarget as HTMLInputElement;
    const navigating = editing !== k;
    const dirty = drafts[k] !== undefined;
    switch (event.key) {
      case 'Enter':
        event.preventDefault();
        if (navigating && !dirty) {
          editing = k;
          caretToEnd(input);
        } else if (confirm?.key === k) {
          accept();
        } else if (commit(r, c)) {
          void tick().then(() => focusAt(r, c));
        }
        return;
      case 'F2':
        event.preventDefault();
        editing = k;
        caretToEnd(input);
        return;
      case 'Escape':
        if (dirty || editing === k) {
          event.preventDefault();
          revert(k);
          editing = null;
          void tick().then(() => input.select());
        }
        return;
      case 'ArrowUp':
      case 'ArrowDown': {
        event.preventDefault();
        if (dirty && !commit(r, c)) return;
        move(r + (event.key === 'ArrowUp' ? -1 : 1), c);
        return;
      }
      case 'ArrowLeft':
      case 'ArrowRight':
        if (!navigating) return;
        event.preventDefault();
        if (dirty && !commit(r, c)) return;
        move(r, c + (event.key === 'ArrowLeft' ? -1 : 1));
        return;
      case 'Home':
      case 'End':
        if (!navigating) return;
        event.preventDefault();
        move(r, event.key === 'Home' ? -1 : symbols.length - 1);
        return;
    }
  }

  function headerKeydown(event: KeyboardEvent, r: number, c: number) {
    const moves: Record<string, [number, number]> = {
      ArrowUp: [r - 1, c],
      ArrowDown: [r + 1, c],
      ArrowLeft: [r, c - 1],
      ArrowRight: [r, c + 1]
    };
    const to = moves[event.key];
    if (!to) return;
    event.preventDefault();
    if (!focusAt(to[0], to[1]) && to[0] !== r) focusAt(to[0], -1);
  }

  function oninput(event: Event, r: number, c: number) {
    const k = key(r, c);
    drafts = { ...drafts, [k]: (event.currentTarget as HTMLInputElement).value };
    editing = k;
    if (problem?.key === k) problem = null;
    if (confirm?.key === k) confirm = null;
  }

  function onblur(event: FocusEvent, r: number, c: number) {
    const k = key(r, c);
    const to = event.relatedTarget as HTMLElement | null;
    if (to?.closest('[data-message]')) return;
    if (drafts[k] !== undefined && confirm?.key !== k) commit(r, c);
    if (editing === k) editing = null;
  }

  function menuItems(kind: 'state' | 'symbol', name: string) {
    return kind === 'state'
      ? stateMenu(pg, name, { rename: () => startRename(name) })
      : symbolMenu(pg, name);
  }

  function startRename(name: string) {
    renaming = { name, draft: name };
    void tick().then(() => {
      const el = root?.querySelector<HTMLInputElement>('[data-rename]');
      el?.focus();
      el?.select();
    });
  }

  const renameProblem = $derived(
    renaming ? stateNameProblem(spec, renaming.draft.trim(), renaming.name) : null
  );

  function finishRename(save: boolean) {
    if (!renaming) return;
    const { name, draft } = renaming;
    const r = states.findIndex((s) => s.name === name);
    if (save && draft.trim() !== name) {
      if (renameProblem) return;
      pg.renameState(name, draft.trim());
    }
    renaming = null;
    void tick().then(() => focusAt(r, -1));
  }

  function startSymbol() {
    addingSymbol = true;
    symbolDraft = '';
    void tick().then(() => root?.querySelector<HTMLInputElement>('[data-symbol]')?.focus());
  }

  let symbolProblem = $state<string | null>(null);

  function finishSymbol(save: boolean) {
    if (save && symbolDraft.trim() !== '') {
      const problem = pg.addSymbol(symbolDraft.trim(), true);
      if (problem) {
        symbolProblem = problem;
        return;
      }
    }
    addingSymbol = false;
    symbolProblem = null;
    void tick().then(() => focusAt(-1, symbols.length));
  }

  function addState() {
    const name = pg.addState(undefined, undefined, 'table');
    if (!name) return;
    void tick().then(() => focusAt(states.length - 1, 0));
  }

  const message = $derived(
    confirm ? null : (problem?.text ?? (addingSymbol ? symbolProblem : null) ?? renameProblem)
  );
  const empty = $derived(spec.transitions.length === 0);
</script>

<div class="wrap" bind:this={root}>
  <div class="scroll">
    <table role="grid" aria-label="Transition table" aria-describedby="table-message">
      <thead>
        <tr>
          <th class="corner" scope="col"><span class="visually-hidden">State</span>δ</th>
          {#each symbols as g, c (g)}
            <th scope="col" class="symbol">
              <button
                type="button"
                class="head-button mono"
                data-r={-1}
                data-c={c}
                tabindex={active.r === -1 && active.c === c ? 0 : -1}
                aria-haspopup="menu"
                aria-label={`Symbol ${g === spec.blank ? 'blank' : g}${spec.inputAlphabet.includes(g) ? ', in the input alphabet' : ''}`}
                onfocus={() => headerFocus(-1, c)}
                onkeydown={(e) => headerKeydown(e, -1, c)}
                onclick={(e) => (menu = { kind: 'symbol', name: g, anchor: e.currentTarget })}
                >{glyph(g)}</button
              >
            </th>
          {/each}
          <th scope="col" class="add-col">
            {#if addingSymbol}
              <input
                class="symbol-input mono"
                data-symbol
                aria-label="New symbol"
                maxlength="2"
                bind:value={symbolDraft}
                onkeydown={(e) => {
                  if (e.key === 'Enter') {
                    e.preventDefault();
                    finishSymbol(true);
                  } else if (e.key === 'Escape') {
                    e.preventDefault();
                    finishSymbol(false);
                  }
                }}
                onblur={() => finishSymbol(symbolDraft.trim() !== '')}
              />
            {:else}
              <button
                type="button"
                class="head-button add"
                data-r={-1}
                data-c={symbols.length}
                tabindex={active.r === -1 && active.c === symbols.length ? 0 : -1}
                onfocus={() => headerFocus(-1, symbols.length)}
                onkeydown={(e) => headerKeydown(e, -1, symbols.length)}
                onclick={startSymbol}>+ Symbol</button
              >
            {/if}
          </th>
        </tr>
      </thead>
      <tbody>
        {#each states as s, r (s.name)}
          <tr
            class:current={current === s.name}
            class:selected={pg.selection?.kind === 'state' && pg.selection.name === s.name}
          >
            <th scope="row" class="state">
              {#if renaming?.name === s.name}
                <input
                  class="rename mono"
                  data-rename
                  aria-label={`New name for ${s.name}`}
                  aria-invalid={renameProblem ? 'true' : undefined}
                  bind:value={renaming.draft}
                  onkeydown={(e) => {
                    if (e.key === 'Enter') {
                      e.preventDefault();
                      finishRename(true);
                    } else if (e.key === 'Escape') {
                      e.preventDefault();
                      finishRename(false);
                    }
                  }}
                  onblur={() => finishRename(!renameProblem)}
                />
              {:else}
                <button
                  type="button"
                  class="head-button state-button mono"
                  data-r={r}
                  data-c={-1}
                  tabindex={active.r === r && active.c === -1 ? 0 : -1}
                  aria-haspopup="menu"
                  aria-label={`State ${s.name}${spec.start === s.name ? ', start' : ''}${s.isFinal ? ', final' : ''}${current === s.name ? ', current' : ''}`}
                  onfocus={() => headerFocus(r, -1)}
                  onkeydown={(e) => headerKeydown(e, r, -1)}
                  onclick={(e) => (menu = { kind: 'state', name: s.name, anchor: e.currentTarget })}
                >
                  <span class="arrow" aria-hidden="true">{spec.start === s.name ? '→' : ''}</span>
                  <span class="name" class:final={s.isFinal}>{s.name}</span>
                  {#if pg.hasBreakpoint({ kind: 'state', state: s.name })}<span
                      class="bp"
                      aria-hidden="true"
                    ></span>{/if}
                </button>
              {/if}
            </th>
            {#each symbols as g, c (g)}
              {@const k = key(r, c)}
              {@const t = transitionAt(s.name, g)}
              {@const isNext = next !== undefined && next.from === s.name && next.read[0] === g}
              <td
                class:next={isNext}
                class:invalid={problem?.key === k}
                class:breakpoint={pg.hasBreakpoint({ kind: 'transition', from: s.name, read: [g] })}
              >
                <input
                  class="cell mono"
                  class:compact
                  data-r={r}
                  data-c={c}
                  tabindex={active.r === r && active.c === c ? 0 : -1}
                  value={drafts[k] ?? showCell(t, compact)}
                  placeholder={empty && r === 0 && c === 0
                    ? compact
                      ? '1RB'
                      : `${s.name}, ${glyph(g)}, R`
                    : ''}
                  aria-label={`δ(${s.name}, ${g === spec.blank ? 'blank' : g})${isNext ? ', applied next' : ''}`}
                  aria-invalid={problem?.key === k ? 'true' : undefined}
                  autocomplete="off"
                  spellcheck="false"
                  autocapitalize="off"
                  onpointerdown={() => (pointerFocus = true)}
                  onfocus={(e) => onfocus(e, r, c)}
                  oninput={(e) => oninput(e, r, c)}
                  onkeydown={(e) => cellKeydown(e, r, c)}
                  onblur={(e) => onblur(e, r, c)}
                />
              </td>
            {/each}
            <td class="filler"></td>
          </tr>
        {/each}
        <tr class="add-row">
          <th scope="row" class="state">
            <button
              type="button"
              class="head-button add"
              data-r={states.length}
              data-c={-1}
              tabindex={active.r === states.length ? 0 : -1}
              onfocus={() => headerFocus(states.length, -1)}
              onkeydown={(e) => headerKeydown(e, states.length, -1)}
              onclick={addState}>+ State</button
            >
          </th>
          <td colspan={symbols.length}></td>
          <td class="filler"></td>
        </tr>
      </tbody>
    </table>
  </div>

  <div class="message" id="table-message" data-message aria-live="polite">
    {#if confirm}
      <span>{confirm.text}</span>
      <span class="actions">
        <button type="button" class="button small" data-confirm onclick={accept}
          >Add and save</button
        >
        <button type="button" class="button small quiet" onclick={decline}>Edit</button>
      </span>
    {:else if message}
      <span>{message}</span>
    {:else}
      <span class="muted"
        >{empty
          ? `Type the next state, the symbol to write and a move into a cell, as in ${compact ? '1RB' : 'q1, 1, R'}.`
          : 'Enter edits a cell, Escape cancels, arrow keys move. Headers open menus.'}</span
      >
    {/if}
  </div>
</div>

{#if menu}
  <Menu
    open={true}
    anchor={menu.anchor}
    label={menu.kind === 'state' ? `State ${menu.name}` : `Symbol ${menu.name}`}
    items={menuItems(menu.kind, menu.name)}
    onclose={() => (menu = null)}
  />
{/if}

<style>
  .wrap {
    min-width: 0;
  }

  .scroll {
    overflow-x: auto;
    scrollbar-width: thin;
  }

  table {
    border-collapse: collapse;
    font-size: var(--text-sm);
  }

  th,
  td {
    padding: 0;
    border: 1px solid var(--rule);
    height: 2.25rem;
  }

  thead th {
    border-top: 0;
    border-bottom: 1px solid var(--rule-strong);
    font-weight: 400;
  }

  th:first-child,
  td:first-child {
    border-left: 0;
  }

  .corner {
    width: 6.5rem;
    text-align: left;
    padding-left: 0.6rem;
    color: var(--muted);
    font-family: var(--font-mono);
    border-right: 1px solid var(--rule-strong);
  }

  .state {
    border-right: 1px solid var(--rule-strong);
    text-align: left;
  }

  .add-col,
  .filler {
    border-top: 0;
    border-right: 0;
    border-bottom: 0;
  }

  .add-row th,
  .add-row td {
    border: 0;
    border-top: 1px solid var(--rule-strong);
  }

  .add-row td.filler {
    border-top: 0;
  }

  tbody tr:last-of-type th {
    border-right: 0;
  }

  .head-button {
    display: flex;
    align-items: center;
    width: 100%;
    height: 100%;
    min-height: 2.25rem;
    padding: 0 0.6rem;
    border: 0;
    background: none;
    cursor: pointer;
    font-size: var(--text-sm);
    font-weight: 400;
    white-space: nowrap;
  }

  .symbol .head-button {
    justify-content: center;
    font-size: 1rem;
  }

  .head-button:hover {
    background: color-mix(in srgb, var(--text) 6%, transparent);
  }

  .head-button:focus-visible,
  .cell:focus-visible,
  .rename:focus-visible,
  .symbol-input:focus-visible {
    outline: 2px solid var(--accent);
    outline-offset: -2px;
  }

  .add {
    color: var(--muted);
    font-family: var(--font-sans);
  }

  .add:hover {
    color: var(--text);
  }

  .state-button {
    gap: 0.25rem;
  }

  .arrow {
    width: 1rem;
    color: var(--muted);
    font-family: var(--font-sans);
  }

  .name {
    padding: 0.1rem 0.2rem;
    border: 1px solid transparent;
  }

  .name.final {
    border: 1px solid currentColor;
    outline: 1px solid currentColor;
    outline-offset: 1px;
  }

  .current .state-button {
    color: var(--accent);
  }

  .bp {
    width: 0.4rem;
    height: 0.4rem;
    margin-left: auto;
    background: var(--text);
  }

  .selected .state-button {
    background: color-mix(in srgb, var(--text) 5%, transparent);
  }

  td {
    position: relative;
  }

  .cell {
    display: block;
    width: 8.5rem;
    height: 2.25rem;
    padding: 0 0.6rem;
    border: 0;
    background: transparent;
    font-size: var(--text-sm);
    color: var(--text);
  }

  .cell.compact {
    width: 4.5rem;
    text-align: center;
  }

  .cell::placeholder {
    color: var(--muted);
    opacity: 0.8;
  }

  td.next {
    outline: 2px solid var(--accent);
    outline-offset: -2px;
  }

  td.next .cell {
    color: var(--accent);
  }

  td.invalid {
    background: color-mix(in srgb, var(--text) 8%, transparent);
  }

  td.invalid .cell {
    text-decoration: underline wavy;
    text-decoration-thickness: 1px;
    text-underline-offset: 0.3em;
  }

  td.breakpoint::after {
    content: '';
    position: absolute;
    top: 3px;
    right: 3px;
    width: 0.4rem;
    height: 0.4rem;
    background: var(--text);
    pointer-events: none;
  }

  .rename,
  .symbol-input {
    width: 100%;
    min-width: 0;
    height: 2.25rem;
    padding: 0 0.6rem;
    border: 0;
    background: var(--field);
    font-size: var(--text-sm);
  }

  .symbol-input {
    width: 5.5rem;
    text-align: center;
  }

  .message {
    display: flex;
    flex-wrap: wrap;
    align-items: center;
    gap: 0.5rem 1rem;
    min-height: 2.75rem;
    padding-top: 0.6rem;
    font-size: var(--text-sm);
  }

  .actions {
    display: flex;
    gap: 0.5rem;
  }

  .button.small {
    min-height: 2rem;
    padding: 0 0.7rem;
  }

  .button.quiet {
    border-color: var(--rule);
  }
</style>
