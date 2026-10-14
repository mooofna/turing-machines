<!--
  The transitions of a machine with several tapes, one per row: state, read tuple, next state,
  write tuple and moves, each an editable field. Enter or leaving a field saves it.
-->
<script lang="ts">
  import { tick } from 'svelte';
  import type { DirDto, TransitionDto } from '#lib/engine/pkg/turing_wasm.js';
  import { glyph } from './cell';
  import { findTransition, stateNameProblem } from './edit';
  import Menu from './Menu.svelte';
  import { stateMenu } from './menus';
  import RenameForm from './RenameForm.svelte';
  import type { Playground } from './state.svelte';

  interface Props {
    pg: Playground;
  }

  let { pg }: Props = $props();
  const id = $props.id();

  const spec = $derived(pg.spec!);
  const k = $derived(spec.tapes);
  const nextIndex = $derived(pg.snapshot?.nextTransition);

  type Field = 'from' | 'read' | 'to' | 'write' | 'dir';
  const fields: { field: Field; label: string }[] = [
    { field: 'from', label: 'State' },
    { field: 'read', label: 'Reads' },
    { field: 'to', label: 'Next state' },
    { field: 'write', label: 'Writes' },
    { field: 'dir', label: 'Moves' }
  ];

  let problem = $state<string | null>(null);
  let menu = $state<{ name: string; anchor: HTMLElement } | null>(null);
  let renaming = $state<{ name: string; anchor: HTMLElement } | null>(null);
  let draft = $state<Record<Field, string>>({ from: '', read: '', to: '', write: '', dir: '' });

  function show(t: TransitionDto, field: Field): string {
    if (field === 'from' || field === 'to') return t[field];
    const items = field === 'dir' ? t.dir : t[field].map(glyph);
    return items.join(', ');
  }

  function tuple(text: string): string[] {
    return text
      .trim()
      .replace(/^\(|\)$/g, '')
      .split(/\s*,\s*|\s+/u)
      .filter((x) => x !== '');
  }

  function symbols(text: string, read?: string[]): string[] | string {
    const items = tuple(text).map((g, i) =>
      g === '*' && read ? (read[i] ?? g) : g === '_' || g === '␣' ? spec.blank : g
    );
    if (items.length !== k)
      return `Give ${k} symbols, one per tape, as in ${Array(k).fill(spec.blank).join(', ')}.`;
    const bad = items.find((g) => Array.from(g).length !== 1);
    if (bad) return `${bad} is not a single symbol.`;
    return items;
  }

  function moves(text: string): DirDto[] | string {
    const items = tuple(text).map((d) => d.toUpperCase());
    if (items.length !== k)
      return `Give ${k} moves, one per tape, as in ${Array(k).fill('R').join(', ')}.`;
    const bad = items.find((d) => !['L', 'R', 'N'].includes(d));
    if (bad) return `${bad} is not a move. Use L, R or N.`;
    return items as DirDto[];
  }

  /** A state name, existing or new; `problem` explains why a new name is not allowed. */
  function stateName(text: string): { name: string; problem: string | null } {
    const name = text.trim();
    if (spec.states.some((s) => s.name === name)) return { name, problem: null };
    return { name, problem: stateNameProblem(spec, name) };
  }

  function save(index: number, field: Field, input: HTMLInputElement) {
    const t = spec.transitions[index];
    if (!t) return;
    if (input.value === show(t, field)) return;
    let patch: Partial<Pick<TransitionDto, Field>>;
    if (field === 'from' || field === 'to') {
      const { name, problem: issue } = stateName(input.value);
      if (issue) {
        problem = issue;
        return;
      }
      patch = { [field]: name };
    } else if (field === 'dir') {
      const dir = moves(input.value);
      if (typeof dir === 'string') {
        problem = dir;
        return;
      }
      patch = { dir };
    } else {
      const glyphs = symbols(input.value, field === 'write' ? t.read : undefined);
      if (typeof glyphs === 'string') {
        problem = glyphs;
        return;
      }
      patch = { [field]: glyphs };
    }
    const from = patch.from ?? t.from;
    const read = patch.read ?? t.read;
    const clash = findTransition(spec, from, read);
    if (clash >= 0 && clash !== index) {
      problem = `δ(${from}, (${read.join(', ')})) is already defined.`;
      return;
    }
    problem = null;
    pg.editTransition(index, patch);
  }

  function keydown(event: KeyboardEvent, index: number, field: Field) {
    const input = event.currentTarget as HTMLInputElement;
    const t = spec.transitions[index];
    if (event.key === 'Enter') {
      event.preventDefault();
      save(index, field, input);
    } else if (event.key === 'Escape' && t) {
      event.preventDefault();
      input.value = show(t, field);
      problem = null;
    }
  }

  function menuItems(name: string, anchor: HTMLElement) {
    return stateMenu(pg, name, { rename: () => (renaming = { name, anchor }) });
  }

  function add(event: SubmitEvent) {
    event.preventDefault();
    const from = stateName(draft.from || spec.start);
    const to = stateName(draft.to || spec.start);
    const read = symbols(draft.read);
    const write = symbols(draft.write, typeof read === 'string' ? undefined : read);
    const dir = moves(draft.dir);
    const issue =
      from.problem ??
      to.problem ??
      [read, write, dir].find((x): x is string => typeof x === 'string') ??
      null;
    if (issue || typeof read === 'string' || typeof write === 'string' || typeof dir === 'string') {
      problem = issue;
      return;
    }
    if (findTransition(spec, from.name, read) >= 0) {
      problem = `δ(${from.name}, (${read.join(', ')})) is already defined.`;
      return;
    }
    problem = null;
    pg.setTransition(from.name, read, { to: to.name, write, dir }, 'table');
    draft = { from: from.name, read: '', to: to.name, write: '', dir: '' };
    void tick().then(() => document.getElementById(`${id}-new-read`)?.focus());
  }
