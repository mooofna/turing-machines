<!--
  The popover form of the diagram for one transition δ(from, read) = (to, write, move): symbols
  and moves are segmented controls, so a transition takes three choices and Enter. Used to
  create a transition (after dragging from a state) and to edit or delete one (after clicking
  its label).
-->
<script lang="ts">
  import type { DirDto } from '#lib/engine/pkg/turing_wasm.js';
  import { glyph } from './cell';
  import { findTransition, symbolProblem } from './edit';
  import Popover from './Popover.svelte';
  import type { Playground, Point } from './state.svelte';

  interface Props {
    pg: Playground;
    from: string;
    /** The target state; a name that does not exist yet is created at `at`. */
    to: string;
    at?: Point | null;
    /** The transition being edited, if any. */
    index?: number | null;
    anchor: DOMRect | HTMLElement | null;
    onclose: () => void;
  }

  let { pg, from, to, at = null, index = null, anchor, onclose }: Props = $props();
  const id = $props.id();

  // svelte-ignore state_referenced_locally
  const spec = pg.spec!;
  const k = spec.tapes;
  // svelte-ignore state_referenced_locally
  const editing = index !== null ? spec.transitions[index] : undefined;
  // svelte-ignore state_referenced_locally
  const isNewState = !spec.states.some((s) => s.name === to);

  const gamma = [...spec.tapeAlphabet];
  /** Read tuples already defined for `from`, except the one being edited. */
  const taken = new Set(
    spec.transitions
      .filter((t, i) => t.from === from && i !== index)
      .map((t) => t.read.join('\u0001'))
  );

  function firstFree(): string[] {
    if (k === 1) {
      const free = gamma.find((g) => !taken.has(g));
      return [free ?? gamma[0]!];
    }
    return Array(k).fill(spec.blank);
  }

  let read = $state<string[]>(editing ? [...editing.read] : firstFree());
  let write = $state<string[]>(editing ? [...editing.write] : [...read]);
  let dir = $state<DirDto[]>(editing ? [...editing.dir] : Array(k).fill('R'));
  // svelte-ignore state_referenced_locally
  let target = $state(to);
  let extra = $state<string[]>([]);
  let custom = $state<Record<string, string>>({});
  let problem = $state<string | null>(null);

  const symbols = $derived([...gamma, ...extra]);
  const states = $derived([...spec.states.map((s) => s.name), ...(isNewState ? [to] : [])]);
  const clash = $derived(k === 1 ? taken.has(read.join('\u0001')) : false);
  const title = $derived(
    editing ? `Edit δ(${from}, ${read.map(glyph).join(', ')})` : `New transition from ${from}`
  );

  function addCustom(group: 'read' | 'write', tape: number) {
    const key = `${group}${tape}`;
    const g = (custom[key] ?? '').trim();
    if (!g) return;
    const issue = gamma.includes(g) || extra.includes(g) ? null : symbolProblem(spec, g);
    if (issue) {
      problem = issue;
      return;
    }
    if (!gamma.includes(g) && !extra.includes(g)) extra = [...extra, g];
    if (group === 'read') read[tape] = g;
    else write[tape] = g;
    custom[key] = '';
    problem = null;
  }

  function submit(event: SubmitEvent) {
    event.preventDefault();
    if (clash) {
      problem = `δ(${from}, ${read.map(glyph).join(', ')}) is already defined.`;
      return;
    }
    if (k > 1) {
      const other = findTransition(spec, from, read);
      if (other >= 0 && other !== index) {
        problem = `δ(${from}, (${read.join(', ')})) is already defined.`;
        return;
      }
    }
    const positions = isNewState && at ? { [to]: at } : undefined;
    if (editing && index !== null) {
      pg.editTransition(
        index,
        { read: [...read], to: target, write: [...write], dir: [...dir] },
        'diagram'
      );
    } else {
      pg.addTransition(
        from,
        [...read],
        { to: target, write: [...write], dir: [...dir] },
        'diagram',
        positions
      );
    }
    onclose();
  }

  function remove() {
    if (index !== null) pg.removeTransition(index, 'diagram');
    onclose();
  }

  function formKeydown(event: KeyboardEvent) {
    const el = event.target as HTMLElement;
    if (event.key !== 'Enter' || el.tagName === 'BUTTON' || el.tagName === 'SELECT') return;
    if (el.dataset.custom) return;
    event.preventDefault();
    (event.currentTarget as HTMLFormElement).requestSubmit();
  }

  const moves: { d: DirDto; label: string }[] = [
    { d: 'L', label: 'Left' },
    { d: 'R', label: 'Right' },
    { d: 'N', label: 'Stay' }
  ];
</script>

