<!--
  The plate of an interactive figure in chapters 1 to 3: a hairline frame around the figure, and
  the small vocabulary the figures share (labels, fields, sliders, hints), so that they read as
  one set. `busy` marks the figure as not ready (aria-busy): from the first render until the
  engine has delivered what the figure shows, including before it scrolls into view. The tests
  and the screenshot script wait for it to clear.

  Usage: <Plate id="fig1.1" busy={loading}>...</Plate>.
-->
<script lang="ts">
  import type { Snippet } from 'svelte';
  import type { Attachment } from 'svelte/attachments';

  interface Props {
    /** Registry id of the figure. */
    id: string;
    /** The figure has not finished loading or computing. */
    busy?: boolean;
    /** Attached to the frame, for example to start work when it scrolls into view. */
    attach?: Attachment<HTMLElement>;
    class?: string;
    children: Snippet;
  }

  let { id, busy = false, attach, class: className = '', children }: Props = $props();

  const none: Attachment<HTMLElement> = () => undefined;
</script>

<div class="plate {className}" data-figure={id} aria-busy={busy} {@attach attach ?? none}>
  {@render children()}
</div>

<style>
  .plate {
    position: relative;
    padding: var(--space-3);
    border: 1px solid var(--rule);
    background: var(--bg);
    font-size: var(--text-sm);
    line-height: 1.45;
  }

  @media (max-width: 47.99rem) {
    .plate {
      padding: var(--space-2) 0;
      border-inline: 0;
    }
  }

  /* Small labels above values and controls. */
  .plate :global(.fig-label) {
    display: block;
    margin: 0 0 0.3rem;
    font-size: var(--text-xs);
    font-weight: 400;
    line-height: 1.3;
    color: var(--muted);
  }

  /* Text and number fields: square, hairline, mono. */
  .plate :global(.fig-field) {
    height: 2.25rem;
    min-width: 0;
    padding: 0 0.6rem;
    border: 1px solid var(--rule-strong);
    border-radius: 0;
    background: var(--field);
    color: var(--text);
    font-family: var(--font-mono);
    font-size: var(--text-sm);
    font-variant-numeric: tabular-nums;
  }

  .plate :global(.fig-field:focus-visible) {
    outline: 2px solid var(--accent);
    outline-offset: 1px;
  }

  .plate :global(.fig-field[aria-invalid='true']) {
    text-decoration: underline wavy;
    text-decoration-thickness: 1px;
    text-underline-offset: 0.3em;
  }

  .plate :global(.fig-field[type='number']) {
    appearance: textfield;
    -moz-appearance: textfield;
  }

  .plate :global(.fig-field[type='number']::-webkit-inner-spin-button),
  .plate :global(.fig-field[type='number']::-webkit-outer-spin-button) {
    appearance: none;
    margin: 0;
  }

  /* Range sliders: a hairline track and a square thumb, as in the playground. */
  .plate :global(input[type='range']) {
    appearance: none;
    height: 2.25rem;
    margin: 0;
    background: transparent;
    cursor: pointer;
  }

  .plate :global(input[type='range']:disabled) {
    cursor: default;
    opacity: 0.45;
  }

  .plate :global(input[type='range']::-webkit-slider-runnable-track) {
    height: 1px;
    background: var(--rule-strong);
  }

  .plate :global(input[type='range']::-moz-range-track) {
    height: 1px;
    background: var(--rule-strong);
  }

  .plate :global(input[type='range']::-webkit-slider-thumb) {
    appearance: none;
    width: 0.75rem;
    height: 1.25rem;
    margin-top: calc(-0.625rem + 0.5px);
    border: 1px solid var(--rule-strong);
    border-radius: 0;
    background: var(--bg);
  }

  .plate :global(input[type='range']::-moz-range-thumb) {
    width: 0.75rem;
    height: 1.25rem;
    border: 1px solid var(--rule-strong);
    border-radius: 0;
    background: var(--bg);
  }

  .plate :global(input[type='range']:focus-visible) {
    outline: 2px solid var(--accent);
    outline-offset: 2px;
  }

  /* Hints and problems below a field. */
  .plate :global(.fig-hint) {
    font-size: var(--text-xs);
    color: var(--muted);
  }

  .plate :global(.fig-hint.problem) {
    color: var(--text);
  }

  /* A line of text that reports what the figure shows. */
  .plate :global(.fig-status) {
    margin: 0;
    font-size: var(--text-sm);
    color: var(--muted);
  }

  .plate :global(.fig-status strong) {
    font-weight: 600;
    color: var(--text);
  }

  .plate :global(.fig-error) {
    margin: 0;
    font-size: var(--text-sm);
    color: var(--text);
  }
</style>
