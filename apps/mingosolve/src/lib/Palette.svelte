<!--
    Command palette (⌘K / Ctrl K): type to find any script by name, topic or variable; arrows to move, Enter to
    open, Esc to close. Typing a long sentence here searches it as a pasted problem instead.
-->
<script lang="ts">
    import { tick } from 'svelte';

    import Icon from './Icon.svelte';
    import { searchScripts } from './catalog.svelte';
    import { openScript, session } from './session.svelte';

    let query = $state('');
    let index = $state(0);
    let input = $state<HTMLInputElement | null>(null);

    const looksLikeProblem = $derived(query.trim().split(/\s+/).length >= 6);
    const results = $derived(looksLikeProblem ? [] : searchScripts(query, 12));

    $effect(() => {
        if (session.paletteOpen) {
            query = '';
            index = 0;
            void tick().then(() => input?.focus());
        }
    });

    function choose(i: number): void {
        if (looksLikeProblem) {
            session.problem = query;
            session.activeView = 'solve';
        } else {
            const r = results[i];
            if (r === undefined) return;
            openScript(r.id);
        }
        session.paletteOpen = false;
    }

    function key(e: KeyboardEvent): void {
        if (e.key === 'Escape') session.paletteOpen = false;
        else if (e.key === 'ArrowDown') {
            index = Math.min(index + 1, Math.max(results.length - 1, 0));
            e.preventDefault();
        } else if (e.key === 'ArrowUp') {
            index = Math.max(index - 1, 0);
            e.preventDefault();
        } else if (e.key === 'Enter') choose(index);
    }
</script>

{#if session.paletteOpen}
    <div class="scrim" role="presentation" onclick={() => (session.paletteOpen = false)}></div>
    <div class="palette glass" role="dialog" aria-modal="true" aria-label="Find a script">
        <div class="bar">
            <Icon name="search" />
            <input autocomplete="off" autocorrect="off" autocapitalize="off" spellcheck="false"
                bind:this={input}
                bind:value={query}
                oninput={() => (index = 0)}
                onkeydown={key}
                placeholder="Spring rate, discharge, skidpad score… or paste a whole problem"
                aria-label="Search scripts"
            />
            <kbd>Esc</kbd>
        </div>
        {#if looksLikeProblem}
            <button type="button" class="row on" onclick={() => choose(0)}>
                <Icon name="spark" size={16} />
                <span class="title">Find the scripts for this problem</span>
                <span class="muted small">Enter</span>
            </button>
        {:else}
            <ul role="listbox" aria-label="Scripts">
                {#each results as r, i (r.id)}
                    <li role="option" aria-selected={i === index}>
                        <button type="button" class="row" class:on={i === index} onmouseenter={() => (index = i)} onclick={() => choose(i)}>
                            <span class="dot" data-hue={r.topic?.hue ?? 'green'}></span>
                            <span class="title">{r.title}</span>
                            <span class="muted small">{r.topic?.name ?? ''}</span>
                        </button>
                    </li>
                {/each}
                {#if results.length === 0}
                    <li class="empty muted">No script matches. Try fewer words, or a variable like <span class="mono">k_s</span>.</li>
                {/if}
            </ul>
        {/if}
    </div>
{/if}

<style>
    .scrim {
        position: fixed;
        inset: 0;
        z-index: 40;
        background: rgba(0, 0, 0, 0.32);
    }
    .palette {
        position: fixed;
        z-index: 41;
        top: 12vh;
        left: 50%;
        transform: translateX(-50%);
        width: min(640px, 90vw);
        border-radius: var(--r-xl);
        padding: var(--space-2);
        background: var(--glass-strong);
    }
    .bar {
        display: flex;
        align-items: center;
        gap: var(--space-3);
        padding: var(--space-2) var(--space-3) var(--space-3);
        border-bottom: 1px solid var(--glass-edge);
        color: var(--muted);
    }
    .bar input {
        flex: 1;
        border: none;
        background: transparent;
        color: var(--text);
        font: inherit;
        font-size: var(--text-lg);
        outline: none;
    }
    ul {
        list-style: none;
        margin: var(--space-2) 0 0;
        padding: 0;
        max-height: 52vh;
        overflow-y: auto;
    }
    .row {
        appearance: none;
        width: 100%;
        display: flex;
        align-items: center;
        gap: var(--space-3);
        padding: 10px var(--space-3);
        border: none;
        border-radius: var(--r-md);
        background: transparent;
        cursor: pointer;
        text-align: left;
    }
    .row.on {
        background: var(--selected);
    }
    .title {
        flex: 1;
    }
    .dot {
        width: 8px;
        height: 8px;
        border-radius: 50%;
        flex: none;
        background: var(--hue-green);
    }
    .dot[data-hue='teal'] {
        background: var(--hue-teal);
    }
    .dot[data-hue='gold'] {
        background: var(--hue-gold);
    }
    .empty {
        padding: var(--space-4) var(--space-3);
        font-size: var(--text-sm);
    }
</style>
