<!--
  A number computed live in this browser, in mono with tabular figures.

  While `pending` is true the value is shown in the muted colour (or the word "computing" if
  there is no value yet), so a reader can tell a result from a value that is still changing. The
  muted state breathes slowly, except under prefers-reduced-motion.
  Usage: <Live value={steps} pending={running} />.
-->
<script module lang="ts">
  const integer = new Intl.NumberFormat('en-US', { maximumFractionDigits: 0 });

  /** Groups digits with commas, as everywhere on the site: 47,176,870. */
  export function formatInteger(n: number): string {
    return integer.format(n);
  }

  /** A duration measured in this browser: 41 ms, 1.27 s. */
  export function formatDuration(ms: number): string {
    if (ms < 1) return '< 1 ms';
    if (ms < 1000) return `${Math.round(ms)} ms`;
    if (ms < 60_000) return `${(ms / 1000).toFixed(2)} s`;
    return `${(ms / 60_000).toFixed(1)} min`;
  }
</script>

<script lang="ts">
  interface Props {
    /** The value; numbers are grouped with commas unless `format` is given. */
    value?: number | string | null;
    /** The value is still being computed. */
    pending?: boolean;
    /** Formats numeric values. */
    format?: (n: number) => string;
    /** Shown while there is no value yet. */
    placeholder?: string;
  }

  let {
    value = null,
    pending = false,
    format = formatInteger,
    placeholder = 'computing'
  }: Props = $props();

  const text = $derived(
    value === null || value === undefined
      ? placeholder
      : typeof value === 'number'
        ? format(value)
        : value
  );
  const empty = $derived(value === null || value === undefined);
</script>

<span class="live mono" class:pending={pending || empty} class:empty aria-busy={pending}
  >{text}</span
>

<style>
  .live {
    white-space: nowrap;
  }

  .pending {
    color: var(--muted);
  }

  .pending.empty {
    animation: breathe 1.6s ease-in-out infinite alternate;
  }

  @keyframes breathe {
    from {
      color: var(--muted);
    }

    to {
      color: var(--text);
    }
  }

  @media (prefers-reduced-motion: reduce) {
    .pending.empty {
      animation: none;
    }
  }
</style>
