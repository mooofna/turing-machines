<!--
  A row of square segments for choosing one of a few options, built from native radio buttons:
  Tab enters the group, the arrow keys move the choice, and screen readers announce the legend,
  the option and its position. The chosen segment is filled with ink.

  Usage: <Segmented legend="States" name="fig5-1-states" options={[{ value: 2, label: '2' }]}
  bind:value={states} />.
-->
<script lang="ts" generics="T extends string | number">
  interface Option {
    value: T;
    /** Visible text of the segment. */
    label: string;
    /** Accessible name, if the visible text is terse (for example `S(4)`). */
    name?: string;
  }

  interface Props {
    /** Name of the choice, shown before the segments. */
    legend: string;
    /** Unique name of the radio group on the page. */
    name: string;
    options: readonly Option[];
    value: T;
    disabled?: boolean;
    /** Called after the choice changed through the control. */
    onchange?: (value: T) => void;
  }

  let { legend, name, options, value = $bindable(), disabled = false, onchange }: Props = $props();
</script>

<fieldset class="segmented" {disabled}>
  <legend class="legend">{legend}</legend>
  <div class="options">
    {#each options as option (option.value)}
      <label class="option" class:checked={value === option.value}>
        <input
          type="radio"
          {name}
          value={option.value}
          bind:group={value}
          aria-label={option.name}
          onchange={() => onchange?.(option.value)}
        />
        <span class="text">{option.label}</span>
      </label>
    {/each}
  </div>
</fieldset>

<style>
  .segmented {
    display: flex;
    flex-wrap: wrap;
    align-items: center;
    gap: 0.5rem 0.9rem;
    min-width: 0;
    margin: 0;
    padding: 0;
    border: 0;
  }

  .legend {
    float: left;
    padding: 0;
    font-size: var(--text-sm);
    color: var(--muted);
  }

  .options {
    display: flex;
    flex-wrap: wrap;
  }

  .option {
    position: relative;
    display: inline-flex;
    align-items: center;
    justify-content: center;
    min-width: 2.5rem;
    min-height: 2.5rem;
    margin-left: -1px;
    padding: 0 0.8rem;
    border: 1px solid var(--rule-strong);
    background: var(--bg);
    color: var(--text);
    font-size: var(--text-sm);
    font-variant-numeric: tabular-nums;
    line-height: 1;
    white-space: nowrap;
    cursor: pointer;
  }

  .option:first-child {
    margin-left: 0;
  }

  .option:hover {
    color: var(--accent);
  }

  .option.checked {
    background: var(--text);
    color: var(--bg);
  }

  .option:has(input:focus-visible) {
    z-index: 1;
    outline: 2px solid var(--accent);
    outline-offset: 2px;
  }

  input {
    position: absolute;
    inset: 0;
    margin: 0;
    opacity: 0;
    cursor: pointer;
  }

  fieldset:disabled .option {
    cursor: default;
    opacity: 0.45;
  }

  fieldset:disabled input {
    cursor: default;
  }
</style>
