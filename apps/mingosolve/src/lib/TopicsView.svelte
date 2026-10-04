<!--
    Topics: the whole script library, full height, with the search focused (see Library.svelte).
-->
<script lang="ts">
    import Library from './Library.svelte';
    import { catalog } from './catalog.svelte';
</script>

<div class="view">
    <header>
        <h1>Topics</h1>
        <p class="muted">{catalog.scripts.size} scripts. Formulas solve for whatever you leave blank; tools run a procedure.</p>
    </header>
    <Library autofocus />
</div>

<style>
    .view {
        flex: 1;
        min-height: 0;
        overflow-y: auto;
        padding: var(--space-5) max(var(--space-5), calc((100% - 1600px) / 2)) var(--space-6);
        display: flex;
        flex-direction: column;
        gap: var(--space-4);
    }
    header {
        padding: 0 var(--space-2);
    }
</style>
