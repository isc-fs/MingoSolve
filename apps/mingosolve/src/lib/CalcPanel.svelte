<!--
    Docked calculator: always visible on the right so a quick unit-aware calculation never costs a view switch.
    Enter evaluates; results stack newest first; click a result to copy its number.
-->
<script lang="ts">
    import { calc } from './solve';
    import { session } from './session.svelte';

    let history = $state<{ expr: string; out: string }[]>([]);

    async function evaluate(): Promise<void> {
        const expr = session.calcInput.trim();
        if (expr.length === 0) return;
        const out = await calc(expr).catch((e: unknown) => `error: ${String(e)}`);
        history.unshift({ expr, out });
        history = history.slice(0, 40);
    }

    function copy(out: string): void {
        const n = out.split(' ')[0];
        void navigator.clipboard.writeText(n);
    }
</script>

<aside class="calc">
    <div class="calc-head">
        <h3>Calculator</h3>
        <span class="muted small">units welcome: <span class="mono">0.5*280kg*(100km/h)**2 -&gt; kJ</span></span>
    </div>
    <input
        class="input mono"
        placeholder="expression [-> unit]"
        bind:value={session.calcInput}
        onkeydown={(e) => {
            if (e.key === 'Enter') void evaluate();
        }}
    />
    <ul>
        {#each history as h, i (i)}
            <li>
                <button type="button" class="entry" title="Copy the number" onclick={() => copy(h.out)}>
                    <span class="mono muted small">{h.expr}</span>
                    <span class="mono result" class:error={h.out.startsWith('error')}>{h.out}</span>
                </button>
            </li>
        {/each}
    </ul>
</aside>

<style>
    .calc {
        width: 300px;
        flex: 0 0 300px;
        padding: var(--space-4) var(--space-3);
        background: var(--surface);
        border-left: 1px solid var(--border);
        display: flex;
        flex-direction: column;
        gap: var(--space-3);
        min-height: 0;
    }
    .calc-head h3 {
        margin: 0 0 2px;
        font-size: var(--text-base);
        font-weight: 600;
    }
    .small {
        font-size: var(--text-xs);
    }
    ul {
        list-style: none;
        margin: 0;
        padding: 0;
        overflow-y: auto;
        display: flex;
        flex-direction: column;
        gap: var(--space-1);
    }
    .entry {
        appearance: none;
        width: 100%;
        text-align: left;
        background: var(--bg-soft);
        border: 1px solid var(--border);
        border-radius: var(--radius-md);
        padding: var(--space-2);
        color: var(--text);
        cursor: pointer;
        display: flex;
        flex-direction: column;
        gap: 2px;
        font: inherit;
    }
    .entry:hover {
        border-color: var(--border-strong);
    }
    .result {
        color: var(--accent);
    }
    .result.error {
        color: var(--danger);
    }
</style>
