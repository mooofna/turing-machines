<script lang="ts">
  import { onMount } from 'svelte';
  import { resolve } from '$app/paths';
  import ChapterNav from '#lib/components/ChapterNav.svelte';
  import Live, { formatDuration } from '#lib/components/Live.svelte';
  import Section from '#lib/components/Section.svelte';
  import { engine } from '#lib/engine/client.ts';
  import type { MachineSpecDto } from '#lib/engine/pkg/turing_wasm.js';
  import { editor } from '#lib/engine/wasm.ts';
  import SpaceTime from '#lib/machine/SpaceTime.svelte';

  /** The five-state champion of Marxen and Buntrock, in the bbchallenge standard format. */
  const CHAMPION = '1RB1LC_1RC1RB_1RD0LE_1LA1LD_1RZ0LA';
  /** More steps than the champion needs; the run stops when the machine halts. */
  const BUDGET = 100_000_000;

  let spec = $state<MachineSpecDto | null>(null);
  let steps = $state<number | null>(null);
  let ones = $state<number | null>(null);
  let elapsed = $state<number | null>(null);
  let running = $state(false);
  let measured = $state(false);
  let failed = $state(false);
  /** Portrait next to the title on wide screens, square below it on phones. */
  let heroAspect = $state(4 / 5);

  // Run the champion once to measure it, then draw it: the diagram is requested only after the
  // run, so the time shown is the time of the run alone.
  onMount(() => {
    if (window.matchMedia('(max-width: 47.99rem)').matches) heroAspect = 1;
    let stopped = false;
    const session = 'landing-champion';
    const run = async () => {
      const wasm = await editor();
      const parsed = wasm.fromStandard(CHAMPION).spec;
      if (!parsed || stopped) return;
      spec = parsed;
      const worker = engine();
      await worker.request(
        { type: 'open', session, spec: parsed, input: '', radius: 0 },
        'snapshot'
      );
      running = true;
      const call = worker.call(
        {
          type: 'run',
          session,
          maxSteps: BUDGET,
          breakpoints: { transitions: [], states: [] },
          radius: 0
        },
        (response) => {
          if (response.type !== 'progress' || stopped) return;
          steps = response.snapshot.step;
          ones = response.snapshot.ones;
        }
      );
      const final = await call.done;
      if (final.type === 'stopped' && !stopped) {
        steps = final.result.snapshot.step;
        ones = final.result.snapshot.ones;
        elapsed = final.elapsedMs;
      }
      running = false;
      measured = true;
      worker.call({ type: 'close', session });
    };
    run().catch(() => {
      running = false;
      failed = true;
    });
    return () => {
      stopped = true;
    };
  });
</script>

