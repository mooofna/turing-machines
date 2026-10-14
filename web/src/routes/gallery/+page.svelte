<script lang="ts">
  import { onMount } from 'svelte';
  import { resolve } from '$app/paths';
  import Cite from '#lib/components/Cite.svelte';
  import Live from '#lib/components/Live.svelte';
  import PageHeader from '#lib/components/PageHeader.svelte';
  import Section from '#lib/components/Section.svelte';
  import type { KindDto, MachineSpecDto } from '#lib/engine/pkg/turing_wasm.js';
  import { editor } from '#lib/engine/wasm.ts';
  import { groups, machinesIn, specOf, thumbnailMode } from '#lib/gallery.ts';
  import SpaceTime from '#lib/machine/SpaceTime.svelte';

  const KIND: Record<KindDto, string> = {
    acceptor: 'Acceptor',
    transducer: 'Transducer',
    enumerator: 'Enumerator',
    beaver: 'Busy beaver'
  };

  const plural = (n: number, word: string) => `${n} ${word}${n === 1 ? '' : 's'}`;

  /**
   * The number of states as busy beaver machines are counted: the halting state that the
   * standard format writes as Z (a final state without transitions) is not one of them.
   */
  function stateCount(spec: MachineSpecDto): number {
    if (spec.kind !== 'beaver') return spec.states.length;
    const active = new Set(spec.transitions.map((t) => t.from));
    return spec.states.filter((s) => !(s.isFinal && !active.has(s.name))).length;
  }

  function size(spec: MachineSpecDto): string {
    const parts = [plural(stateCount(spec), 'state'), plural(spec.tapeAlphabet.length, 'symbol')];
    if (spec.tapes > 1) parts.push(plural(spec.tapes, 'tape'));
    return parts.join(', ');
  }

  /** Parsed machines by id, once the editor module has loaded. */
  let specs = $state<Record<string, MachineSpecDto | null>>({});
  let loaded = $state(false);

  onMount(() => {
    editor()
      .then((wasm) => {
        const next: Record<string, MachineSpecDto | null> = {};
        for (const group of groups) {
          for (const machine of machinesIn(group.id)) {
            next[machine.id] = specOf(wasm, machine) ?? null;
          }
        }
        specs = next;
      })
      .catch(() => undefined)
      .finally(() => (loaded = true));
  });
</script>

<PageHeader
  title="Gallery"
  lede="Every machine on this site, grouped by origin. Each one opens in the playground, where you can run it step by step and change it."
/>

{#each groups as group (group.id)}
  {@const list = machinesIn(group.id)}
  <Section name={group.title} id={group.id} wide>
    {#snippet aside()}{plural(list.length, 'machine')}{/snippet}
    <table class="table machines">
      <thead>
        <tr>
          <th scope="col">Machine</th>
          <th scope="col">Kind and size</th>
          <th scope="col">Origin</th>
          <th scope="col"><span class="visually-hidden">Space-time diagram</span></th>
        </tr>
      </thead>
      <tbody>
        {#each list as machine (machine.id)}
          {@const spec = specs[machine.id]}
          <tr id="m-{machine.id}">
            <th scope="row" class="name">
              <span class="title">{machine.title}</span>
              <span class="description">{machine.description}</span>
              <!-- rel="external": the playground reads #g= on load, and the prerenderer does not
                   look for an element with that id. -->
              <a class="open" href={resolve(`/playground#g=${machine.id}`)} rel="external"
                >Open in playground<span class="visually-hidden">: {machine.title}</span></a
              >
            </th>
            <td class="kind">
              {#if spec}
                {KIND[spec.kind]}
                <span class="size">{size(spec)}</span>
              {:else if loaded}
                <span class="muted">Not available</span>
              {:else}
                <Live pending />
              {/if}
            </td>
            <td class="origin">
              {machine.provenance}{#if machine.source}<Cite ids={machine.source} />{/if}
            </td>
            <td class="thumb">
              {#if spec && spec.tapes === 1}
                <SpaceTime
                  {spec}
                  input={machine.input ?? spec.input}
                  mode={thumbnailMode(machine, spec)}
                  width={144}
                  height={88}
                  label="Space-time diagram of {machine.title}"
                />
              {:else if spec}
                <span class="muted none">No diagram for several tapes</span>
              {/if}
            </td>
          </tr>
        {/each}
      </tbody>
    </table>
  </Section>
{/each}

<style>
  .machines {
    table-layout: fixed;
  }

  /* Column widths; the machine column takes the rest. */
  .machines thead th:nth-child(2) {
    width: 9.5rem;
  }

  .machines thead th:nth-child(3) {
    width: 10rem;
  }

  .machines thead th:nth-child(4) {
    width: calc(144px + 2px);
  }

  .machines td,
  .machines tbody th {
    padding-block: 0.9rem;
  }

  .name {
    font-weight: 400;
  }

  .title {
    display: block;
    font-size: var(--text-base);
    font-weight: 650;
    line-height: 1.3;
  }

  .description {
    display: block;
    margin-top: 0.2rem;
    color: var(--muted);
  }

  .size {
    display: block;
    color: var(--muted);
  }

  .thumb :global(.spacetime) {
    border: 1px solid var(--rule);
  }

  .none {
    font-size: var(--text-xs);
  }

  .open {
    display: inline-block;
    margin-top: 0.5rem;
    font-weight: 400;
  }

  /* Below 64rem the four columns do not fit: each machine becomes a block. */
  @media (max-width: 63.99rem) {
    .machines,
    .machines tbody,
    .machines tr,
    .machines td,
    .machines tbody th {
      display: block;
      width: auto;
    }

    .machines thead {
      position: absolute;
      width: 1px;
      height: 1px;
      overflow: hidden;
      clip: rect(0, 0, 0, 0);
    }

    .machines tr {
      display: grid;
      grid-template-columns: minmax(0, 1fr) auto;
      column-gap: var(--space-2);
      padding-block: var(--space-2);
      border-bottom: 1px solid var(--rule);
    }

    .machines tr:first-child {
      border-top: 1px solid var(--rule-strong);
    }

    .machines td,
    .machines tbody th {
      padding: 0;
      border: 0;
    }

    .name {
      grid-column: 1 / -1;
    }

    .kind,
    .origin {
      grid-column: 1;
      margin-top: 0.5rem;
    }

    .thumb {
      grid-column: 2;
      grid-row: 2 / span 2;
      margin-top: 0.6rem;
    }
  }
</style>
