<script lang="ts">
  import { onMount } from 'svelte';
  import { goto } from '$app/navigation';
  import { galleryMachine, machines, type GalleryMachine } from '#lib/gallery.ts';
  import Controls from '#lib/machine/Controls.svelte';
  import { clipConfiguration, outcome, transition } from '#lib/machine/describe.ts';
  import { services } from '#lib/machine/services.ts';
  import TapeView from '#lib/machine/TapeView.svelte';
  import { glyph } from '#lib/playground/cell.ts';
  import EditorTabs, { type Tab } from '#lib/playground/EditorTabs.svelte';
  import MachineBar from '#lib/playground/MachineBar.svelte';
  import MoreControls from '#lib/playground/MoreControls.svelte';
  import SpaceTimeSlot from '#lib/playground/SpaceTimeSlot.svelte';
  import { DEFAULT_BUDGET, Playground, stepsPerSecond } from '#lib/playground/state.svelte.ts';
  import { parseHash } from '#lib/share.ts';

  const STORE = 'turing-machines:playground';
  const TAB_STORE = 'turing-machines:playground-tab';

  const pg = new Playground();
  let narrow = $state(false);
  let tab = $state<Tab>('diagram');
  let spaceOpen = $state(false);
  let spaceDismissed = false;
  let seenLongRun = 0;
  let loadError = $state<string | null>(null);
  let moreOpen = $state(false);
  let lastHash = '';

  const spec = $derived(pg.spec);
  const snapshot = $derived(pg.snapshot);
  const rate = $derived(stepsPerSecond(pg.speed));
  const slide = $derived(pg.running ? (rate <= 8 ? Math.min(180, 700 / rate) : 0) : 160);
  const nextText = $derived.by(() => {
    const i = snapshot?.nextTransition;
    const t = i === undefined ? undefined : spec?.transitions[i];
    return t ? transition({ ...t, read: t.read.map(glyph), write: t.write.map(glyph) }) : undefined;
  });
  const status = $derived(
    spec && spec.transitions.length === 0
      ? 'No transitions yet'
      : snapshot && spec
        ? outcome(snapshot, spec.kind, {
            running: pg.running,
            stop: pg.stop,
            rate: pg.speed >= 100 ? pg.measuredRate : 0,
            next: nextText
          })
        : ''
  );

  function read<T>(key: string): T | null {
    try {
      const raw = localStorage.getItem(key);
      return raw === null ? null : (JSON.parse(raw) as T);
    } catch {
      return null;
    }
  }

  function write(key: string, value: unknown) {
    try {
      localStorage.setItem(key, JSON.stringify(value));
    } catch {
      // Storage may be unavailable; the playground works without it.
    }
  }

  /** The machine shown on a first visit. */
  const firstMachine = galleryMachine('binary-increment') ?? machines[0];

  /** Loads a machine of the gallery; false if its description does not parse. */
  function loadEntry(entry: GalleryMachine, input?: string): boolean {
    const options = { galleryId: entry.id, input: input ?? entry.input };
    if (entry.format === 'dsl') {
      // The text as written, with its comments and layout.
      pg.load(entry.text, options);
    } else {
      const parsed = pg.services.fromStandard(entry.text);
      if (!parsed.spec) {
        loadError = `The gallery entry ${entry.id} is not a valid machine.`;
        return false;
      }
      pg.load({ ...parsed.spec, name: parsed.spec.name || entry.title }, options);
    }
    pg.budget = Math.max(DEFAULT_BUDGET, entry.budget ?? 0);
    loadError = null;
    return true;
  }

  /** Loads the machine named by a share link; false if the hash names none or it is broken. */
  function loadHash(hash: string): boolean {
    const link = parseHash(hash);
    if (!link) return false;
    const m = link.machine;
    if (m.kind === 'gallery') {
      const entry = galleryMachine(m.id);
      if (!entry) {
        loadError = `The gallery has no machine called ${m.id}.`;
        return false;
      }
      return loadEntry(entry, link.input);
    }
    const parsed =
      m.kind === 'standard' ? pg.services.fromStandard(m.text) : pg.services.decodeShare(m.payload);
    if (!parsed.spec) {
      const first = parsed.diagnostics[0]?.message;
      loadError = `The link does not contain a valid machine${first ? `: ${first}` : ''}.`;
      return false;
    }
    pg.load(parsed.spec, { input: link.input });
    loadError = null;
    return true;
  }

  function setTab(next: Tab) {
    tab = next;
    write(TAB_STORE, next);
  }

  async function share() {
    const url = pg.shareUrl();
    if (!url) return null;
    lastHash = new URL(url).hash;
    try {
      await goto(url, { replace: true, shallow: true, state: {} });
    } catch {
      // The address bar keeps its old hash; the link itself is still right.
    }
    let copied = false;
    try {
      await navigator.clipboard.writeText(url);
      copied = true;
    } catch {
      copied = false;
    }
    return { url, copied };
  }

  onMount(() => {
    const media = window.matchMedia('(max-width: 47.99rem)');
    narrow = media.matches;
    const stored = read<Tab>(TAB_STORE);
    tab = stored ?? (narrow ? 'table' : 'diagram');
    const onMedia = () => (narrow = media.matches);
    media.addEventListener('change', onMedia);

    let alive = true;
    void services().then((svc) => {
      if (!alive) return;
      pg.attach(svc);
      lastHash = window.location.hash;
      if (!loadHash(window.location.hash)) {
        const saved = read<{ source: string; input: string; galleryId?: string | null }>(STORE);
        if (saved && !loadError) {
          pg.load(saved.source, { input: saved.input, galleryId: saved.galleryId ?? undefined });
        } else if (firstMachine) loadEntry(firstMachine);
        if (!pg.spec && !loadError && firstMachine) loadEntry(firstMachine);
      }
      if (!pg.spec) tab = 'text';
    });

    const onHash = () => {
      if (window.location.hash === lastHash || !pg.ready) return;
      lastHash = window.location.hash;
      loadHash(lastHash);
    };
    window.addEventListener('hashchange', onHash);

    return () => {
      alive = false;
      media.removeEventListener('change', onMedia);
      window.removeEventListener('hashchange', onHash);
      pg.detach();
    };
  });

  // Keep the machine for the next visit.
  $effect(() => {
    const value = { source: pg.source, input: pg.input, galleryId: pg.galleryId };
    if (!pg.ready || !pg.spec) return;
    const timer = setTimeout(() => write(STORE, value), 400);
    return () => clearTimeout(timer);
  });

  // Open the space-time diagram after a long run, unless it was closed by hand.
  $effect(() => {
    const long = pg.longRun;
    if (long > seenLongRun) {
      seenLongRun = long;
      if (!spaceDismissed) spaceOpen = true;
    }
  });

  $effect(() => {
    void pg.loads;
    spaceDismissed = false;
  });

  function isTextTarget(target: EventTarget | null) {
    return (
      target instanceof Element &&
      target.closest('input, textarea, select, [contenteditable], .cm-editor') !== null
    );
  }

  function keydown(event: KeyboardEvent) {
    if (!(event.metaKey || event.ctrlKey) || event.key.toLowerCase() !== 'z') return;
    if (isTextTarget(event.target)) return;
    event.preventDefault();
    if (event.shiftKey) pg.redo();
    else pg.undo();
  }

  /** A set in the course notation; ∅ when empty. */
  function set(items: string[]) {
    return items.length ? `{${items.join(', ')}}` : '∅';
  }

  const sigma = $derived(spec ? set(spec.inputAlphabet.map(glyph)) : '');
  const blankStart = $derived(
    spec !== null && (spec.kind === 'beaver' || spec.inputAlphabet.length === 0)
  );
