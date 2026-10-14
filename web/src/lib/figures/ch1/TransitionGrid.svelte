<!--
  A read-only transition table of a single-tape machine in the course notation: one row per
  state, one column per tape symbol, each cell δ(q, a) as "q', b, D". The cell of the current
  state and the symbol under the head is outlined in the accent: the transition the next step
  applies, or the undefined entry at which the machine has halted.
-->
<script lang="ts">
  import type { MachineSpecDto } from '#lib/engine/pkg/turing_wasm.js';

  interface Props {
    spec: MachineSpecDto | null;
    /** The current state and the symbol under the head. */
    at?: { state: string; read: string } | null;
    /** Accessible name of the table. */
    label: string;
  }

  let { spec, at = null, label }: Props = $props();

  const symbols = $derived(spec?.tapeAlphabet ?? []);
  const states = $derived(spec?.states ?? []);

  function entry(state: string, read: string) {
    return spec?.transitions.find((t) => t.from === state && t.read[0] === read);
  }

  const show = (g: string) => (g === '_' ? '␣' : g);
  const speak = (g: string) => (g === '␣' || g === '_' ? 'blank' : g);
</script>

<table class="grid" aria-label={label}>
  <thead>
    <tr>
      <th scope="col" class="corner"
        ><span aria-hidden="true">δ</span><span class="visually-hidden">State</span></th
      >
      {#each symbols as g (g)}
        <th scope="col" class="mono"
          ><span aria-hidden="true">{show(g)}</span><span class="visually-hidden"
            >reading {speak(g)}</span
          ></th
        >
      {/each}
    </tr>
  </thead>
  <tbody>
    {#each states as s (s.name)}
      {@const current = at?.state === s.name}
      <tr class:current>
        <th scope="row" class="state mono">
          <span class="start" aria-hidden="true">{spec?.start === s.name ? '→' : ''}</span>
          <span class="name" class:final={s.isFinal}>{s.name}</span>
          {#if spec?.start === s.name}<span class="visually-hidden">, start state</span>{/if}
          {#if s.isFinal}<span class="visually-hidden">, final state</span>{/if}
        </th>
        {#each symbols as g (g)}
          {@const t = entry(s.name, g)}
          {@const active = current && at?.read === g}
          <td class="mono" class:active class:empty={!t}>
            {#if t}
              {t.to}, {show(t.write[0] ?? '')}, {t.dir[0]}
            {:else}
              <span class="visually-hidden">undefined</span>
            {/if}
            {#if active}<span class="visually-hidden"
                >{t ? ', applied next' : ', the machine halts here'}</span
              >{/if}
          </td>
        {/each}
      </tr>
    {/each}
  </tbody>
</table>

<style>
  .grid {
    border-collapse: collapse;
    font-size: var(--text-sm);
    line-height: 1.3;
  }

  th,
  td {
    height: 2.1rem;
    padding: 0 0.7rem;
    border-bottom: 1px solid var(--rule);
    text-align: left;
    white-space: nowrap;
  }

  thead th {
    border-bottom: 1px solid var(--rule-strong);
    font-weight: 400;
    color: var(--muted);
  }

  thead th.mono {
    font-size: 0.95rem;
    color: var(--text);
  }

  .corner {
    padding-left: 0;
    font-family: var(--font-mono);
  }

  .state {
    padding-left: 0;
    font-weight: 400;
    border-right: 1px solid var(--rule-strong);
  }

  .start {
    display: inline-block;
    width: 1.1rem;
    color: var(--muted);
    font-family: var(--font-sans);
  }

  .name {
    padding: 0.05rem 0.2rem;
    border: 1px solid transparent;
  }

  .name.final {
    border-color: currentColor;
    outline: 1px solid currentColor;
    outline-offset: 1px;
  }

  .current .state {
    color: var(--accent);
  }

  td.active {
    outline: 2px solid var(--accent);
    outline-offset: -2px;
    color: var(--accent);
  }

  td.mono,
  th.mono {
    font-size: 0.86rem;
  }

  @media (max-width: 47.99rem) {
    th,
    td {
      padding: 0 0.45rem;
    }

    td.mono {
      font-size: 0.8rem;
    }
  }
</style>
