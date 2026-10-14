<!--
  Stands in for an interactive figure that is not built yet: an empty frame with the figure's
  aspect ratio, so that the page keeps its final layout. The caption is shown by the chapter.
-->
<script lang="ts">
  import type { FigureProps } from './registry.ts';

  let { id, aspect, aspectPhone }: FigureProps = $props();
</script>

<div
  class="frame placeholder"
  data-figure={id}
  style:--aspect={aspect}
  style:--aspect-phone={aspectPhone}
  aria-hidden="true"
></div>

<style>
  .placeholder {
    width: 100%;
    aspect-ratio: var(--aspect);
  }

  @media (max-width: 47.99rem) {
    .placeholder {
      aspect-ratio: var(--aspect-phone);
    }
  }
</style>
