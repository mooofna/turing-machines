<!--
  Superscript reference numbers that link to the bibliography on the About page.
  Usage: <Cite ids="turing1936" /> or <Cite ids={['rado1962', 'lin1965']} />.
-->
<script lang="ts">
  import { resolve } from '$app/paths';
  import { shortCitation, sourceAnchor, sourceById, sourceNumber } from './bibliography';

  interface Props {
    /** One source id or several, from `#lib/content/sources.ts`. */
    ids: string | readonly string[];
  }

  let { ids }: Props = $props();

  const refs = $derived(
    (typeof ids === 'string' ? [ids] : ids).map((id) => {
      const source = sourceById(id);
      return {
        id,
        n: sourceNumber(id),
        label: source ? `Source ${sourceNumber(id)}: ${shortCitation(source)}` : `Source ${id}`
      };
    })
  );
</script>

<sup class="cite"
  >{#each refs as ref, i (ref.id)}{#if i > 0}<span class="sep">,</span>{/if}<a
      href={resolve(`/about#${sourceAnchor(ref.id)}`)}
      aria-label={ref.label}
      title={ref.label}>{ref.n ?? '?'}</a
    >{/each}</sup
>

<style>
  .cite {
    margin-left: 0.1em;
    font-family: var(--font-mono);
    font-size: 0.68em;
    font-variant-numeric: tabular-nums;
    vertical-align: super;
    line-height: 0;
  }

  a {
    color: var(--muted);
    text-decoration: none;
    padding-inline: 0.05em;
  }

  a:hover {
    color: var(--accent);
  }

  .sep {
    color: var(--muted);
  }
</style>
