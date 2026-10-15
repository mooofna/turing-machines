<!--
  A reference by author and year, linking to the entry in the bibliography on the About page.

  Figures cite this way instead of with numbers: the numbers in a chapter count the sources in
  the order the text cites them, and a figure cannot know that order.
  Usage: <SourceLink id="rado1962" />, or with a label: <SourceLink id="coqbb5" label="Coq-BB5" />.
-->
<script lang="ts">
  import { resolve } from '$app/paths';
  import { shortCitation, sourceAnchor, sourceById } from '#lib/components/bibliography.ts';

  interface Props {
    id: string;
    /** Visible text; by default the family names of up to two authors and the year. */
    label?: string;
  }

  let { id, label }: Props = $props();

  const source = $derived(sourceById(id));

  /** "Radó 1962", "Lin and Radó 1965", "bbchallenge 2025". */
  function authorYear(): string {
    if (!source) return id;
    const authors = source.authors
      .split(/,\s*|\s+and\s+/)
      .map((a) => a.trim())
      .filter(Boolean);
    const family = (name: string) =>
      /collaboration|contributors/i.test(name)
        ? name.replace(/^the\s+/i, '').split(/\s+/)[0]!
        : (name.split(/\s+/).at(-1) ?? name);
    const names =
      authors.length === 2
        ? `${family(authors[0]!)} and ${family(authors[1]!)}`
        : family(authors[0] ?? source.authors);
    return `${names} ${source.year}`;
  }

  const text = $derived(label ?? authorYear());
  const title = $derived(source ? `${shortCitation(source)}: ${source.title}` : id);
</script>

<a class="source" href="{resolve('/about')}#{sourceAnchor(id)}" {title}>{text}</a>

<style>
  .source {
    color: var(--muted);
    text-decoration-color: color-mix(in srgb, currentColor 30%, transparent);
  }

  .source:hover {
    color: var(--accent);
  }
</style>
