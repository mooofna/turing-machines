<script lang="ts">
  import Chapter from '#lib/components/Chapter.svelte';

  let { data } = $props();
</script>

<Chapter chapter={data.chapter} content={data.content} />
