<!--
  A busy beaver value such as S(4) or Σ(3), or a single variable such as a, as MathML, so that it
  matches the mathematics of the chapter text.
  Usage: <Fn name="S" arg={4} />, <Fn name="a" />.
-->
<script lang="ts">
  interface Props {
    /** A Latin letter (set in italics) or a capital Greek letter (set upright). */
    name: string;
    /** The argument in parentheses, if any. */
    arg?: number | string;
  }

  let { name, arg }: Props = $props();

  const upright = $derived(/^[Α-Ω]$/.test(name));
</script>

<math
  ><mi mathvariant={upright ? 'normal' : undefined}>{name}</mi>{#if arg !== undefined}<mo
      stretchy="false">(</mo
    ><mn>{arg}</mn><mo stretchy="false">)</mo>{/if}</math
>
