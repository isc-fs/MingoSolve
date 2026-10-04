<!--
    Typeset LaTeX from the engine (src/latex.rs) with KaTeX, bundled for offline use. MathML is emitted alongside
    the visual rendering so screen readers read the formula, not the markup.
-->
<script lang="ts">
    import katex from 'katex';

    let { tex, display = false }: { tex: string; display?: boolean } = $props();

    const html = $derived(katex.renderToString(tex, { throwOnError: false, displayMode: display, output: 'htmlAndMathml' }));
</script>

<span class="tex">{@html html}</span>
