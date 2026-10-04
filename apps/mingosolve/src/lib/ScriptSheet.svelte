<!--
    The open script. Formula scripts: one field per variable (value with units, blank = unknown), solved live; every
    root is listed and the picked one fills the answer slab, which copies it in quiz format and can be checked
    against pasted multiple-choice options. Tool scripts: parameters with their defaults, Enter runs.
    Worked examples (past FS-Quiz questions that use this script) load their inputs in one click.
-->
<script lang="ts">
    import Icon from './Icon.svelte';
    import { catalog } from './catalog.svelte';
    import { solveFormula } from './solve';
    import { runTool } from './tools';
    import { formatAnswer, matchOptions } from './finder';
    import { scriptExamples } from './topics';
    import { openCommand, session, togglePin } from './session.svelte';
    import { settings } from './settings.svelte';
    import type { Matching, SolveResult, WorkedExample } from './types';

    const script = $derived(session.script !== null ? catalog.scripts.get(session.script.id) ?? null : null);

    let values = $state<Record<string, string>>({});
    let display = $state<Record<string, string>>({});
    let result = $state<SolveResult | null>(null);
    let toolOut = $state<string | null>(null);
    let error = $state<string | null>(null);
    let picked = $state<{ name: string; index: number } | null>(null);
    let examples = $state<WorkedExample[]>([]);
    let options = $state('');
    let showOptions = $state(false);
    let matching = $state<Matching | null>(null);
    let formatted = $state('');
    let copied = $state(false);

    // Reset and pre-fill whenever a script is opened (nonce changes even for the same script).
    $effect(() => {
        const open = session.script;
        const s = script;
        if (open === null || s === null) return;
        void open.nonce;
        const given = Object.fromEntries(open.values);
        if (s.formula !== undefined) {
            values = Object.fromEntries(s.formula.vars.map((v) => [v.name, given[v.name] ?? '']));
            display = Object.fromEntries(s.formula.vars.map((v) => [v.name, open.display?.[v.name] ?? '']));
        } else if (s.tool !== undefined) {
            const vals: Record<string, string> = Object.fromEntries(s.tool.params.map((p) => [p.name, '']));
            if ('rules' in vals) vals.rules = settings.rules;
            (open.positional ?? []).forEach((v, i) => {
                const p = s.tool!.params[i];
                if (p !== undefined) vals[p.name] = v;
            });
            for (const [k, v] of open.values) vals[k] = v;
            values = vals;
            display = {};
        }
        result = null;
        toolOut = null;
        error = null;
        picked = null;
        options = '';
        matching = null;
        showOptions = false;
        void scriptExamples(s.id).then((ex) => (examples = ex));
        if (s.tool !== undefined && (open.positional?.length ?? 0) + open.values.length > 0) void run();
    });

    // Formula scripts solve live.
    let timer: ReturnType<typeof setTimeout> | null = null;
    $effect(() => {
        const s = script;
        if (s === null || s.formula === undefined) return;
        const given: [string, string][] = Object.entries(values).filter(([, v]) => v.trim().length > 0);
        const disp = { ...display };
        if (timer !== null) clearTimeout(timer);
        timer = setTimeout(async () => {
            try {
                result = await solveFormula(s.id, given, disp);
                error = null;
                if (picked === null || !result.found.some((f) => f.name === picked!.name)) {
                    picked = result.found.length > 0 ? { name: result.found[0].name, index: 0 } : null;
                }
            } catch (e) {
                result = null;
                error = String(e);
            }
        }, 110);
    });

    async function run(): Promise<void> {
        const s = script;
        if (s === null || s.tool === undefined) return;
        try {
            toolOut = await runTool(s.id, Object.entries(values));
            error = null;
        } catch (e) {
            toolOut = null;
            error = String(e);
        }
    }

    const answer = $derived.by(() => {
        if (script?.tool !== undefined) {
            if (toolOut === null) return null;
            const n = parseFloat(toolOut);
            return { label: script.id, shown: toolOut, n: Number.isFinite(n) && /^-?[\d.]+(e[-+]?\d+)?$/.test(toolOut.trim()) ? n : null, others: [] as string[] };
        }
        if (result === null || picked === null) return null;
        const fv = result.found.find((f) => f.name === picked!.name);
        const shown = fv?.shown[picked.index];
        if (fv === undefined || shown === undefined) return null;
        const n = parseFloat(shown);
        return { label: fv.name, shown, n: Number.isFinite(n) ? n : null, others: fv.shown.filter((_, i) => i !== picked!.index) };
    });

    $effect(() => {
        const a = answer;
        const prec = settings.precision.kind === 'sig' ? { sig: settings.precision.n } : { decimals: settings.precision.n };
        const comma = settings.decimalComma;
        const opts = options;
        copied = false;
        if (a === null || a.n === null) {
            formatted = a?.shown ?? '';
            matching = null;
            return;
        }
        void formatAnswer(a.n, prec, comma).then((f) => (formatted = f));
        matching = null;
        if (opts.trim().length > 0) void matchOptions(a.n, opts).then((m) => (matching = m));
    });

    function copy(): void {
        if (formatted.length === 0) return;
        void navigator.clipboard.writeText(formatted);
        copied = true;
    }

    function clear(): void {
        for (const k of Object.keys(values)) values[k] = k === 'rules' ? settings.rules : '';
    }

    const pinned = $derived(script !== null && settings.pinned.includes(script.id));
