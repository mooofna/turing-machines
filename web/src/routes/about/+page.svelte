<script lang="ts">
  import Bibliography from '#lib/components/Bibliography.svelte';
  import Details from '#lib/components/Details.svelte';
  import PageHeader from '#lib/components/PageHeader.svelte';
  import Section from '#lib/components/Section.svelte';

  const REPO = 'https://github.com/mooofna/turing-machines';
  const blob = (file: string) => `${REPO}/blob/main/${file}`;

  const BIBTEX = `@software{heide2026turing,
  author     = {Heide, Maurice},
  title      = {Turing machines: a {Rust} simulator and busy beaver
                enumeration with a {WebAssembly} site},
  year       = {2026},
  license    = {MIT OR Apache-2.0},
  url        = {https://mooofna.github.io/turing-machines/},
  repository = {${REPO}}
}`;

  const CRATES = [
    {
      name: 'turing-core',
      role: 'Machines with one to three tapes, the description language of the course, the bbchallenge standard format and the simulators. No input or output, no unsafe code.'
    },
    {
      name: 'turing-bb',
      role: 'Enumeration of machines in tree normal form, the deciders that prove machines never halt, and the Antihydra rules.'
    },
    {
      name: 'turing-render',
      role: 'Space-time diagrams, the layout and edge routing of state diagrams, and SVG export.'
    },
    {
      name: 'turing-cli',
      role: 'The command-line program tm, which runs, traces, checks, converts, enumerates and renders machines.'
    },
    {
      name: 'turing-wasm',
      role: 'Bindings for the browser. The TypeScript types of the site are generated from the Rust types.'
    }
  ];

  const SOFTWARE = [
    { name: 'Svelte and SvelteKit', use: 'pages and components', licence: 'MIT' },
    { name: 'CodeMirror', use: 'text editor in the playground', licence: 'MIT' },
    { name: 'Temml', use: 'mathematics as MathML, at build time', licence: 'MIT' },
    {
      name: 'wasm-bindgen',
      use: 'bindings between Rust and JavaScript',
      licence: 'MIT or Apache-2.0'
    },
    { name: 'Schibsted Grotesk', use: 'typeface for text', licence: 'SIL Open Font License 1.1' },
    {
      name: 'IBM Plex Mono',
      use: 'typeface for tapes and numbers',
      licence: 'SIL Open Font License 1.1'
    }
  ];

  let copied = $state(false);

  async function copyCitation() {
    try {
      await navigator.clipboard.writeText(BIBTEX);
      copied = true;
      setTimeout(() => (copied = false), 2500);
    } catch {
      copied = false;
    }
  }
</script>

<PageHeader
  title="About"
  lede="Where this project comes from, how it is built, and the terms under which you can use it."
/>

<Section name="Origin" id="origin">
  <p>
    The project began in April 2024 as an exercise in Theoretische Informatik und Logik (TheoLog),
    the course on theoretical computer science and logic taught by Prof. Markus Krötzsch and Dr.
    Stephan Mennicke at TU Dresden. Task 3 of the first exercise sheet asked for a Turing machine
    that multiplies two numbers written in unary. To test my answer I wrote a small simulator in
    Rust, a single file that read machines from JSON.
  </p>
  <p>
    In 2026 I rewrote it from scratch as a Rust workspace with tests, a command-line program and a
    WebAssembly build for this site. The multiplication machine from the exercise is part of the
    gallery and of the first chapter.
  </p>
</Section>