<section class="page row intro" aria-labelledby="title">
  <div class="head">
    <h1 id="title">Turing machines</h1>
    <p class="lede">
      A simulator for Turing machines and six chapters that lead from Turing’s definition of 1936 to
      open problems about the busy beaver function.
    </p>
    <p class="note">
      Every machine on these pages runs in your browser. The project began as an exercise in the
      course Theoretische Informatik und Logik at TU Dresden.
    </p>
    <p class="actions">
      <a class="button primary" href={resolve('/machine')}>Start with chapter 1</a>
      <a class="secondary" href={resolve('/playground')}>Open the playground</a>
    </p>
  </div>

  <figure class="hero">
    <div class="plate frame">
      {#if spec && measured}
        <SpaceTime
          {spec}
          mode={{ kind: 'overview', maxSteps: BUDGET }}
          aspect={heroAspect}
          label="Space-time diagram of the five-state busy beaver champion, from the blank tape until it halts. Time runs downwards."
        />
      {:else}
        <p class="waiting mono" aria-hidden="true">{failed ? 'No diagram' : 'Computing'}</p>
      {/if}
    </div>
    <figcaption>
      The five-state busy beaver champion <span class="mono machine">{CHAMPION}</span>, run from a
      blank tape until it halts. Time runs downwards; each row samples the tape.
    </figcaption>
  </figure>

  <dl class="facts" aria-live="polite" aria-busy={running}>
    <div>
      <dt>Steps until it halts</dt>
      <dd><Live value={steps} pending={running} /></dd>
    </div>
    <div>
      <dt>Ones left on the tape</dt>
      <dd><Live value={ones} pending={running} /></dd>
    </div>
    <div>
      <dt>Time in this browser</dt>
      <dd><Live value={elapsed} pending={running} format={formatDuration} /></dd>
    </div>
  </dl>
</section>

<Section name="Chapters" id="chapters">
  <ChapterNav summaries label="Chapters" />
</Section>

<Section name="Playground and gallery" id="tools" wide>
  <ul class="index">
    <li>
      <a href={resolve('/playground')}>
        <span class="title">Playground</span>
        <span class="summary">
          Write a machine in the course notation, as a table or as a state diagram, and run it step
          by step or at full speed.
        </span>
      </a>
    </li>
    <li>
      <a href={resolve('/gallery')}>
        <span class="title">Gallery</span>
        <span class="summary">
          Every machine on this site with its origin: exercises from the course, classic examples,
          busy beaver champions and machines whose behaviour is still unknown.
        </span>
      </a>
    </li>
  </ul>
</Section>

<style>
  .intro {
    padding-block: var(--space-5) var(--space-4);
    row-gap: var(--space-3);
    border-bottom: 1px solid var(--rule-strong);
  }

  .head {
    grid-column: 1 / span 7;
    grid-row: 1;
  }

  h1 {
    font-size: var(--text-display);
    font-weight: 700;
    letter-spacing: -0.03em;
    line-height: 1;
    margin-left: -0.03em;
  }

  .lede {
    max-width: 34rem;
    margin: var(--space-3) 0 0;
    color: var(--text);
    font-size: var(--text-lg);
    line-height: 1.35;
    letter-spacing: -0.01em;
  }

  .note {
    max-width: 34rem;
    margin: var(--space-2) 0 0;
    color: var(--muted);
  }

  .actions {
    display: flex;
    flex-wrap: wrap;
    align-items: center;
    gap: var(--space-2) var(--space-3);
    margin: var(--space-4) 0 0;
  }

  .secondary {
    font-size: var(--text-sm);
  }

  .hero {
    grid-column: 9 / span 4;
    grid-row: 1 / span 2;
    margin: 0;
  }

  .waiting {
    aspect-ratio: 4 / 5;
    display: grid;
    place-items: center;
    margin: 0;
    color: var(--muted);
    font-size: var(--text-xs);
  }

  figcaption {
    margin-top: 0.6rem;
    font-size: var(--text-xs);
    line-height: 1.5;
    color: var(--muted);
  }

  .machine {
    color: var(--text);
    overflow-wrap: anywhere;
  }

  .facts {
    grid-column: 1 / span 7;
    grid-row: 2;
    align-self: end;
    display: grid;
    grid-template-columns: repeat(3, minmax(0, 1fr));
    gap: var(--space-2) var(--gutter);
    margin: var(--space-3) 0 0;
    font-size: var(--text-sm);
    line-height: 1.4;
  }

  dt {
    color: var(--muted);
    margin-bottom: 0.2rem;
  }

  dd {
    margin: 0;
    font-size: var(--text-lg);
    line-height: 1.2;
  }

  @media (max-width: 47.99rem) {
    .intro {
      padding-top: var(--space-4);
    }

    .head,
    .hero,
    .facts {
      grid-column: 1 / -1;
      grid-row: auto;
    }

    .actions {
      margin-top: var(--space-3);
    }

    .facts {
      margin-top: 0;
      gap: var(--space-2) var(--space-2);
    }

    .waiting {
      aspect-ratio: 1;
    }

    dd {
      font-size: var(--text-base);
    }
  }

  .index {
    margin: 0;
    padding: 0;
    list-style: none;
  }

  .index li {
    border-top: 1px solid var(--rule);
  }

  .index a {
    display: grid;
    row-gap: 0.25rem;
    padding-block: var(--space-2) calc(var(--space-2) + 0.1rem);
    padding-left: 2.5rem;
    text-decoration: none;
  }

  .index .title {
    font-size: var(--text-lg);
    font-weight: 650;
    letter-spacing: -0.01em;
    line-height: 1.2;
  }

  .index a:hover .title {
    text-decoration: underline;
    text-decoration-color: var(--accent);
    text-decoration-thickness: 2px;
    text-underline-offset: 0.18em;
  }

  @media (max-width: 47.99rem) {
    .index a {
      padding-left: 0;
    }
  }

  .index .summary {
    max-width: var(--measure);
    font-size: var(--text-sm);
    color: var(--muted);
  }
</style>
