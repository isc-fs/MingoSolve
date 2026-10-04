<!--
    Chain: give what you know, name what you want; the engine applies formulas until it gets there and shows only
    the steps the answer depends on. A red banner means two formulas disagree: read it before answering.
-->
<script lang="ts">
    import { onMount } from 'svelte';

    import { chainFormulas, listFormulas } from './solve';
    import { session } from './session.svelte';
    import type { ChainResult } from './types';

    let names = $state<string[]>([]);
    let target = $state('');
    let givenText = $state('');
    let only = $state('');
    let shownIn = $state('');
    let result = $state<ChainResult | null>(null);
    let error = $state<string | null>(null);

    onMount(async () => {
        const fs = await listFormulas();
        names = [...new Set(fs.flatMap((f) => f.vars.map((v) => v.name)))].sort();
        const req = session.request;
        if (req !== null && req.view === 'chain') {
            session.request = null;
            target = req.id;
            givenText = req.values.map(([k, v]) => `${k} = ${v}`).join('\n');
            shownIn = req.display?.[req.id] ?? '';
            void run();
        }
    });

    function parseGiven(): [string, string][] {
        return givenText
            .split(/\n|;/)
            .map((l) => l.trim())
            .filter((l) => l.includes('='))
            .map((l) => {
                const i = l.indexOf('=');
                return [l.slice(0, i).trim(), l.slice(i + 1).trim()] as [string, string];
            });
    }

    async function run(): Promise<void> {
        if (target.trim().length === 0) return;
        try {
            const tags = only.split(/[,\s]+/).filter((t) => t.length > 0);
            result = await chainFormulas(target.trim(), parseGiven(), tags, { [target.trim()]: shownIn });
            error = null;
        } catch (e) {
            result = null;
            error = String(e);
        }
    }
</script>

<section class="view">
    <header>
        <div>
            <h2>Chain</h2>
            <p>
                One value per line (<span class="mono">h_cg = 0.205 m</span>). The engine chains every formula it
                can until the target is known. Restrict with tags (<span class="mono">dynamics, aero</span>) if two
                formulas share a variable name with different meanings.
            </p>
        </div>
    </header>

    <div class="card stack">
        <div class="row inputs">
            <label class="field grow">
                <span>Target variable</span>
                <input type="text" class="mono" list="var-names" bind:value={target} placeholder="v" />
            </label>
            <label class="field">
                <span>Show in</span>
                <input type="text" class="mono" bind:value={shownIn} placeholder="km/h" />
            </label>
            <label class="field grow">
                <span>Only formulas tagged</span>
                <input type="text" class="mono" bind:value={only} placeholder="all" />
            </label>
        </div>
        <datalist id="var-names">
            {#each names as n (n)}<option value={n}></option>{/each}
        </datalist>
        <label class="field">
            <span>Known values</span>
            <textarea rows="6" class="mono" bind:value={givenText} placeholder="h_cg = 0.205 m&#10;R_c = 14.5 m&#10;t_tr = 1.24 m"></textarea>
        </label>
        <div class="row">
            <button type="button" class="btn btn-primary" onclick={run}>Chain</button>
        </div>
    </div>

    {#if error !== null}
        <div class="banner banner-danger">{error}</div>
    {/if}
    {#if result !== null}
        <div class="card stack">
            {#if result.conflicts.length > 0}
                <div class="banner banner-danger">
                    <strong>Formulas disagree.</strong> A variable is probably used with two meanings, or the inputs
                    contradict each other:
                    <ul class="mono small">{#each result.conflicts as c (c)}<li>{c}</li>{/each}</ul>
                </div>
            {/if}
            {#if result.defaults.length > 0}
                <p class="muted small">Defaults used: {result.defaults.map((d) => `${d.name} = ${d.shown}`).join(', ')}</p>
            {/if}
            <ol class="steps mono">
                {#each result.steps as s, i (i)}
                    <li><span class="muted">[{s.formula}]</span> {s.var} = {s.shown}</li>
                {/each}
            </ol>
            {#if result.reached}
                <p class="target mono">{target} = {result.target}</p>
            {:else}
                <div class="banner banner-warning">
                    Could not reach <span class="mono">{target}</span>. Known: <span class="mono">{result.known.join(', ')}</span>
                </div>
            {/if}
        </div>
    {/if}
</section>

<style>
    .inputs {
        align-items: flex-end;
        gap: var(--space-3);
    }
    .grow {
        flex: 1;
    }
    .small {
        font-size: var(--text-xs);
    }
    .steps {
        margin: 0;
        padding-left: var(--space-5);
        display: flex;
        flex-direction: column;
        gap: var(--space-1);
    }
    .target {
        margin: 0;
        font-size: var(--text-xl);
        color: var(--accent);
    }
</style>
