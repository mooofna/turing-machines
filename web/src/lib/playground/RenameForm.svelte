<!--
  A popover with one field to rename a state.
-->
<script lang="ts">
  import { stateNameProblem } from './edit';
  import Popover from './Popover.svelte';
  import type { Origin, Playground } from './state.svelte';

  interface Props {
    placement?: 'below' | 'side';
    pg: Playground;
    name: string;
    anchor: HTMLElement | DOMRect | null;
    origin?: Origin;
    onclose: () => void;
  }

  let { pg, name, anchor, origin = 'table', onclose, placement = 'below' }: Props = $props();
  const id = $props.id();

  // svelte-ignore state_referenced_locally
  let draft = $state(name);
  const problem = $derived(pg.spec ? stateNameProblem(pg.spec, draft.trim(), name) : null);

  function submit(event: SubmitEvent) {
    event.preventDefault();
    if (problem) return;
    if (draft.trim() !== name) pg.renameState(name, draft.trim(), origin);
    onclose();
  }
</script>

<Popover open={true} {anchor} label={`Rename ${name}`} {onclose} width={240} {placement}>
  <form class="form" onsubmit={submit}>
    <label class="field" for={`${id}-name`}>New name for {name}</label>
    <input
      id={`${id}-name`}
      class="text mono"
      data-autofocus
      bind:value={draft}
      autocomplete="off"
      spellcheck="false"
      aria-invalid={problem ? 'true' : undefined}
      aria-describedby={`${id}-problem`}
      onfocus={(e) => e.currentTarget.select()}
    />
    <p class="problem" id={`${id}-problem`}>{problem ?? ''}</p>
    <div class="actions">
      <button class="button primary" type="submit" disabled={problem !== null}>Rename</button>
      <button class="button" type="button" onclick={onclose}>Cancel</button>
    </div>
  </form>
</Popover>

<style>
  .form {
    display: grid;
    gap: 0.4rem;
    padding: 0.75rem;
  }

  .field {
    font-size: var(--text-xs);
    color: var(--muted);
  }

  .text {
    height: 2.25rem;
    padding: 0 0.6rem;
    border: 1px solid var(--rule-strong);
    background: var(--field);
    font-size: var(--text-sm);
  }

  .text:focus-visible {
    outline: 2px solid var(--accent);
    outline-offset: -1px;
  }

  .problem {
    min-height: 1.2em;
    margin: 0;
    font-size: var(--text-xs);
  }

  .actions {
    display: flex;
    gap: 0.5rem;
  }
</style>