</script>

<div class="states">
  <span class="states-label">States</span>
  <ul>
    {#each spec.states as s (s.name)}
      <li>
        <button
          type="button"
          class="state mono"
          class:current={pg.snapshot?.state === s.name}
          aria-haspopup="menu"
          aria-label={`State ${s.name}${spec.start === s.name ? ', start' : ''}${s.isFinal ? ', final' : ''}`}
          onclick={(e) => (menu = { name: s.name, anchor: e.currentTarget })}
        >
          {#if spec.start === s.name}<span aria-hidden="true">→</span>{/if}
          <span class:final={s.isFinal}>{s.name}</span>
        </button>
      </li>
    {/each}
    <li>
      <button
        type="button"
        class="state add"
        onclick={() => pg.addState(undefined, undefined, 'table')}>+ State</button
      >
    </li>
  </ul>
</div>

<div class="scroll">
  <table aria-label="Transitions">
    <thead>
      <tr>
        {#each fields as f (f.field)}<th scope="col">{f.label}</th>{/each}
        <th scope="col"><span class="visually-hidden">Remove</span></th>
      </tr>
    </thead>
    <tbody>
      {#each spec.transitions as t, i (`${t.from} ${t.read.join(' ')}`)}
        <tr class:next={i === nextIndex}>
          {#each fields as f (f.field)}
            <td>
              <input
                class="mono"
                value={show(t, f.field)}
                aria-label={`${f.label} of δ(${t.from}, (${t.read.join(', ')}))`}
                autocomplete="off"
                spellcheck="false"
                onkeydown={(e) => keydown(e, i, f.field)}
                onblur={(e) => save(i, f.field, e.currentTarget)}
              />
            </td>
          {/each}
          <td class="remove">
            <button
              type="button"
              class="link"
              aria-label={`Remove δ(${t.from}, (${t.read.join(', ')}))`}
              onclick={() => pg.removeTransition(i)}>Remove</button
            >
          </td>
        </tr>
      {/each}
    </tbody>
  </table>
</div>

<form class="new" onsubmit={add} aria-label="New transition">
  {#each fields as f (f.field)}
    <label>
      <span>{f.label}</span>
      <input
        id={`${id}-new-${f.field}`}
        class="mono"
        bind:value={draft[f.field]}
        autocomplete="off"
        spellcheck="false"
        placeholder={f.field === 'from' || f.field === 'to'
          ? spec.start
          : f.field === 'dir'
            ? Array(k).fill('R').join(', ')
            : Array(k).fill(glyph(spec.blank)).join(', ')}
      />
    </label>
  {/each}
  <button class="button" type="submit">Add</button>
</form>

<p class="message" aria-live="polite">{problem ?? ''}</p>

{#if menu}
  <Menu
    open={true}
    anchor={menu.anchor}
    label={`State ${menu.name}`}
    items={menuItems(menu.name, menu.anchor)}
    onclose={() => (menu = null)}
  />
{/if}

{#if renaming}
  <RenameForm
    {pg}
    name={renaming.name}
    anchor={renaming.anchor}
    onclose={() => (renaming = null)}
  />
{/if}

<style>
  .states {
    display: flex;
    flex-wrap: wrap;
    align-items: center;
    gap: 0.5rem 0.75rem;
    margin-bottom: var(--space-2);
  }

  .states-label {
    font-size: var(--text-xs);
    color: var(--muted);
  }

  ul {
    display: flex;
    flex-wrap: wrap;
    gap: 0.4rem;
    margin: 0;
    padding: 0;
    list-style: none;
  }

  .state {
    display: inline-flex;
    gap: 0.25rem;
    align-items: center;
    min-height: 2rem;
    padding: 0 0.6rem;
    border: 1px solid var(--rule);
    background: var(--bg);
    cursor: pointer;
    font-size: var(--text-sm);
  }

  .state:hover {
    border-color: var(--rule-strong);
  }

  .state.current {
    color: var(--accent);
    border-color: var(--accent);
  }

  .final {
    text-decoration: underline double;
    text-underline-offset: 0.25em;
  }

  .add {
    color: var(--muted);
  }

  .scroll {
    overflow-x: auto;
  }

  table {
    border-collapse: collapse;
    font-size: var(--text-sm);
  }

  th {
    text-align: left;
    font-weight: 400;
    font-size: var(--text-xs);
    color: var(--muted);
    padding: 0 0.6rem 0.3rem;
    border-bottom: 1px solid var(--rule-strong);
  }

  td {
    border-bottom: 1px solid var(--rule);
    padding: 0;
  }

  td input {
    width: 7.5rem;
    height: 2.25rem;
    padding: 0 0.6rem;
    border: 0;
    background: transparent;
    font-size: var(--text-sm);
  }

  td input:focus-visible,
  .new input:focus-visible {
    outline: 2px solid var(--accent);
    outline-offset: -2px;
  }

  tr.next td input {
    color: var(--accent);
  }

  tr.next td:first-child {
    border-left: 2px solid var(--accent);
  }

  .remove {
    padding-inline: 0.6rem;
  }

  .link {
    border: 0;
    background: none;
    padding: 0;
    color: var(--muted);
    font-size: var(--text-xs);
    cursor: pointer;
    text-decoration: underline;
    text-underline-offset: 0.18em;
  }

  .link:hover {
    color: var(--text);
  }

  .new {
    display: flex;
    flex-wrap: wrap;
    align-items: end;
    gap: 0.5rem;
    margin-top: var(--space-2);
  }

  .new label {
    display: grid;
    gap: 0.2rem;
    font-size: var(--text-xs);
    color: var(--muted);
  }

  .new input {
    width: 7rem;
    height: 2.25rem;
    padding: 0 0.6rem;
    border: 1px solid var(--rule-strong);
    background: var(--field);
    color: var(--text);
    font-size: var(--text-sm);
  }

  .message {
    min-height: 1.5em;
    margin: 0.6rem 0 0;
    font-size: var(--text-sm);
  }
</style>
