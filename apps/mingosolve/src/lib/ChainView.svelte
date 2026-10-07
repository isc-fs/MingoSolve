<!--
    Chain: list what you know, name what you want; the engine applies formulas until it gets there and shows only
    the steps the answer depends on. A red note means two formulas disagree: read it before using the number.
    Past questions that were solved this way load in one click (in a fold, closed until opened). Names are read the
    way people write them (v_i as v0): a quiet note says what was read as what, and an unknown or ambiguous name
    offers its likely meanings as buttons. The form lives in session.chain.
-->
<script lang="ts">
    import { onMount, tick, untrack } from 'svelte';

    import Disclosure from './Disclosure.svelte';
    import Tex from './Tex.svelte';
    import { chainFormulas } from './solve';
    import { catalog, searchVariables, variableInfo, type VarEntry } from './catalog.svelte';
    import { openCommand, session } from './session.svelte';
    import { friendlyError } from './errors';
    import { settings } from './settings.svelte';
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

    /** Read out when a chain finishes (polite: the visible result is not itself a live region). */
    const chainSays = $derived.by(() => {
        if (result === null) return '';
        if (result.problems.length > 0) return `${result.problems.length} names need checking.`;
        const conflict = result.conflicts.length > 0 ? ' The formulas disagree: check the red note.' : '';
        return result.reached ? `${resultTarget} = ${result.target}.${conflict}` : `No path to ${resultTarget}.${conflict}`;
    });

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

    /** The names the form uses now, so notes about an earlier run do not outlive an edit. */
    const typedNames = $derived(new Set([session.chain.target.trim(), ...parseGiven().map(([n]) => n)]));
    const reads = $derived((result?.mapped ?? []).filter((m) => typedNames.has(m.from)));
    const problems = $derived((result?.problems ?? []).filter((p) => typedNames.has(p.name)));

    /** Replace a typed name by the one the person picked and chain again. */
    function useName(from: string, to: string): void {
        const c = session.chain;
        if (c.target.trim() === from) c.target = to;
        c.given = c.given
            .split(/(\n|;)/)
            .map((seg) => {
                const i = seg.indexOf('=');
                return i >= 0 && seg.slice(0, i).trim() === from ? seg.slice(0, i).replace(from, to) + seg.slice(i) : seg;
            })
            .join('');
        void run();
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
            <Disclosure label="Past questions" id="chain-past" bind:open={settings.chainPastOpen}>
                <div class="chips">
                    {#each examples as ex (ex.id)}
                        <button type="button" class="chip chip-quiet" title={ex.what || ex.cmd} onclick={() => openCommand(ex.cmd, ex.answer)}>
                            Q{ex.id}{ex.what ? ` · ${ex.what.length > 40 ? ex.what.slice(0, 40) + '…' : ex.what}` : ''}
                        </button>
                    {/each}
                </div>
            </Disclosure>
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
        {#if reads.length > 0}
            <ul class="reads" aria-label="Names read as">
                {#each reads as m (m.from)}
                    <li class="muted small"><span class="mono">{m.from}</span> → <span class="mono">{m.to}</span> ({m.desc})</li>
                {/each}
            </ul>
        {/if}
        {#each problems as p (p.name)}
            <p class="note note-warn">
                {#if p.kind === 'unknown'}Unknown name “{p.name}”.{:else}“{p.name}” could mean several variables.{/if}
                {#if p.choices.length > 0}
                    Did you mean
                    {#each p.choices as c, i (c.name)}{#if i > 0}{i === p.choices.length - 1 ? ' or ' : ', '}{/if}<button type="button" class="pick mono" onclick={() => useName(p.name, c.name)}>{c.name} ({c.desc})</button>{/each}?
                {/if}
            </p>
        {/each}
        <div><button type="button" class="btn btn-primary" onclick={run}>Chain</button></div>
    </section>

    <p class="sr-only" role="status" aria-live="polite">{chainSays}</p>
    {#if error !== null}<p class="note note-bad" role="alert"><strong>Can't chain that.</strong> {error}</p>{/if}

    {#if result !== null && problems.length === 0}
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
        padding: var(--space-5) max(var(--space-5), calc((100% - 980px) / 2)) var(--space-6);
        display: flex;
        flex-direction: column;
        gap: var(--space-3);
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
    .reads {
        margin: 0;
        padding: 0;
        list-style: none;
        display: flex;
        flex-direction: column;
        gap: 2px;
    }
    .pick {
        appearance: none;
        border: none;
        background: var(--accent-soft);
        color: var(--ink-accent);
        border-radius: var(--r-pill);
        padding: 1px 10px;
        margin: 0 2px;
        font-size: var(--text-xs);
        cursor: pointer;
    }
    .pick:hover {
        filter: brightness(1.1);
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
        border-radius: var(--slab-radius);
        background: var(--slab-ornament), var(--answer-bg);
        box-shadow: var(--slab-ring);
        border: var(--slab-border);
    }
    .a-label {
        font-weight: 600;
        color: var(--answer-label);
        background: var(--slab-cell);
    }
    .a-value {
        font-family: var(--answer-font);
        font-weight: var(--answer-weight);
        font-size: var(--text-answer);
        letter-spacing: var(--answer-tracking);
        font-variant-numeric: var(--answer-numeric);
        color: var(--value-text);
        background: var(--value-bg);
    }
</style>
