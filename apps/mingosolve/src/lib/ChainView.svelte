<!--
    Chain: list what you know, name what you want; the engine applies formulas until it gets there and shows only
    the steps the answer depends on. A red note means two formulas disagree: read it before using the number.
-->
<script lang="ts">
    import Tex from './Tex.svelte';
    import { chainFormulas } from './solve';
    import { catalog } from './catalog.svelte';
    import type { ChainResult } from './types';

    let target = $state('');
    let givenText = $state('');
    let only = $state('');
    let shownIn = $state('');
    let result = $state<ChainResult | null>(null);
    let error = $state<string | null>(null);

    const names = $derived(
        [...new Set([...catalog.scripts.values()].flatMap((s) => s.formula?.vars.map((v) => v.name) ?? []))].sort(),
    );

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

<div class="view">
    <header>
        <h1>Chain</h1>
        <p class="muted">
            For problems that need several formulas. One known value per line
            (<span class="mono">h_cg = 0.205 m</span>); the engine finds a path to the target.
        </p>
    </header>

    <section class="glass panel">
        <div class="row">
            <label class="field grow">
                <span class="label">Target</span>
                <input autocomplete="off" autocorrect="off" autocapitalize="off" spellcheck="false" class="input mono" list="var-names" bind:value={target} placeholder="v" onkeydown={(e) => e.key === 'Enter' && run()} />
            </label>
            <label class="field">
                <span class="label">Show in</span>
                <input autocomplete="off" autocorrect="off" autocapitalize="off" spellcheck="false" class="input mono" bind:value={shownIn} placeholder="km/h" />
            </label>
            <label class="field grow">
                <span class="label">Only formulas tagged</span>
                <input autocomplete="off" autocorrect="off" autocapitalize="off" spellcheck="false" class="input mono" bind:value={only} placeholder="all (or: dynamics, aero)" />
            </label>
        </div>
        <datalist id="var-names">{#each names as n (n)}<option value={n}></option>{/each}</datalist>
        <label class="field">
            <span class="label">Known values</span>
            <textarea autocomplete="off" autocapitalize="off" spellcheck="false" class="input mono" rows="5" bind:value={givenText} placeholder={'h_cg = 0.205 m\nR_c = 14.5 m\nt_tr = 1.24 m'}></textarea>
        </label>
        <div><button type="button" class="btn btn-primary" onclick={run}>Chain</button></div>
    </section>

    {#if error !== null}<p class="note note-bad"><strong>Can't chain that.</strong> {error}</p>{/if}

    {#if result !== null}
        <section class="glass panel">
            {#if result.conflicts.length > 0}
                <div class="note note-bad">
                    <strong>Formulas disagree.</strong> A variable may mean two things here, or the inputs contradict.
                    <ul class="mono small">{#each result.conflicts as c (c)}<li>{c}</li>{/each}</ul>
                </div>
            {/if}
            <ol class="steps">
                {#each result.steps as s, i (i)}
                    <li><span class="muted small">{catalog.scripts.get(s.formula)?.title ?? s.formula}</span><span class="mono"><Tex tex={catalog.scripts.get(s.formula)?.formula?.vars.find((v) => v.name === s.var)?.tex ?? s.var} /> = {s.shown}</span></li>
                {/each}
            </ol>
            {#if result.reached}
                <div class="answer"><span class="mono a-label">{target} =</span><span class="a-value">{result.target}</span></div>
            {:else}
                <p class="note note-warn"><strong>No path to {target}.</strong> Known: <span class="mono">{result.known.join(', ')}</span></p>
            {/if}
            {#if result.defaults.length > 0}
                <p class="muted small">Assumed {result.defaults.map((d) => `${d.name} = ${d.shown}`).join(', ')}.</p>
            {/if}
        </section>
    {/if}
</div>

<style>
    .view {
        flex: 1;
        min-height: 0;
        overflow-y: auto;
        padding: var(--space-5) var(--space-5) var(--space-6);
        display: flex;
        flex-direction: column;
        gap: var(--space-3);
        max-width: 980px;
    }
    header {
        padding: 0 var(--space-2);
        display: flex;
        flex-direction: column;
        gap: var(--space-1);
    }
    .panel {
        padding: var(--space-5);
        border-radius: var(--r-xl);
        display: flex;
        flex-direction: column;
        gap: var(--space-4);
    }
    .row {
        display: flex;
        gap: var(--space-3);
        align-items: flex-end;
    }
    .field {
        display: flex;
        flex-direction: column;
        gap: 6px;
    }
    .grow {
        flex: 1;
    }
    .steps {
        margin: 0;
        padding-left: var(--space-5);
        display: flex;
        flex-direction: column;
        gap: var(--space-2);
    }
    .steps li {
        display: flex;
        justify-content: space-between;
        gap: var(--space-4);
    }
    .answer {
        display: flex;
        align-items: baseline;
        gap: var(--space-3);
        padding: var(--space-4) var(--space-5);
        border-radius: var(--r-lg);
        background: var(--answer-bg);
        border: 1px solid var(--answer-edge);
    }
    .a-label {
        font-weight: 600;
        color: var(--answer-label);
    }
    .a-value {
        font-family: var(--font-display);
        font-weight: 600;
        font-size: var(--text-answer);
        color: var(--answer-text);
    }
</style>
