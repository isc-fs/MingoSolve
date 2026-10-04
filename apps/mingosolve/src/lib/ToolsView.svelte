<!--
    Tools: procedural helpers that aren't a single equation (event scoring incl. inverse cones and DV/DC, rule
    tables, exact circuit analysis, E-series, CAN/UART timing...). Parameters left blank use the tool's default;
    the rule year defaults to Settings.
-->
<script lang="ts">
    import { onMount } from 'svelte';

    import { listTools, runTool } from './tools';
    import { session } from './session.svelte';
    import { settings } from './settings.svelte';
    import type { ToolInfo } from './types';

    let tools = $state<ToolInfo[]>([]);
    let filter = $state('');
    let current = $state<ToolInfo | null>(null);
    let args = $state<Record<string, string>>({});
    let output = $state<string | null>(null);
    let error = $state<string | null>(null);

    const shown = $derived(
        tools.filter((t) => `${t.name} ${t.doc}`.toLowerCase().includes(filter.toLowerCase())),
    );

    onMount(async () => {
        tools = await listTools();
        const req = session.request;
        if (req !== null && req.view === 'tools') {
            session.request = null;
            const t = tools.find((x) => x.name === req.id);
            if (t !== undefined) {
                choose(t);
                (req.positional ?? []).forEach((v, i) => {
                    const p = t.params[i];
                    if (p !== undefined) args[p.name] = v;
                });
                for (const [k, v] of req.values) args[k] = v;
                void run();
            }
        }
    });

    function choose(t: ToolInfo): void {
        current = t;
        args = Object.fromEntries(t.params.map((p) => [p.name, '']));
        if (t.params.some((p) => p.name === 'rules')) args.rules = settings.rules;
        output = null;
        error = null;
    }

    async function run(): Promise<void> {
        if (current === null) return;
        try {
            output = await runTool(current.name, Object.entries(args));
            error = null;
        } catch (e) {
            output = null;
            error = String(e);
        }
    }
</script>

<section class="view">
    <header>
        <div>
            <h2>Tools</h2>
            <p>Scoring under the legacy, 2026 and 2027 rules, rule tables, exact nodal circuit analysis and other helpers.</p>
        </div>
    </header>

    <div class="cols">
        <div class="card card-tight stack stack-sm">
            <input class="input" placeholder="filter tools" bind:value={filter} />
            <ul class="list">
                {#each shown as t (t.name)}
                    <li>
                        <button type="button" class="item" class:active={current?.name === t.name} onclick={() => choose(t)}>
                            <span class="mono">{t.name}</span>
                        </button>
                    </li>
                {/each}
            </ul>
        </div>

        {#if current === null}
            <div class="banner banner-info">Pick a tool on the left.</div>
        {:else}
            <div class="card stack">
                <div class="card-header"><h3 class="mono">{current.name}</h3></div>
                <p class="muted doc">{current.doc}</p>
                <form
                    class="grid"
                    onsubmit={(e) => {
                        e.preventDefault();
                        void run();
                    }}
                >
                    {#each current.params as p (p.name)}
                        <label class="field" class:wide={p.name === 'netlist' || p.name === 'levels' || p.name === 'teeth'}>
                            <span class="mono">{p.name}</span>
                            <input
                                type="text"
                                class="mono"
                                placeholder={p.default === null ? 'required' : p.default === '' ? '(none)' : `default ${p.default}`}
                                bind:value={args[p.name]}
                            />
                        </label>
                    {/each}
                    <div class="row submit"><button type="submit" class="btn btn-primary">Run</button></div>
                </form>
                {#if error !== null}
                    <div class="banner banner-danger">{error}</div>
                {/if}
                {#if output !== null}
                    <pre class="out mono">{output}</pre>
                {/if}
            </div>
        {/if}
    </div>
</section>

<style>
    .cols {
        display: grid;
        grid-template-columns: 240px 1fr;
        gap: var(--space-4);
        align-items: start;
    }
    .list {
        list-style: none;
        margin: 0;
        padding: 0;
        display: flex;
        flex-direction: column;
        gap: 2px;
        max-height: 60vh;
        overflow-y: auto;
    }
    .item {
        appearance: none;
        width: 100%;
        text-align: left;
        background: transparent;
        border: none;
        color: var(--text);
        padding: var(--space-1) var(--space-2);
        border-radius: var(--radius-md);
        cursor: pointer;
        font: inherit;
        font-size: var(--text-sm);
    }
    .item:hover {
        background: var(--hover);
    }
    .item.active {
        background: var(--hover);
        color: var(--accent);
    }
    .doc {
        margin: 0;
        line-height: 1.5;
    }
    .grid {
        display: grid;
        grid-template-columns: repeat(auto-fill, minmax(180px, 1fr));
        gap: var(--space-3);
    }
    .wide {
        grid-column: 1 / -1;
    }
    .submit {
        grid-column: 1 / -1;
    }
    .out {
        margin: 0;
        padding: var(--space-3);
        background: var(--code-bg);
        border-radius: var(--radius-md);
        color: var(--accent);
        font-size: var(--text-lg);
        white-space: pre-wrap;
    }
</style>
