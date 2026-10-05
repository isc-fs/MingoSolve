<!--
    Chain: list what you know, name what you want; the engine applies formulas until it gets there and shows only
    the steps the answer depends on. A red note means two formulas disagree: read it before using the number.
    Past questions that were solved this way load in one click. The form lives in session.chain.
-->
<script lang="ts">
    import { onMount, tick, untrack } from 'svelte';

    import Tex from './Tex.svelte';
    import { chainFormulas } from './solve';
    import { catalog, searchVariables, variableInfo, type VarEntry } from './catalog.svelte';
    import { openCommand, session } from './session.svelte';
    import { friendlyError } from './errors';
    import { chainExamples } from './topics';
    import type { ChainResult, WorkedExample } from './types';

    let result = $state<ChainResult | null>(null);
    /** The target the shown result was computed for (the input may have been edited since). */
    let resultTarget = $state('');
    let error = $state<string | null>(null);
    let examples = $state<WorkedExample[]>([]);
    let open = $state(false);
    let index = $state(0);
    let list = $state<HTMLElement | null>(null);
    let ranFor = 0;
    let latest = 0;

    const options = $derived<VarEntry[]>(open && catalog.ready ? searchVariables(session.chain.target) : []);
    const picked = $derived(catalog.ready ? variableInfo(session.chain.target.trim()) : undefined);
    const resultInfo = $derived(catalog.ready ? variableInfo(resultTarget) : undefined);

    onMount(() => {
        void chainExamples().then((ex) => (examples = ex));
    });

    // a past question (openCommand) fills the form and asks to run it
    $effect(() => {
        const r = session.chain.run;
        if (r > 0 && r !== ranFor) {
            ranFor = r;
            untrack(() => void run());
        }
    });

    $effect(() => {
        const i = index;
        void tick().then(() => list?.querySelector(`#chain-opt-${i}`)?.scrollIntoView?.({ block: 'nearest' }));
    });

    const descOf = (name: string): string | undefined =>
        [...catalog.scripts.values()].flatMap((s) => s.formula?.vars ?? []).find((v) => v.name === name)?.desc;

    function parseGiven(): [string, string][] {
        return session.chain.given
            .split(/\n|;/)
            .map((l) => l.trim())
            .filter((l) => l.includes('='))
            .map((l) => {
                const i = l.indexOf('=');
                return [l.slice(0, i).trim(), l.slice(i + 1).trim()] as [string, string];
            });
    }

    async function run(): Promise<void> {
        const c = session.chain;
        const target = c.target.trim();
        if (target.length === 0) return;
        latest += 1;
        const mine = latest;
        try {
            const tags = c.only.split(/[,\s]+/).filter((t) => t.length > 0);
            const r = await chainFormulas(target, parseGiven(), tags, { [target]: c.shownIn });
            if (mine !== latest) return;
            result = r;
            resultTarget = target;
            error = null;
        } catch (e) {
            if (mine !== latest) return;
            result = null;
            error = friendlyError(e, {
                fields: [
                    ...parseGiven().map(([name, value]) => ({ name, label: descOf(name) ?? name, value })),
                    { name: '@show', label: 'Show in', value: c.shownIn },
                ],
                nameOf: descOf,
            });
        }
    }

    /** Pick an option; true when the user had already typed exactly that symbol (Enter then runs). */
    function choose(i: number): boolean {
        const o = options[i];
        if (o === undefined) return false;
        const typed = session.chain.target.trim();
        session.chain.target = o.name;
        open = false;
        return typed === o.name;
    }

    function key(e: KeyboardEvent): void {
        if (e.key === 'Escape' && open) {
            open = false;
            e.preventDefault();
        } else if (e.key === 'ArrowDown') {
            if (!open) open = true;
            else index = Math.min(index + 1, Math.max(options.length - 1, 0));
            e.preventDefault();
        } else if (e.key === 'ArrowUp') {
            index = Math.max(index - 1, 0);
            e.preventDefault();
        } else if (e.key === 'Enter') {
            e.preventDefault();
            if (options.length > 0) {
                if (choose(index)) void run();
            } else void run();
        }
    }
</script>

