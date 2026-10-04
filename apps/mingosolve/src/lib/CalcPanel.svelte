<!--
    Calculator, docked on the right and collapsible: unit-aware expressions (0.5*280kg*(100km/h)**2 -> kJ). Enter
    evaluates; results stack newest first; click one to copy its number.
-->
<script lang="ts">
    import Icon from './Icon.svelte';
    import { calc } from './solve';
    import { session } from './session.svelte';
    import { settings } from './settings.svelte';

    let history = $state<{ expr: string; out: string }[]>([]);

    async function evaluate(): Promise<void> {
        const expr = session.calcInput.trim();
        if (expr.length === 0) return;
        const out = await calc(expr).catch((e: unknown) => `error: ${String(e)}`);
        history = [{ expr, out }, ...history].slice(0, 40);
    }
</script>

{#if settings.calcOpen}
    <aside class="calc glass" aria-label="Calculator">
        <div class="head">
            <Icon name="calc" />
            <h3>Calculator</h3>
            <button type="button" class="btn btn-ghost btn-sm" onclick={() => (settings.calcOpen = false)} aria-label="Hide calculator">
                <Icon name="x" size={14} />
            </button>
        </div>
        <input autocomplete="off" autocorrect="off" autocapitalize="off" spellcheck="false"
            class="input mono"
            placeholder="2*pi*9.125m/4.9s -> km/h"
            bind:value={session.calcInput}
            onkeydown={(e) => e.key === 'Enter' && evaluate()}
            aria-label="Expression"
        />
        <ul>
            {#each history as h, i (i)}
                <li>
                    <button type="button" class="entry" title="Copy the number" onclick={() => navigator.clipboard.writeText(h.out.split(' ')[0])}>
                        <span class="mono muted small">{h.expr}</span>
                        <span class="mono result" class:error={h.out.startsWith('error')}>{h.out}</span>
                    </button>
                </li>
            {/each}
        </ul>
        {#if history.length === 0}
            <p class="muted small">Units work everywhere: <span class="mono">km/h</span>, <span class="mono">rpm</span>, <span class="mono">bar</span>, <span class="mono">Ah</span>, <span class="mono">g0</span>. End with <span class="mono">-&gt; unit</span> to convert.</p>
        {/if}
    </aside>
{:else}
    <button type="button" class="open glass" onclick={() => (settings.calcOpen = true)} aria-label="Show calculator" title="Calculator">
        <Icon name="calc" />
    </button>
{/if}

<style>
    .calc {
        width: 280px;
        flex: 0 0 280px;
        padding: var(--space-4);
        border-width: 0 0 0 1px;
        border-radius: 0;
        box-shadow: none;
        display: flex;
        flex-direction: column;
        gap: var(--space-3);
        min-height: 0;
    }
    .head {
        display: flex;
        align-items: center;
        gap: var(--space-2);
        color: var(--ink-accent);
    }
    .head h3 {
        flex: 1;
        color: var(--text);
    }
    ul {
        list-style: none;
        margin: 0;
        padding: 0;
        overflow-y: auto;
        display: flex;
        flex-direction: column;
        gap: 6px;
    }
    .entry {
        appearance: none;
        width: 100%;
        text-align: left;
        padding: var(--space-2) var(--space-3);
        border-radius: var(--r-md);
        border: 1px solid var(--well-edge);
        background: var(--well);
        cursor: pointer;
        display: flex;
        flex-direction: column;
        gap: 2px;
    }
    .entry:hover {
        border-color: var(--accent-edge);
    }
    .result {
        color: var(--ink-accent);
        font-size: var(--text-lg);
    }
    .result.error {
        color: var(--bad);
        font-size: var(--text-sm);
    }
    .open {
        align-self: flex-start;
        margin: var(--space-3) var(--space-3) 0 0;
        width: 44px;
        height: 44px;
        border-radius: 14px;
        display: grid;
        place-items: center;
        cursor: pointer;
        color: var(--ink-accent);
    }
</style>
