<!--
    A fold: a button that shows or hides its content, with the state announced (aria-expanded) and the content
    reachable by the keyboard like any button. Used for the "Past questions" lists; the caller keeps the open state
    (bind:open) so it can be remembered.
-->
<script lang="ts">
    import type { Snippet } from 'svelte';

    import Icon from './Icon.svelte';

    let { label, id, open = $bindable(false), children }: { label: string; id: string; open?: boolean; children: Snippet } = $props();
</script>

<div class="disclosure">
    <button type="button" class="head" aria-expanded={open} aria-controls={id} onclick={() => (open = !open)}>
        <span class="chev" class:open><Icon name="chevron" size={14} /></span>
        <span class="label">{label}</span>
    </button>
    <div {id} class="body" hidden={!open}>
        {@render children()}
    </div>
</div>

<style>
    .head {
        appearance: none;
        display: inline-flex;
        align-items: center;
        gap: var(--space-2);
        padding: 4px var(--space-2) 4px 0;
        border: none;
        background: transparent;
        color: var(--text-2);
        font: inherit;
        cursor: pointer;
        border-radius: var(--r-sm);
    }
    .head:hover {
        color: var(--text);
    }
    .chev {
        display: inline-flex;
        transition: transform var(--motion);
    }
    .chev.open {
        transform: rotate(90deg);
    }
    .body {
        padding-top: var(--space-2);
    }
    .body[hidden] {
        display: none;
    }
    @media (prefers-reduced-motion: reduce) {
        .chev {
            transition: none;
        }
    }
</style>