<Section name="How it is built" id="build">
  <p>
    The engine is a Rust workspace of five crates. The site is written with SvelteKit and
    prerendered to static pages; the engine runs in two web workers, one for the playground and the
    diagrams and one for long enumerations, so that neither blocks the page. Formulas are converted
    to MathML when the site is built.
  </p>
  <table class="table crates">
    <thead>
      <tr><th scope="col">Crate</th><th scope="col">Contents</th></tr>
    </thead>
    <tbody>
      {#each CRATES as crate (crate.name)}
        <tr>
          <th scope="row"
            ><a class="mono" href="{REPO}/tree/main/crates/{crate.name}">{crate.name}</a></th
          >
          <td>{crate.role}</td>
        </tr>
      {/each}
    </tbody>
  </table>
</Section>

<!--
  Bibliography: rendered from #lib/content/sources.ts (export const sources: Source[]), which the
  chapters cite with <Cite> and <Fact>. Every reference number on the site links to an entry here.
-->
<Section name="Sources" id="sources">
  <Bibliography />
</Section>

<Section name="Acknowledgements" id="acknowledgements">
  <p>
    The chapters on the busy beaver build on the work of the bbchallenge collaboration: its Coq
    proof of the value of BB(5), its wiki and its catalogue of machines. The tests of the
    enumeration compare every machine with the verified enumerations of two- and three-state
    machines published with Coq-BB5 under the MIT licence.
  </p>
  <p>
    The exercise that started the project comes from the TheoLog course by Markus Krötzsch and
    Stephan Mennicke.
  </p>
</Section>

<Section name="Privacy" id="privacy">
  <ul class="plain">
    <li>
      Every machine runs in your browser. Nothing you enter in the playground is sent anywhere; a
      shared link carries the machine in its address, after the # sign, which browsers do not send
      to the server.
    </li>
    <li>
      The site sets no cookies, collects no analytics and makes no requests to other servers. Fonts,
      scripts and the WebAssembly module are all served from this site.
    </li>
    <li>
      Your browser's local storage keeps your settings, such as the colour theme. Clearing the site
      data removes them.
    </li>
    <li>
      The site is hosted on GitHub Pages. GitHub processes visitors' IP addresses in its server logs
      under its own privacy statement.
    </li>
  </ul>
</Section>

<Section name="Licence" id="licence">
  <p>
    The code is available under the <a href={blob('LICENSE-MIT')}>MIT licence</a> or the
    <a href={blob('LICENSE-APACHE')}>Apache License 2.0</a>, at your option. The Coq-BB5 enumeration
    data used in the tests is under the MIT licence; see the <a href={blob('NOTICE')}>NOTICE</a> file.
  </p>
  <Details summary="Software and typefaces">
    <table class="table">
      <thead>
        <tr><th scope="col">Name</th><th scope="col">Use</th><th scope="col">Licence</th></tr>
      </thead>
      <tbody>
        {#each SOFTWARE as s (s.name)}
          <tr><th scope="row">{s.name}</th><td>{s.use}</td><td>{s.licence}</td></tr>
        {/each}
      </tbody>
    </table>
  </Details>
</Section>

<Section name="Citation" id="citation">
  <p>
    If you use the simulator or the enumeration in your work, please cite the software. The
    repository also contains a <a href={blob('CITATION.cff')}>CITATION.cff</a> file.
  </p>
  <!-- The entry scrolls sideways on narrow screens; focusable so that keyboards can scroll it. -->
  <!-- svelte-ignore a11y_no_noninteractive_tabindex -->
  <pre class="bib" tabindex="0"><code>{BIBTEX}</code></pre>
  <p class="cite-row">
    <button class="button" type="button" onclick={copyCitation}>Copy BibTeX</button>
    <span class="muted" role="status">{copied ? 'Copied to the clipboard.' : ''}</span>
  </p>
</Section>

<Section name="Author" id="author">
  <p>
    Maurice Heide, MSc student in Data Science at ETH Zurich. More of my work is on
    <a href="https://mooofna.github.io/">my homepage</a>; the code of this site is on
    <a href={REPO}>GitHub</a>.
  </p>
</Section>

<style>
  .crates {
    max-width: 48rem;
    margin-top: var(--space-3);
  }

  .crates tbody th {
    width: 9.5rem;
    font-weight: 400;
    white-space: nowrap;
  }

  .crates .mono {
    font-size: var(--text-xs);
  }

  @media (max-width: 47.99rem) {
    .crates thead {
      display: none;
    }

    .crates tr {
      display: grid;
      padding-block: 0.6rem;
      border-bottom: 1px solid var(--rule);
    }

    .crates tr:first-child {
      border-top: 1px solid var(--rule-strong);
    }

    .crates tbody th,
    .crates td {
      width: auto;
      padding: 0;
      border: 0;
    }
  }

  .plain {
    max-width: var(--measure);
    margin: 0;
    padding: 0;
    list-style: none;
  }

  .plain li {
    padding-block: 0.6rem;
    border-top: 1px solid var(--rule);
  }

  .plain li:first-child {
    padding-top: 0;
    border-top: 0;
  }

  .bib {
    max-width: 48rem;
    margin-top: var(--space-3);
  }

  .cite-row {
    display: flex;
    flex-wrap: wrap;
    align-items: center;
    gap: var(--space-2);
  }
</style>
