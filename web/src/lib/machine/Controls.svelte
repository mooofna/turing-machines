<!--
  Run controls: Reset, Back, Step, Run or Pause, a logarithmic speed slider, and a status line.
  With `shortcuts`, Space runs or pauses, the arrow keys step and Home resets, unless the focus is
  in a field or a widget that uses those keys itself.
-->
<script lang="ts">
  import type { Snippet } from 'svelte';
  import { count } from './describe';

  interface Props {
    running: boolean;
    canStep: boolean;
    canBack: boolean;
    /** Slider position 0 to 100; 100 is the maximum speed. */
    speed: number;
    onspeed: (speed: number) => void;
    onreset: () => void;
    onback: () => void;
    onstep: () => void;
    ontoggle: () => void;
    step?: number | null;
    state?: string | null;
    /** The result, for example "Halted, accepted". */
    outcome?: string;
    /** The configuration in course notation, one line per tape. */
    configuration?: string | null;
    shortcuts?: boolean;
    disabled?: boolean;
    /** Extra controls behind a "More controls" disclosure. */
    more?: Snippet;
    moreOpen?: boolean;
  }

  let {
    running,
    canStep,
    canBack,
    speed,
    onspeed,
    onreset,
    onback,
    onstep,
    ontoggle,
    step = null,
    state = null,
    outcome = '',
    configuration = null,
    shortcuts = false,
    disabled = false,
    more,
    moreOpen = $bindable(false)
  }: Props = $props();

  const perSecond = $derived(speed >= 100 ? Infinity : Math.max(1, Math.round(10 ** (speed / 25))));
  const speedText = $derived(
    perSecond === Infinity ? 'Max' : `${count(perSecond)} step${perSecond === 1 ? '' : 's'}/s`
  );

  /** Fields and widgets that use `key` themselves. */
  function ownsKey(target: EventTarget | null, key: string): boolean {
    if (!(target instanceof Element)) return false;
    const fields =
      'input, textarea, select, [contenteditable], .cm-editor, [role="grid"], [role="menu"], [role="dialog"], [role="slider"], [role="tablist"], [role="radiogroup"], [data-own-keys]';
    if (target.closest(fields)) return true;
    // Buttons, links and disclosures activate on Space themselves.
    return key === ' ' && target.closest('button, a, summary, [role="button"]') !== null;
  }

  function keydown(event: KeyboardEvent) {
    if (!shortcuts || disabled || event.defaultPrevented) return;
    if (event.altKey || event.ctrlKey || event.metaKey) return;
    if (ownsKey(event.target, event.key)) return;
    const actions: Record<string, () => void> = {
      ' ': ontoggle,
      ArrowRight: onstep,
      ArrowLeft: onback,
      Home: onreset
    };
    const action = actions[event.key];
    if (!action) return;
    event.preventDefault();
    action();
  }
</script>

<svelte:window onkeydown={keydown} />

<div class="controls">
  <div class="buttons">
    <button class="button" type="button" onclick={onreset} disabled={disabled || !canBack}
      >Reset</button
    >
    <button class="button" type="button" onclick={onback} disabled={disabled || !canBack}
      >Back</button
    >
    <button class="button" type="button" onclick={onstep} disabled={disabled || !canStep}
      >Step</button
    >
    <button
      class="button primary run"
      type="button"
      onclick={ontoggle}
      {disabled}
      aria-keyshortcuts="Space">{running ? 'Pause' : 'Run'}</button
    >
  </div>
  <label class="speed">
    <span class="speed-label">Speed</span>
    <input
      type="range"
      min="0"
      max="100"
      step="1"
      value={speed}
      oninput={(e) => onspeed(Number(e.currentTarget.value))}
      aria-valuetext={speedText}
    />
    <span class="speed-value mono">{speedText}</span>
  </label>
</div>

<dl class="status" aria-live="polite" aria-atomic="false">
  <div class="field step">
    <dt>Step</dt>
    <dd class="mono">{step === null ? '' : count(step)}</dd>
  </div>
  <div class="field state">
    <dt>State</dt>
    <dd class="mono">{state ?? ''}</dd>
  </div>
  <div class="field result">
    <dt>Status</dt>
    <dd>{outcome}</dd>
  </div>