</script>

{#if script === null}
    <div class="empty glass">
        <Icon name="spark" size={28} />
        <h2>Paste a problem, or pick a script</h2>
        <p class="muted">
            Paste the question text above and the matching scripts open with its values filled in. Or press
            <kbd>⌘K</kbd> and type what you need: <span class="mono">spring rate</span>,
            <span class="mono">discharge</span>, <span class="mono">skidpad score</span>.
        </p>
    </div>
{:else}
    <section class="sheet glass" onkeydown={(e) => e.key === 'Escape' && clear()} role="presentation">
        <header>
            <div class="titles">
                <span class="label">{script.topic?.name ?? 'Script'} · {script.kind === 'formula' ? 'solve for the blank' : 'tool'}</span>
                <h2>{script.title}</h2>
            </div>
            <button type="button" class="btn btn-ghost btn-sm" class:pinned onclick={() => togglePin(script.id)} aria-pressed={pinned} title={pinned ? 'Unpin from the rail' : 'Pin to the rail'}>
                <Icon name="pin" size={15} />{pinned ? 'Pinned' : 'Pin'}
            </button>
            <button type="button" class="btn btn-ghost btn-sm" onclick={clear} title="Clear all fields (Esc)">Clear</button>
        </header>

        {#if script.formula !== undefined}
            <pre class="eqs">{script.formula.eqs.join('\n')}</pre>
            {#if script.formula.notes.length > 0}
                <p class="note">{script.formula.notes}</p>
            {/if}
            <div class="fields">
                {#each script.formula.vars as v (v.name)}
                    {@const solved = result?.found.find((f) => f.name === v.name)}
                    <label class="field">
                        <span class="flabel"><span class="var">{v.name}</span> {v.desc}</span>
                        <input autocomplete="off" autocorrect="off" autocapitalize="off" spellcheck="false"
                            class="input mono"
                            class:filled={values[v.name]?.trim().length > 0}
                            class:solved={solved !== undefined}
                            bind:value={values[v.name]}
                            placeholder={solved !== undefined ? `= ${solved.shown[0]}` : v.default !== null ? `default ${v.default}` : v.unit === 'dimensionless' ? 'unknown' : `unknown [${v.unit}]`}
                        />
                    </label>
                {/each}
            </div>
        {:else if script.tool !== undefined}
            <p class="doc">{script.tool.doc}</p>
            <form
                class="fields"
                onsubmit={(e) => {
                    e.preventDefault();
                    void run();
                }}
            >
                {#each script.tool.params as p (p.name)}
                    <label class="field" class:wide={['netlist', 'levels', 'teeth', 'events'].includes(p.name)}>
                        <span class="flabel"><span class="var">{p.name}</span></span>
                        <input autocomplete="off" autocorrect="off" autocapitalize="off" spellcheck="false"
                            class="input mono"
                            bind:value={values[p.name]}
                            placeholder={p.default === null ? 'required' : p.default === '' ? 'optional' : `default ${p.default}`}
                        />
                    </label>
                {/each}
                <div class="run"><button type="submit" class="btn btn-primary">Run <kbd>↵</kbd></button></div>
            </form>
        {/if}

        {#if error !== null}
            <p class="note note-bad"><strong>Can't solve that.</strong> {error}</p>
        {/if}
        {#if result !== null && result.conflicts.length > 0}
            <div class="note note-bad">
                <strong>These values contradict each other.</strong>
                <ul class="mono small">{#each result.conflicts as c (c)}<li>{c}</li>{/each}</ul>
            </div>
        {/if}

        {#if answer !== null}
            <div class="answer">
                <div class="answer-row">
                    <span class="a-label">{answer.label} =</span>
                    <span class="a-value">{answer.shown}</span>
                    <button type="button" class="btn copy" onclick={copy} title="Copy as {formatted}">
                        <Icon name={copied ? 'check' : 'copy'} size={15} />{copied ? 'Copied' : `Copy ${formatted}`}
                    </button>
                </div>
                {#if answer.others.length > 0}
                    <p class="a-note">Other root{answer.others.length > 1 ? 's' : ''}: {answer.others.join(' · ')}. Pick one below if the physics says so.</p>
                {/if}
                {#if script.formula !== undefined && picked !== null}
                    <label class="a-unit">
                        <span>Show in</span>
                        <input autocomplete="off" autocorrect="off" autocapitalize="off" spellcheck="false" class="input mono" bind:value={display[picked.name]} placeholder="km/h, kN, mm…" />
                    </label>
                {/if}
            </div>
        {/if}

        {#if result !== null && result.found.length > 0}
            <div class="results">
                {#each result.found as fv (fv.name)}
                    {#each fv.shown as s, i (i)}
                        <button
                            type="button"
                            class="res"
                            class:on={picked?.name === fv.name && picked?.index === i}
                            onclick={() => (picked = { name: fv.name, index: i })}
                            title={fv.desc}
                        >
                            <span class="var">{fv.name}</span><span class="mono">{s}</span>
                        </button>
                    {/each}
                {/each}
            </div>
            {#if result.defaults.length > 0}
                <p class="muted small">Assumed {result.defaults.map((d) => `${d.name} = ${d.shown}`).join(', ')}.</p>
            {/if}
        {/if}

        {#if answer !== null && answer.n !== null}
            <div class="check">
                <button type="button" class="btn btn-ghost btn-sm" onclick={() => (showOptions = !showOptions)} aria-expanded={showOptions}>
                    <Icon name="chevron" size={14} />Check against options
                </button>
                {#if showOptions}
                    <textarea autocomplete="off" autocapitalize="off" spellcheck="false" class="input mono" rows="3" bind:value={options} placeholder={'a) 73.4 A\nb) 77.9 A\nc) 80.4 A'}></textarea>
                    {#if matching !== null}
                        {#if matching.warning !== null}<p class="note note-warn"><strong>Careful:</strong> {matching.warning}</p>{/if}
                        <ul class="opts">
                            {#each matching.options as o, i (i)}
                                <li class:best={o.best}>
                                    <span class="mono">{o.text}</span>
                                    {#if o.rel_diff !== null}<span class="small">{(o.rel_diff * 100).toFixed(2)} %</span>{/if}
                                </li>
                            {/each}
                        </ul>
                    {/if}
                {/if}
            </div>
        {/if}

        {#if examples.length > 0}
            <footer>
                <span class="label">Worked examples</span>
                {#each examples as ex (ex.id)}
                    <button type="button" class="chip chip-quiet" title={ex.what} onclick={() => openCommand(ex.cmd)}>
                        Q{ex.id} · {ex.what.length > 34 ? ex.what.slice(0, 34) + '…' : ex.what}
                    </button>
                {/each}
            </footer>
        {/if}
    </section>
{/if}

<style>
    .empty {
        padding: var(--space-8);
        border-radius: var(--r-xl);
        display: flex;
        flex-direction: column;
        gap: var(--space-3);
        align-items: flex-start;
        color: var(--ink-accent);
    }
    .empty h2 {
        color: var(--text);
    }
    .empty p {
        max-width: 52ch;
        line-height: 1.6;
    }
    .sheet {
        padding: var(--space-5);
        border-radius: var(--r-xl);
        display: flex;
        flex-direction: column;
        gap: var(--space-4);
    }
    header {
        display: flex;
        align-items: flex-start;
        gap: var(--space-2);
    }
    .titles {
        flex: 1;
        display: flex;
        flex-direction: column;
        gap: var(--space-1);
    }
    .pinned {
        color: var(--ink-accent);
    }
    .eqs {
        margin: 0;
        padding: var(--space-3) var(--space-4);
        border-radius: var(--r-md);
        background: var(--code);
        font-size: var(--text-sm);
        line-height: 1.7;
        color: var(--text-2);
        white-space: pre-wrap;
    }
    .doc {
        color: var(--text-2);
        line-height: 1.55;
    }
    .fields {
        display: grid;
        grid-template-columns: repeat(auto-fill, minmax(210px, 1fr));
        gap: var(--space-3);
    }
    .field {
        display: flex;
        flex-direction: column;
        gap: 6px;
        min-width: 0;
    }
    .field.wide,
    .run {
        grid-column: 1 / -1;
    }
    .flabel {
        font-size: var(--text-sm);
        color: var(--text-2);
        overflow: hidden;
        text-overflow: ellipsis;
        white-space: nowrap;
    }
    .input.filled {
        border-color: var(--accent-edge);
    }
    .input.solved::placeholder {
        color: var(--ink-accent);
        opacity: 1;
    }
    .answer {
        padding: var(--space-4) var(--space-5);
        border-radius: var(--r-lg);
        background: var(--answer-bg);
        border: 1px solid var(--answer-edge);
        display: flex;
        flex-direction: column;
        gap: var(--space-2);
    }
    .answer-row {
        display: flex;
        align-items: baseline;
        gap: var(--space-3);
        flex-wrap: wrap;
    }
    .a-label {
        font-family: var(--font-mono);
        font-weight: 600;
        color: var(--answer-label);
    }
    .a-value {
        font-family: var(--font-display);
        font-weight: 600;
        font-size: var(--text-answer);
        letter-spacing: 0.01em;
        color: var(--answer-text);
        overflow-wrap: anywhere;
    }
    .copy {
        margin-left: auto;
        align-self: center;
        background: var(--isc-gold);
        color: #1a1406;
        border-color: transparent;
        font-weight: 600;
    }
    .copy:hover {
        border-color: transparent;
        filter: brightness(1.05);
    }
    .a-note {
        color: var(--answer-note);
        font-size: var(--text-sm);
    }
    .a-unit {
        display: flex;
        align-items: center;
        gap: var(--space-2);
        color: var(--answer-note);
        font-size: var(--text-sm);
    }
    .a-unit .input {
        width: 160px;
        min-height: 30px;
        padding: 4px 10px;
    }
    .results {
        display: flex;
        flex-wrap: wrap;
        gap: var(--space-2);
    }
    .res {
        appearance: none;
        display: inline-flex;
        gap: var(--space-2);
        align-items: baseline;
        padding: 6px var(--space-3);
        border-radius: var(--r-pill);
        border: 1px solid var(--well-edge);
        background: var(--well);
        cursor: pointer;
        font-size: var(--text-sm);
    }
    .res:hover {
        border-color: var(--accent-edge);
    }
    .res.on {
        border-color: var(--accent);
        box-shadow: 0 0 0 3px var(--accent-soft);
    }
    .check {
        display: flex;
        flex-direction: column;
        gap: var(--space-2);
        align-items: flex-start;
    }
    .check .input {
        max-width: 420px;
    }
    .check :global(button[aria-expanded='true'] svg) {
        transform: rotate(90deg);
    }
    .opts {
        list-style: none;
        margin: 0;
        padding: 0;
        display: flex;
        flex-direction: column;
        gap: 4px;
        min-width: 300px;
    }
    .opts li {
        display: flex;
        justify-content: space-between;
        gap: var(--space-4);
        padding: 6px var(--space-3);
        border-radius: var(--r-sm);
        background: var(--well);
        color: var(--muted);
    }
    .opts li span:first-child {
        color: var(--text-2);
    }
    .opts li.best {
        background: var(--good-soft);
        outline: 1px solid var(--good);
    }
    .opts li.best span {
        color: var(--good);
        font-weight: 600;
    }
    footer {
        display: flex;
        flex-wrap: wrap;
        align-items: center;
        gap: var(--space-2);
        padding-top: var(--space-3);
        border-top: 1px solid var(--glass-edge);
    }
    footer .chip {
        border: none;
        cursor: pointer;
        font-family: var(--font-sans);
    }
    footer .chip:hover {
        color: var(--text);
    }
</style>
