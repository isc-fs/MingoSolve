<!--
    Solve: the quiz hub. Paste the question to rank formulas, tools and past questions (a formula hit opens with
    the question's values already filled in); or search a formula directly (Cmd/Ctrl+K). The form solves live for
    whatever is blank; pick a result to match it against the multiple-choice options and copy it in quiz format.
-->
<script lang="ts">
    import { onMount, tick } from 'svelte';

    import { listFormulas, solveFormula } from './solve';
    import { findQuestion, formatAnswer, matchOptions } from './finder';
    import { logAnswer, open, session } from './session.svelte';
    import { settings } from './settings.svelte';
    import type { FormulaInfo, Hit, Matching, SolveResult } from './types';

    let formulas = $state<FormulaInfo[]>([]);
    let question = $state('');
    let hits = $state<Hit[]>([]);
    let search = $state('');
    let searchEl = $state<HTMLInputElement | null>(null);
    let current = $state<FormulaInfo | null>(null);
    let values = $state<Record<string, string>>({});
    let display = $state<Record<string, string>>({});
    let result = $state<SolveResult | null>(null);
    let error = $state<string | null>(null);
    let picked = $state<{ name: string; index: number } | null>(null);
    let options = $state('');
    let matching = $state<Matching | null>(null);
    let formatted = $state('');

    const filtered = $derived.by(() => {
        const words = search.toLowerCase().split(/\s+/).filter((w) => w.length > 0);
        if (words.length === 0) return [];
        return formulas
            .filter((f) => {
                const hay = `${f.key} ${f.title} ${f.tags.join(' ')} ${f.vars.map((v) => v.desc).join(' ')}`.toLowerCase();
                return words.every((w) => hay.includes(w));
            })
            .slice(0, 12);
    });

    onMount(async () => {
        formulas = await listFormulas();
        applyRequest();
    });

    // Cmd/Ctrl+K from anywhere lands here with the search focused.
    $effect(() => {
        if (session.focusSearch > 0) {
            void tick().then(() => searchEl?.focus());
        }
    });

    // A request from a finder hit, a past question or another view.
    $effect(() => {
        if (session.request !== null && session.request.view === 'solve' && formulas.length > 0) {
            applyRequest();
        }
    });

    function applyRequest(): void {
        const req = session.request;
        if (req === null || req.view !== 'solve') return;
        session.request = null;
        const f = formulas.find((x) => x.key === req.id);
        if (f === undefined) return;
        choose(f, Object.fromEntries(req.values), req.display ?? {});
    }

    function choose(f: FormulaInfo, vals: Record<string, string> = {}, disp: Record<string, string> = {}): void {
        current = f;
        values = Object.fromEntries(f.vars.map((v) => [v.name, vals[v.name] ?? '']));
        display = Object.fromEntries(f.vars.map((v) => [v.name, disp[v.name] ?? '']));
        search = '';
        picked = null;
        matching = null;
    }

    function clearForm(): void {
        if (current !== null) choose(current);
    }

    // Live solve, debounced.
    let timer: ReturnType<typeof setTimeout> | null = null;
    $effect(() => {
        const f = current;
        const given: [string, string][] = Object.entries(values).filter(([, v]) => v.trim().length > 0);
        const disp = { ...display };
        if (timer !== null) clearTimeout(timer);
        if (f === null) return;
        timer = setTimeout(async () => {
            try {
                result = await solveFormula(f.key, given, disp);
                error = null;
            } catch (e) {
                result = null;
                error = String(e);
            }
        }, 120);
    });

    // Find hits for the pasted question, debounced.
    let qTimer: ReturnType<typeof setTimeout> | null = null;
    $effect(() => {
        const q = question;
        if (qTimer !== null) clearTimeout(qTimer);
        qTimer = setTimeout(async () => {
            hits = q.trim().length > 0 ? await findQuestion(q) : [];
        }, 200);
    });

    function openHit(h: Hit): void {
        if (h.kind === 'formula') {
            open({ view: 'solve', id: h.id, values: h.prefill });
        } else if (h.kind === 'tool') {
            open({ view: 'tools', id: h.id, values: [] });
        } else {
            session.activeView = 'examples';
            session.request = { view: 'examples', id: h.id, values: [] };
        }
    }

    const pickedValue = $derived.by(() => {
        if (picked === null || result === null) return null;
        const fv = result.found.find((f) => f.name === picked!.name);
        const shown = fv?.shown[picked.index];
        if (shown === undefined) return null;
        const n = parseFloat(shown.split(' ')[0]);
        return Number.isFinite(n) ? { n, shown } : null;
    });

    $effect(() => {
        const p = pickedValue;
        const prec = settings.precision.kind === 'sig' ? { sig: settings.precision.n } : { decimals: settings.precision.n };
        const comma = settings.decimalComma;
        const opts = options;
        if (p === null) {
            formatted = '';
            matching = null;
            return;
        }
        void formatAnswer(p.n, prec, comma).then((s) => (formatted = s));
        if (opts.trim().length > 0) {
            void matchOptions(p.n, opts).then((m) => (matching = m));
        } else {
            matching = null;
        }
    });

    function copyAnswer(): void {
        if (formatted.length === 0 || current === null || pickedValue === null) return;
        void navigator.clipboard.writeText(formatted);
        const inputs = Object.entries(values)
            .filter(([, v]) => v.trim().length > 0)
            .map(([k, v]) => `${k}=${v}`)
            .join(' ');
        logAnswer(`${current.key}: ${picked?.name}`, inputs, `${formatted} (${pickedValue.shown})`);
    }

    function keydown(e: KeyboardEvent): void {
        if (e.key === 'Escape') clearForm();
    }
</script>

<section class="view" onkeydown={keydown} role="presentation">
    <header>
        <div>
            <h2>Solve</h2>
            <p>
                Paste the quiz question to find the formula, or search one directly
                (<span class="mono">⌘K</span> / <span class="mono">Ctrl K</span>). Leave blank whatever you want
                solved; values take units (<span class="mono">100 km/h</span>, <span class="mono">8000 rpm</span>,
                <span class="mono">1.5 g0</span>).
            </p>
        </div>
    </header>

    <div class="cols">
        <div class="stack left">
            <div class="card card-tight stack stack-sm">
                <label class="field">
                    <span>Question</span>
                    <textarea rows="5" placeholder="Paste the question text here" bind:value={question}></textarea>
                </label>
                {#if hits.length > 0}
                    <ul class="hits">
                        {#each hits as h (h.kind + h.id)}
                            <li>
                                <button type="button" class="hit" onclick={() => openHit(h)}>
                                    <span class="row">
                                        <span class="pill" class:pill-accent={h.kind === 'formula'} class:pill-info={h.kind === 'tool'}>
                                            {h.kind}
                                        </span>
                                        <span class="mono">{h.id}</span>
                                        {#if h.prefill.length > 0}
                                            <span class="muted small">fills {h.prefill.length}</span>
                                        {/if}
                                        {#if h.warning !== null}
                                            <span class="pill pill-warning">key off</span>
                                        {/if}
                                    </span>
                                    <span class="muted small title">{h.title}</span>
                                </button>
                            </li>
                        {/each}
                    </ul>
                {/if}
            </div>

            <div class="card card-tight stack stack-sm">
                <label class="field">
                    <span>Formula</span>
                    <input
                        type="text"
                        placeholder="search: spring, discharge, skidpad…"
                        bind:this={searchEl}
                        bind:value={search}
                        onkeydown={(e) => {
                            if (e.key === 'Enter' && filtered.length > 0) choose(filtered[0]);
                        }}
                    />
                </label>
                {#if filtered.length > 0}
                    <ul class="hits">
                        {#each filtered as f (f.key)}
                            <li>
                                <button type="button" class="hit" onclick={() => choose(f)}>
                                    <span class="mono">{f.key}</span>
                                    <span class="muted small title">{f.title}</span>
                                </button>
                            </li>
                        {/each}
                    </ul>
                {/if}
            </div>
        </div>

        <div class="stack right">
            {#if current === null}
                <div class="banner banner-info">
                    Paste a question or search a formula to start. Past questions with their exact inputs are under
                    <strong>Past questions</strong>.
                </div>
            {:else}
                <div class="card stack">
                    <div class="card-header">
                        <h3>{current.title} <span class="muted mono small">{current.key}</span></h3>
                        <button type="button" class="btn btn-sm btn-ghost" onclick={clearForm} title="Esc">Clear</button>
                    </div>
                    <pre class="eqs mono">{current.eqs.join('\n')}</pre>
                    {#if current.notes.length > 0}
                        <div class="banner banner-info small">{current.notes}</div>
                    {/if}
                    <div class="grid">
                        {#each current.vars as v (v.name)}
                            <label class="field">
                                <span>
                                    <span class="mono var">{v.name}</span>
                                    {v.desc}
                                    <span class="muted">[{v.unit === 'dimensionless' ? '–' : v.unit}]</span>
                                </span>
                                <input
                                    type="text"
                                    class="mono"
                                    placeholder={v.default !== null ? `default ${v.default}` : 'unknown'}
                                    bind:value={values[v.name]}
                                />
                            </label>
                        {/each}
                    </div>
                </div>

                <div class="card stack">
                    <div class="card-header"><h3>Result</h3></div>
                    {#if error !== null}
                        <div class="banner banner-danger">{error}</div>
                    {:else if result !== null}
                        {#if result.conflicts.length > 0}
                            <div class="banner banner-danger">
                                <strong>Inconsistent inputs.</strong> These equations don't hold with the values given:
                                <ul class="mono small">
                                    {#each result.conflicts as c (c)}<li>{c}</li>{/each}
                                </ul>
                            </div>
                        {/if}
                        {#if result.found.length === 0}
                            <p class="muted">Nothing determinable yet: fill more values.</p>
                        {/if}
                        {#each result.found as fv (fv.name)}
                            <div class="found">
                                <div class="row">
                                    <span class="mono var">{fv.name}</span>
                                    <span class="muted small">{fv.desc}</span>
                                    {#if fv.shown.length > 1}
                                        <span class="pill pill-warning">{fv.shown.length} roots</span>
                                    {/if}
                                    <input
                                        class="input mono unit"
                                        placeholder="show in…"
                                        title="Display unit, e.g. km/h"
                                        bind:value={display[fv.name]}
                                    />
                                </div>
                                <div class="row values">
                                    {#each fv.shown as s, i (i)}
                                        <button
                                            type="button"
                                            class="value mono"
                                            class:picked={picked?.name === fv.name && picked?.index === i}
                                            onclick={() => (picked = { name: fv.name, index: i })}
                                        >
                                            {s}
                                        </button>
                                    {/each}
                                </div>
                            </div>
                        {/each}
                        {#if result.defaults.length > 0}
                            <p class="muted small">
                                Defaults used: {result.defaults.map((d) => `${d.name} = ${d.shown}`).join(', ')}
                            </p>
                        {/if}
                    {/if}
                </div>

                <div class="card stack">
                    <div class="card-header">
                        <h3>Answer</h3>
                        {#if pickedValue !== null}
                            <span class="muted small">for <span class="mono">{picked?.name} = {pickedValue.shown}</span></span>
                        {/if}
                    </div>
                    {#if pickedValue === null}
                        <p class="muted">Click a result value to check it against the options and copy it.</p>
                    {:else}
                        <div class="row">
                            <span class="mono answer">{formatted}</span>
                            <button type="button" class="btn btn-primary btn-sm" onclick={copyAnswer}>Copy &amp; log</button>
                            <span class="muted small">
                                {settings.precision.kind === 'sig' ? `${settings.precision.n} significant figures` : `${settings.precision.n} decimals`},
                                decimal {settings.decimalComma ? 'comma' : 'point'} (Settings)
                            </span>
                        </div>
                        <label class="field">
                            <span>Multiple-choice options (one per line)</span>
                            <textarea rows="4" class="mono" bind:value={options}></textarea>
                        </label>
                        {#if matching !== null}
                            {#if matching.warning !== null}
                                <div class="banner banner-warning">{matching.warning}</div>
                            {/if}
                            <ul class="options">
                                {#each matching.options as o, i (i)}
                                    <li class:best={o.best}>
                                        <span class="mono">{o.text}</span>
                                        {#if o.rel_diff !== null}
                                            <span class="muted small">{(o.rel_diff * 100).toFixed(2)} % off</span>
                                        {/if}
                                    </li>
                                {/each}
                            </ul>
                        {/if}
                    {/if}
                </div>
            {/if}

            {#if session.log.length > 0}
                <div class="card stack">
                    <div class="card-header">
                        <h3>Session log</h3>
                        <button
                            type="button"
                            class="btn btn-sm"
                            onclick={() =>
                                navigator.clipboard.writeText(
                                    session.log.map((l) => `Q${l.question}\t${l.what}\t${l.inputs}\t${l.answer}`).join('\n'),
                                )}
                        >
                            Copy all
                        </button>
                    </div>
                    <ul class="log mono small">
                        {#each session.log as l (l.question)}
                            <li><strong>Q{l.question}</strong> {l.what} → {l.answer} <span class="muted">{l.inputs}</span></li>
                        {/each}
                    </ul>
                </div>
            {/if}
        </div>
    </div>
</section>

<style>
    .cols {
        display: grid;
        grid-template-columns: minmax(260px, 340px) 1fr;
        gap: var(--space-4);
        align-items: start;
    }
    .small {
        font-size: var(--text-xs);
    }
    .hits,
    .options,
    .log {
        list-style: none;
        margin: 0;
        padding: 0;
        display: flex;
        flex-direction: column;
        gap: var(--space-1);
    }
    .hit {
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
    .hit:hover {
        border-color: var(--accent);
    }
    .title {
        overflow: hidden;
        text-overflow: ellipsis;
        white-space: nowrap;
    }
    .eqs {
        margin: 0;
        padding: var(--space-3);
        background: var(--code-bg);
        border-radius: var(--radius-md);
        font-size: var(--text-sm);
        white-space: pre-wrap;
    }
    .grid {
        display: grid;
        grid-template-columns: repeat(auto-fill, minmax(220px, 1fr));
        gap: var(--space-3);
    }
    .var {
        color: var(--accent);
        font-weight: 600;
    }
    .found {
        display: flex;
        flex-direction: column;
        gap: var(--space-1);
        padding-bottom: var(--space-2);
        border-bottom: 1px solid var(--border);
    }
    .unit {
        margin-left: auto;
        width: 110px;
        padding: 2px var(--space-2);
        font-size: var(--text-sm);
    }
    .values {
        flex-wrap: wrap;
    }
    .value {
        appearance: none;
        background: var(--bg-soft);
        border: 1px solid var(--border);
        border-radius: var(--radius-md);
        color: var(--text);
        padding: var(--space-1) var(--space-3);
        font-size: var(--text-lg);
        cursor: pointer;
    }
    .value:hover {
        border-color: var(--border-strong);
    }
    .value.picked {
        border-color: var(--accent);
        color: var(--accent);
    }
    .answer {
        font-size: var(--text-xl);
        color: var(--accent);
    }
    .options li {
        display: flex;
        justify-content: space-between;
        padding: var(--space-1) var(--space-2);
        border: 1px solid var(--border);
        border-radius: var(--radius-md);
    }
    .options li.best {
        border-color: var(--success);
        color: var(--success);
    }
</style>