</dl>

{#if configuration}
  <div class="configuration">
    <span class="config-label">Configuration</span>
    <output class="mono config">{configuration}</output>
  </div>
{/if}

{#if more}
  <details class="more" bind:open={moreOpen}>
    <summary>More controls</summary>
    <div class="more-body">{@render more()}</div>
  </details>
{/if}

<style>
  .controls {
    display: flex;
    flex-wrap: wrap;
    align-items: center;
    justify-content: space-between;
    gap: var(--space-2) var(--space-3);
    margin-top: var(--space-3);
  }

  .buttons {
    display: flex;
    gap: 0.5rem;
  }

  .run {
    min-width: 5.25rem;
    justify-content: center;
  }

  .speed {
    display: flex;
    align-items: center;
    gap: 0.75rem;
    font-size: var(--text-sm);
  }

  .speed-label {
    color: var(--muted);
  }

  .speed-value {
    min-width: 7.5rem;
    font-size: var(--text-xs);
  }

  input[type='range'] {
    appearance: none;
    width: 10rem;
    height: 2.25rem;
    margin: 0;
    background: transparent;
    cursor: pointer;
  }

  input[type='range']::-webkit-slider-runnable-track {
    height: 1px;
    background: var(--rule-strong);
  }

  input[type='range']::-moz-range-track {
    height: 1px;
    background: var(--rule-strong);
  }

  input[type='range']::-webkit-slider-thumb {
    appearance: none;
    width: 0.75rem;
    height: 1.25rem;
    margin-top: calc(-0.625rem + 0.5px);
    border: 1px solid var(--rule-strong);
    border-radius: 0;
    background: var(--bg);
  }

  input[type='range']::-moz-range-thumb {
    width: 0.75rem;
    height: 1.25rem;
    border: 1px solid var(--rule-strong);
    border-radius: 0;
    background: var(--bg);
  }

  input[type='range']:focus-visible {
    outline: 2px solid var(--accent);
    outline-offset: 2px;
  }

  .status {
    display: grid;
    grid-template-columns: minmax(6rem, auto) minmax(6rem, auto) 1fr;
    gap: var(--space-3);
    margin: var(--space-3) 0 0;
    padding-top: var(--space-2);
    border-top: 1px solid var(--rule);
  }

  .field {
    min-width: 0;
  }

  dt {
    font-size: var(--text-xs);
    color: var(--muted);
  }

  dd {
    margin: 0;
    font-size: var(--text-sm);
    min-height: 1.5em;
    overflow-wrap: anywhere;
  }

  dd.mono {
    font-size: var(--text-sm);
  }

  .configuration {
    display: grid;
    gap: 0.1rem;
    margin-top: var(--space-2);
  }

  .config-label {
    font-size: var(--text-xs);
    color: var(--muted);
  }

  .config {
    display: block;
    overflow-x: auto;
    white-space: pre;
    font-size: var(--text-sm);
    padding-bottom: 0.25rem;
    scrollbar-width: thin;
  }

  .more {
    margin-top: var(--space-2);
    border-top: 1px solid var(--rule);
  }

  summary {
    cursor: pointer;
    padding-block: 0.6rem;
    font-size: var(--text-sm);
    color: var(--muted);
    width: max-content;
  }

  summary:hover {
    color: var(--text);
  }

  .more-body {
    padding-bottom: var(--space-2);
  }

  @media (max-width: 47.99rem) {
    .controls {
      flex-direction: column;
      align-items: stretch;
    }

    .buttons {
      display: grid;
      grid-template-columns: repeat(4, 1fr);
    }

    .buttons .button {
      justify-content: center;
      padding-inline: 0.25rem;
    }

    .run {
      min-width: 0;
    }

    .speed input[type='range'] {
      flex: 1;
      width: auto;
    }

    .speed-value {
      min-width: 6.5rem;
      text-align: right;
    }

    .status {
      grid-template-columns: auto auto;
      gap: var(--space-2) var(--space-3);
    }

    .result {
      grid-column: 1 / -1;
    }
  }
</style>