<div class="view">
    <header>
        <h1>Chain</h1>
        <p class="muted">
            For problems that need several formulas. Say what you want and what you know; the engine finds the path.
        </p>
    </header>

    {#if examples.length > 0}
        <section class="glass panel past" aria-label="Past questions">
            <span class="label">Past questions</span>
            <div class="chips">
                {#each examples as ex (ex.id)}
                    <button type="button" class="chip chip-quiet" title={ex.what || ex.cmd} onclick={() => openCommand(ex.cmd, ex.answer)}>
                        Q{ex.id}{ex.what ? ` · ${ex.what.length > 40 ? ex.what.slice(0, 40) + '…' : ex.what}` : ''}
                    </button>
                {/each}
            </div>
        </section>
    {/if}

    <section class="glass panel">
        <div class="row">
            <div class="field grow target">
                <label class="label" for="chain-target">Target</label>
                <input id="chain-target" autocomplete="off" autocorrect="off" autocapitalize="off" spellcheck="false" class="input mono"
                    bind:value={session.chain.target}
                    oninput={() => { open = true; index = 0; }}
                    onblur={() => (open = false)}
                    onkeydown={key}
                    placeholder="v, or describe it: rollover speed"
                    role="combobox"
                    aria-expanded={options.length > 0}
                    aria-controls="chain-list"
                    aria-autocomplete="list"
                    aria-activedescendant={options.length > 0 ? `chain-opt-${index}` : undefined}
                />
                <ul class="options glass" role="listbox" id="chain-list" aria-label="Variables" bind:this={list} hidden={options.length === 0}>
                    {#each options as o, i (o.name)}
                        <!-- keyboard selection lives on the combobox (aria-activedescendant); the click is for pointers -->
                        <!-- svelte-ignore a11y_click_events_have_key_events -->
                        <li role="option" id="chain-opt-{i}" aria-selected={i === index} class="opt" class:on={i === index}
                            onmousedown={(e) => e.preventDefault()}
                            onmouseenter={() => (index = i)}
                            onclick={() => choose(i)}>
                            <Tex tex={o.tex} /><span class="desc">— {o.desc}{o.unit ? ` [${o.unit}]` : ''}</span>
                        </li>
                    {/each}
                </ul>
                {#if picked !== undefined}
                    <span class="muted small hint"><Tex tex={picked.tex} /> — {picked.desc}{picked.unit ? ` [${picked.unit}]` : ''}</span>
                {/if}
            </div>
            <label class="field">
                <span class="label">Show in</span>
                <input autocomplete="off" autocorrect="off" autocapitalize="off" spellcheck="false" class="input mono" bind:value={session.chain.shownIn} placeholder="km/h" />
            </label>
            <label class="field grow">
                <span class="label">Only formulas tagged</span>
                <input autocomplete="off" autocorrect="off" autocapitalize="off" spellcheck="false" class="input mono" bind:value={session.chain.only} placeholder="all (or: dynamics, aero)" />
            </label>
        </div>
        <label class="field">
            <span class="label">Known values, one per line</span>
            <textarea autocomplete="off" autocapitalize="off" spellcheck="false" class="input mono" rows="5" bind:value={session.chain.given} placeholder={'h_cg = 0.205 m\nR_c = 14.5 m\nt_tr = 1.24 m'}></textarea>
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
                <div class="answer"><span class="a-label"><Tex tex={resultInfo?.tex ?? resultTarget} /> =</span><span class="a-value">{result.target}</span></div>
            {:else}
                <p class="note note-warn"><strong>No path to {resultTarget}.</strong> Known: <span class="mono">{result.known.join(', ')}</span></p>
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
    .past {
        gap: var(--space-2);
        padding: var(--space-4) var(--space-5);
    }
    .chips {
        display: flex;
        flex-wrap: wrap;
        gap: 6px;
    }
    .chips .chip {
        border: none;
        cursor: pointer;
        font-family: var(--font-sans);
    }
    .row {
        display: flex;
        gap: var(--space-3);
        align-items: flex-start;
    }
    .field {
        display: flex;
        flex-direction: column;
        gap: 6px;
    }
    .grow {
        flex: 1;
    }
    .target {
        position: relative;
    }
    .options {
        position: absolute;
        z-index: 5;
        top: 62px;
        left: 0;
        right: 0;
        margin: 0;
        padding: var(--space-1);
        list-style: none;
        max-height: 280px;
        overflow-y: auto;
        border-radius: var(--r-lg);
        background: var(--glass-strong);
    }
    .options[hidden] {
        display: none;
    }
    .opt {
        padding: 8px var(--space-3);
        border-radius: var(--r-md);
        cursor: pointer;
    }
    .opt.on {
        background: var(--selected);
    }
    .desc {
        margin-left: 0.4em;
    }
    .hint {
        min-height: 1.2em;
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