<Popover open={true} {anchor} label={title} {onclose} width={280}>
  <!-- svelte-ignore a11y_no_noninteractive_element_interactions -->
  <form class="form" onsubmit={submit} onkeydown={formKeydown}>
    <p class="title mono">{title}</p>

    {#each { length: k } as _, tape (tape)}
      <fieldset class="group">
        <legend>Read{k > 1 ? ` on tape ${tape + 1}` : ''}</legend>
        <div class="segments">
          {#each symbols as g (g)}
            {@const used = k === 1 && taken.has(g)}
            <label
              class="segment mono"
              class:used
              title={used ? `δ(${from}, ${glyph(g)}) is already defined` : undefined}
            >
              <input
                type="radio"
                name={`${id}-read-${tape}`}
                value={g}
                bind:group={read[tape]}
                disabled={used}
                data-autofocus={tape === 0 && read[0] === g ? true : undefined}
                onchange={() => {
                  if (!editing) write[tape] = g;
                }}
              />
              <span>{glyph(g)}</span>
            </label>
          {/each}
          <input
            class="custom mono"
            aria-label="Another symbol to read"
            placeholder="+"
            maxlength="2"
            data-custom="true"
            bind:value={custom[`read${tape}`]}
            onkeydown={(e) => {
              if (e.key === 'Enter') {
                e.preventDefault();
                addCustom('read', tape);
              }
            }}
            onblur={() => addCustom('read', tape)}
          />
        </div>
      </fieldset>
    {/each}

    {#each { length: k } as _, tape (tape)}
      <fieldset class="group">
        <legend>Write{k > 1 ? ` on tape ${tape + 1}` : ''}</legend>
        <div class="segments">
          {#each symbols as g (g)}
            <label class="segment mono">
              <input type="radio" name={`${id}-write-${tape}`} value={g} bind:group={write[tape]} />
              <span>{glyph(g)}</span>
            </label>
          {/each}
          <input
            class="custom mono"
            aria-label="Another symbol to write"
            placeholder="+"
            maxlength="2"
            data-custom="true"
            bind:value={custom[`write${tape}`]}
            onkeydown={(e) => {
              if (e.key === 'Enter') {
                e.preventDefault();
                addCustom('write', tape);
              }
            }}
            onblur={() => addCustom('write', tape)}
          />
        </div>
      </fieldset>
    {/each}

    {#each { length: k } as _, tape (tape)}
      <fieldset class="group">
        <legend>Move{k > 1 ? ` on tape ${tape + 1}` : ''}</legend>
        <div class="segments">
          {#each moves as m (m.d)}
            <label class="segment" title={m.label}>
              <input type="radio" name={`${id}-dir-${tape}`} value={m.d} bind:group={dir[tape]} />
              <span class="mono">{m.d}</span>
              <span class="visually-hidden">{m.label}</span>
            </label>
          {/each}
        </div>
      </fieldset>
    {/each}

    <label class="next">
      <span>Next state</span>
      <select bind:value={target} class="mono">
        {#each states as s (s)}
          <option value={s}>{s}{s === to && isNewState ? ' (new)' : ''}</option>
        {/each}
      </select>
    </label>

    <p class="problem" aria-live="polite">
      {problem ?? (clash ? `δ(${from}, ${glyph(read[0] ?? '')}) is already defined.` : '')}
    </p>

    <div class="actions">
      <button class="button primary" type="submit" disabled={clash}
        >{editing ? 'Save' : 'Create'}</button
      >
      {#if editing}
        <button class="button" type="button" onclick={remove}>Delete</button>
      {/if}
      <button class="button quiet" type="button" onclick={onclose}>Cancel</button>
    </div>
  </form>
</Popover>

<style>
  .form {
    display: grid;
    gap: 0.6rem;
    padding: 0.75rem;
  }

  .title {
    margin: 0;
    font-size: var(--text-sm);
    font-weight: 500;
  }

  .group {
    margin: 0;
    padding: 0;
    border: 0;
    min-width: 0;
  }

  legend {
    padding: 0;
    margin-bottom: 0.25rem;
    font-size: var(--text-xs);
    color: var(--muted);
  }

  .segments {
    display: flex;
    flex-wrap: wrap;
  }

  .segment {
    position: relative;
    display: grid;
    place-items: center;
    min-width: 2.25rem;
    height: 2.25rem;
    margin: 0 -1px -1px 0;
    border: 1px solid var(--rule-strong);
    cursor: pointer;
    font-size: var(--text-sm);
  }

  .segment input {
    position: absolute;
    inset: 0;
    margin: 0;
    opacity: 0;
    cursor: pointer;
  }

  .segment:has(input:checked) {
    background: var(--text);
    color: var(--bg);
  }

  .segment:has(input:focus-visible) {
    outline: 2px solid var(--accent);
    outline-offset: 1px;
    z-index: 1;
  }

  .segment.used {
    color: var(--muted);
    text-decoration: line-through;
    cursor: not-allowed;
  }

  .segment.used input {
    cursor: not-allowed;
  }

  .custom {
    width: 2.25rem;
    height: 2.25rem;
    margin: 0 -1px -1px 0;
    padding: 0;
    border: 1px solid var(--rule-strong);
    background: var(--field);
    text-align: center;
    font-size: var(--text-sm);
  }

  .custom::placeholder {
    color: var(--muted);
  }

  .custom:focus-visible,
  select:focus-visible {
    outline: 2px solid var(--accent);
    outline-offset: 1px;
  }

  .next {
    display: grid;
    gap: 0.25rem;
    font-size: var(--text-xs);
    color: var(--muted);
  }

  select {
    height: 2.25rem;
    padding: 0 0.5rem;
    border: 1px solid var(--rule-strong);
    border-radius: 0;
    background: var(--field);
    color: var(--text);
    font-size: var(--text-sm);
  }

  .problem {
    min-height: 1.2em;
    margin: 0;
    font-size: var(--text-xs);
  }

  .actions {
    display: flex;
    flex-wrap: wrap;
    gap: 0.5rem;
  }

  .button.quiet {
    border-color: var(--rule);
  }
</style>
