<!--
  A value from the facts ledger: the printed value in mono, followed by its references.
  Usage: <Fact fact={facts['bb5-steps']} />.
-->
<script lang="ts">
  import type { Fact } from '#lib/content/types.ts';
  import Cite from './Cite.svelte';

  interface Props {
    fact: Fact;
    /** Show the reference numbers; off when the same sentence already cites the source. */
    cite?: boolean;
  }

  let { fact, cite = true }: Props = $props();
</script>

<span class="fact" data-fact={fact.id}
  ><span class="mono value">{fact.display}</span>{#if cite}<Cite ids={fact.source} />{/if}</span
>

<style>
  .fact {
    white-space: nowrap;
  }
</style>