</script>

<svelte:window onkeydown={keydown} />

<div class="page playground">
  <header class="row head">
    <div class="label">
      <h1>Playground</h1>
    </div>
    <div class="content">
      <MachineBar {pg} onpick={(m) => loadEntry(m)} onshare={share} />
      {#if loadError}<p class="load-error" role="alert">{loadError}</p>{/if}
    </div>
  </header>

  <section class="row block" aria-labelledby="run-heading">
    <div class="label">
      <h2 id="run-heading" class="section-name">Run</h2>
      <p class="aside">
        <kbd>Space</kbd> runs or pauses, <kbd>Right</kbd> steps, <kbd>Left</kbd> steps back,
        <kbd>Home</kbd> resets.
      </p>
    </div>
    <div class="content">
      <div class="input-row">
        <label for="pg-input">Input</label>
        <input
          id="pg-input"
          class="mono"
          value={pg.input}
          placeholder={blankStart ? 'blank tape' : 'empty word'}
          autocomplete="off"
          spellcheck="false"
          autocapitalize="off"
          disabled={!spec}
          aria-invalid={pg.inputProblem ? 'true' : undefined}
          aria-describedby="pg-input-help"
          oninput={(e) => pg.setInput(e.currentTarget.value)}
          onkeydown={(e) => {
            if (e.key === 'Enter') {
              e.preventDefault();
              void pg.run();
            }
          }}
        />
        <span id="pg-input-help" class="input-help" class:problem={pg.inputProblem}>
          {#if pg.inputProblem}
            {pg.inputProblem}
          {:else if blankStart && pg.input === ''}
            This machine starts on a blank tape.
          {:else if spec}
            <span class="mono">Σ = {sigma}</span>
          {/if}
        </span>
      </div>

      <TapeView
        tapes={snapshot?.tapes ?? null}
        current={snapshot?.state ?? spec?.start ?? null}
        blank={spec?.blank ?? '␣'}
        rows={spec?.tapes ?? 1}
        {slide}
        onradius={(r) => pg.setRadius(r)}
      />

      <Controls
        running={pg.running}
        canStep={pg.canStep}
        canBack={pg.canBack}
        speed={pg.speed}
        onspeed={(s) => pg.setSpeed(s)}
        onreset={() => void pg.reset()}
        onback={() => void pg.back()}
        onstep={() => void pg.step()}
        ontoggle={() => pg.toggleRun()}
        step={snapshot?.step ?? null}
        state={snapshot?.state ?? null}
        outcome={pg.engineError ? `The engine stopped: ${pg.engineError}` : status}
        configuration={snapshot ? clipConfiguration(snapshot.configuration, snapshot.state) : null}
        shortcuts={true}
        disabled={!snapshot}
        bind:moreOpen
      >
        {#snippet more()}
          <MoreControls {pg} />
        {/snippet}
      </Controls>

      {#if snapshot && spec?.kind === 'enumerator' && snapshot.emittedCount > 0}
        <p class="emitted">
          <span class="muted">Printed {snapshot.emittedCount}:</span>
          <span class="mono">{snapshot.emitted.map((w) => (w === '' ? 'ε' : w)).join(', ')}</span>
        </p>
      {/if}
    </div>
  </section>

  <section class="row block" aria-labelledby="machine-heading">
    <div class="label">
      <h2 id="machine-heading" class="section-name">Machine</h2>
      {#if spec}
        <dl class="tuple mono">
          <div>
            <dt>Q</dt>
            <dd>{set(spec.states.map((s) => s.name))}</dd>
          </div>
          <div>
            <dt>Σ</dt>
            <dd>{sigma}</dd>
          </div>
          <div>
            <dt>Γ</dt>
            <dd>{set(spec.tapeAlphabet.map(glyph))}</dd>
          </div>
          <div>
            <dt>q<sub>0</sub></dt>
            <dd>{spec.start}</dd>
          </div>
          <div>
            <dt>F</dt>
            <dd>{set(spec.states.filter((s) => s.isFinal).map((s) => s.name))}</dd>
          </div>
          {#if spec.tapes > 1}<div>
              <dt>k</dt>
              <dd>{spec.tapes} tapes</dd>
            </div>{/if}
        </dl>
      {/if}
    </div>
    <div class="content">
      {#if pg.ready}
        <EditorTabs {pg} {tab} ontab={setTab} {narrow} />
      {:else}
        <p class="muted loading">Loading the engine.</p>
      {/if}
    </div>
  </section>

  <section class="row block last" aria-labelledby="spacetime-heading">
    <div class="label">
      <h2 id="spacetime-heading" class="section-name">Space-time</h2>
    </div>
    <div class="content">
      <details
        bind:open={spaceOpen}
        ontoggle={(e) => {
          if (!e.currentTarget.open) spaceDismissed = true;
        }}
      >
        <summary>Space-time diagram of this run</summary>
        <SpaceTimeSlot {spec} input={pg.input} steps={pg.lastRun} />
      </details>
    </div>
  </section>
</div>

<style>
  .playground {
    padding-bottom: var(--space-4);
  }

  .head {
    align-items: center;
    padding-block: var(--space-3);
  }

  h1 {
    font-size: var(--text-xl);
  }

  .load-error {
    margin: 0.5rem 0 0;
    font-size: var(--text-sm);
  }

  .block {
    padding-block: var(--space-3) var(--space-4);
    border-top: 1px solid var(--rule);
  }

  .block.last {
    padding-bottom: 0;
  }

  .section-name {
    margin-bottom: var(--space-1);
  }

  .aside {
    font-size: var(--text-xs);
    color: var(--muted);
    max-width: 16rem;
  }

  kbd {
    font-family: var(--font-mono);
    font-size: 0.95em;
    color: var(--text);
  }

  .input-row {
    display: flex;
    flex-wrap: wrap;
    align-items: center;
    gap: 0.5rem 0.75rem;
    margin-bottom: var(--space-3);
  }

  .input-row label {
    font-size: var(--text-sm);
    font-weight: 600;
  }

  .input-row input {
    flex: 0 1 18rem;
    min-width: 0;
    height: 2.25rem;
    padding: 0 0.6rem;
    border: 1px solid var(--rule-strong);
    border-radius: 0;
    background: var(--field);
    color: var(--text);
    font-size: var(--text-sm);
  }

  .input-row input:focus-visible {
    outline: 2px solid var(--accent);
    outline-offset: 1px;
  }

  .input-row input[aria-invalid='true'] {
    text-decoration: underline wavy;
    text-decoration-thickness: 1px;
    text-underline-offset: 0.3em;
  }

  .input-help {
    font-size: var(--text-sm);
    color: var(--muted);
  }

  .input-help.problem {
    color: var(--text);
  }

  .emitted {
    margin-top: var(--space-2);
    font-size: var(--text-sm);
    overflow-wrap: anywhere;
  }

  .tuple {
    display: grid;
    gap: 0.2rem;
    margin: var(--space-2) 0 0;
    font-size: var(--text-xs);
  }

  .tuple div {
    display: grid;
    grid-template-columns: 1.5rem 1fr;
    gap: 0.4rem;
  }

  .tuple dt {
    color: var(--muted);
  }

  .tuple dd {
    margin: 0;
    overflow-wrap: anywhere;
  }

  .loading {
    font-size: var(--text-sm);
  }

  summary {
    cursor: pointer;
    padding-block: 0.2rem 0.8rem;
    font-size: var(--text-sm);
    color: var(--muted);
    width: max-content;
  }

  summary:hover {
    color: var(--text);
  }

  @media (max-width: 47.99rem) {
    .head {
      row-gap: var(--space-1);
    }

    .tuple {
      display: flex;
      flex-wrap: wrap;
      gap: 0.2rem 1.25rem;
      margin-bottom: var(--space-2);
    }

    .tuple div {
      display: flex;
    }

    .aside {
      display: none;
    }
  }
</style>
