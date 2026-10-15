<!--
  Fig. 4.2: milestones of the busy beaver functions, from Radó's definition to the verified value
  for five states. Every value comes from the facts ledger and every row names the publication
  it rests on; the year is that publication's year. On phones the values and the source move
  under the milestone, so the table needs no sideways scrolling.
-->
<script lang="ts">
  import Fact from '#lib/components/Fact.svelte';
  import { sourceById } from '#lib/components/bibliography.ts';
  import { facts } from '#lib/content/facts.ts';
  import type { FigureProps } from '../registry.ts';
  import Fn from './common/Fn.svelte';
  import SourceLink from './common/SourceLink.svelte';

  let { id }: FigureProps = $props();

  type FactId = keyof typeof facts;

  interface Value {
    /** The function, `S` or `Σ`, and its argument. */
    name: 'S' | 'Σ';
    arg: number;
    relation: '=' | '≥';
    fact: FactId;
  }

  interface Milestone {
    /** The publication the row rests on; its year is the year of the row. */
    source: string;
    /** Further sources, cited after the first. */
    more?: string[];
    /** Text, with busy beaver functions as `[name]` or `[name, argument]`. */
    text: (string | [string] | [string, number])[];
    values: Value[];
  }

  const year = (source: string) => sourceById(source)?.year ?? '';

  const MILESTONES: Milestone[] = [
    {
      source: 'rado1962',
      text: [
        'Radó defines the functions ',
        ['S'],
        ' and ',
        ['Σ'],
        '; his paper states the values of ',
        ['Σ'],
        ' for one and two states.'
      ],
      values: [
        { name: 'Σ', arg: 1, relation: '=', fact: 'bb1-ones' },
        { name: 'Σ', arg: 2, relation: '=', fact: 'bb2-ones' }
      ]
    },
    {
      source: 'lin1963',
      more: ['linrado1965'],
      text: [
        'Lin determines ',
        ['S', 2],
        ` and both values for three states with a computer program in his dissertation, published with Radó in ${year('linrado1965')}.`
      ],
      values: [
        { name: 'S', arg: 2, relation: '=', fact: 'bb2-steps' },
        { name: 'S', arg: 3, relation: '=', fact: 'bb3-steps' },
        { name: 'Σ', arg: 3, relation: '=', fact: 'bb3-ones' }
      ]
    },
    {
      source: 'brady1983',
      text: ['Brady’s determination of both values for four states is published.'],
      values: [
        { name: 'S', arg: 4, relation: '=', fact: 'bb4-steps' },
        { name: 'Σ', arg: 4, relation: '=', fact: 'bb4-ones' }
      ]
    },
    {
      source: 'machlinstout1990',
      text: ['Machlin and Stout give an independent proof for four states.'],
      values: [
        { name: 'S', arg: 4, relation: '=', fact: 'bb4-steps' },
        { name: 'Σ', arg: 4, relation: '=', fact: 'bb4-ones' }
      ]
    },
    {
      source: 'marxenbuntrock1990',
      text: [
        'Marxen and Buntrock find the five-state machine that is now known to be the champion, in a computer search.'
      ],
      values: [
        { name: 'S', arg: 5, relation: '≥', fact: 'bb5-steps' },
        { name: 'Σ', arg: 5, relation: '≥', fact: 'bb5-ones' }
      ]
    },
    {
      source: 'bbchallenge-zenodo',
      text: ['First release of bbchallenge.org, the website of the bbchallenge collaboration.'],
      values: []
    },
    {
      source: 'bbchallenge-announcement-2024',
      more: ['coqbb5'],
      text: [
        'The collaboration announces a proof of the five-state value, checked in the proof assistant Coq.'
      ],
      values: [{ name: 'S', arg: 5, relation: '=', fact: 'bb5-steps' }]
    },
    {
      source: 'bbchallenge2025',
      text: ['The paper on the proof describes its methods; it also determines ', ['Σ', 5], '.'],
      values: [
        { name: 'S', arg: 5, relation: '=', fact: 'bb5-steps' },
        { name: 'Σ', arg: 5, relation: '=', fact: 'bb5-ones' }
      ]
    }
  ];
</script>

{#snippet values(row: Milestone)}
  {#each row.values as value, i (i)}
    <span class="value"
      ><Fn name={value.name} arg={value.arg} />&nbsp;{value.relation}&nbsp;<Fact
        fact={facts[value.fact]}
        cite={false}
      /></span
    >{#if i < row.values.length - 1}<span class="sep">{', '}</span>{/if}
  {/each}
{/snippet}

{#snippet sources(row: Milestone)}
  {#each [row.source, ...(row.more ?? [])] as source, i (source)}
    {#if i > 0}<span class="sep">{', '}</span>{/if}<SourceLink id={source} />
  {/each}
{/snippet}

<div class="history">
  <table class="table" aria-labelledby="{id}-caption">
    <caption id="{id}-caption" class="visually-hidden">
      Milestones in the history of the busy beaver functions, with the values they established and
      their sources
    </caption>
    <thead>
      <tr>
        <th scope="col" class="year">Year</th>
        <th scope="col">Milestone</th>
        <th scope="col" class="wide-only">Values</th>
        <th scope="col" class="wide-only">Source</th>
      </tr>
    </thead>
    <tbody>
      {#each MILESTONES as row (row.source)}
        <tr>
          <td class="year mono">{year(row.source)}</td>
          <td class="text">
            {#each row.text as part, i (i)}{#if typeof part === 'string'}{part}{:else}<Fn
                  name={part[0]}
                  arg={part[1]}
                />{/if}{/each}
            <span class="narrow-only">
              {#if row.values.length > 0}<span class="line values">{@render values(row)}</span>{/if}
              <span class="line">{@render sources(row)}</span>
            </span>
          </td>
          <td class="wide-only values">{@render values(row)}</td>
          <td class="wide-only">{@render sources(row)}</td>
        </tr>
      {/each}
    </tbody>
  </table>
</div>

<style>
  .history {
    border-top: 1px solid var(--rule-strong);
    padding-top: 0.6rem;
  }

  .table td {
    padding-block: 0.7rem;
  }

  .year {
    width: 4.5rem;
    white-space: nowrap;
  }

  td.year {
    font-size: var(--text-sm);
  }

  .text {
    max-width: 24rem;
    text-wrap: pretty;
  }

  .values {
    min-width: 11rem;
  }

  .value {
    white-space: nowrap;
  }

  .value :global(.fact .value) {
    font-size: 0.92em;
  }

  .sep {
    color: var(--muted);
  }

  .narrow-only {
    display: none;
  }

  @media (max-width: 47.99rem) {
    .wide-only {
      display: none;
    }

    .narrow-only {
      display: block;
    }

    .line {
      display: block;
      margin-top: 0.35rem;
    }

    .year {
      width: 3.5rem;
    }
  }
</style>
