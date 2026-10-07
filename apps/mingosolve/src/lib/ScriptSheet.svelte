<!--
    The open script. Formula scripts: one field per variable (value with units, blank = unknown), solved live; every
    root is listed and the picked one fills the answer slab, which copies it in quiz format and can be checked
    against pasted multiple-choice options. Tool scripts: parameters with their defaults, Enter runs; a tool that
    takes a rule year also shows what the other years would answer. The answer slab sticks to the bottom of the
    scrolling view so it stays on screen with long forms. Copy: ⌘/Ctrl+Enter anywhere in the sheet. Every copy is
    appended to the session log.
    Worked examples (past FS-Quiz questions that use this script) load their inputs in one click.
-->
<script lang="ts">
    import { onDestroy, untrack } from 'svelte';

    import Icon from './Icon.svelte';
    import Tex from './Tex.svelte';
    import { catalog } from './catalog.svelte';
    import { friendlyError, type ErrorContext } from './errors';
    import { solveFormula } from './solve';
    import { runTool } from './tools';
    import { formatAnswer, matchOptions } from './finder';
    import { scriptExamples } from './topics';
    import { openCommand, prettyQuantity, session, togglePin } from './session.svelte';
    import { platform, searchShortcut } from './platform';
    import { RULE_YEARS, settings } from './settings.svelte';
    import { initialValue, placeholderFor } from './toolform';
    import { recordCopy, sessionLog, type LogEntry } from './sessionlog.svelte';
    import type { Matching, ParamInfo, Precision, SolveResult, WorkedExample } from './types';

    const script = $derived(session.script !== null ? catalog.scripts.get(session.script.id) ?? null : null);

    let values = $state<Record<string, string>>({});
    let display = $state<Record<string, string>>({});
    let fromProblem = $state<Record<string, string>>({});
    // The pasted question's own rounding and unit apply to this sheet until the user picks their Settings.
    let useSettings = $state(false);
    let unitTried = false;
    const hint = $derived(session.script?.format ?? null);
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
    let announce = $state('');
    let announceFlip = false;
    // Tool runs: the rule year that produced the answer on screen, and what the other years answer.
    let toolYear = $state<string | null>(null);
    let otherYears = $state<{ year: string; out: string }[]>([]);
    let yearHint = $state<string | null>(null);
    let undo = $state<Snapshot | null>(null);
    let undoTimer: ReturnType<typeof setTimeout> | null = null;
    const shortcut = platform === 'mac' ? '⌘↵' : 'Ctrl+↵';

    interface Snapshot {
        values: Record<string, string>;
        display: Record<string, string>;
        fromProblem: Record<string, string>;
        picked: { name: string; index: number } | null;
        options: string;
        showOptions: boolean;
        useSettings: boolean;
    }

    // Reset and pre-fill whenever a script is opened (nonce changes even for the same script); coming back to the
    // same opened script from another view restores what was typed.
    $effect(() => {
        const open = session.script;
        const s = script;
        if (open === null || s === null) return;
        const nonce = open.nonce;
        untrack(() => {
            // what a tool's fields hold before anything is typed (a bare open waits for Run while they all match)
            const untouched: Record<string, string> = Object.fromEntries((s.tool?.params ?? []).map((p) => [p.name, initialValue(p, settings.rules)]));
            const saved = session.sheet;
            if (saved !== null && saved.nonce === nonce) {
                values = { ...saved.values };
                display = { ...saved.display };
                fromProblem = { ...saved.fromProblem };
                picked = saved.picked;
                options = saved.options;
                showOptions = saved.options.length > 0;
                useSettings = saved.useSettings;
                unitTried = true;
            } else {
                useSettings = false;
                unitTried = false;
                const given = Object.fromEntries(open.values);
                fromProblem = { ...(open.fromProblem ?? {}) };
                if (s.formula !== undefined) {
                    values = Object.fromEntries(s.formula.vars.map((v) => [v.name, given[v.name] ?? '']));
                    display = Object.fromEntries(s.formula.vars.map((v) => [v.name, open.display?.[v.name] ?? '']));
                } else if (s.tool !== undefined) {
                    const vals: Record<string, string> = { ...untouched };
                    (open.positional ?? []).forEach((v, i) => {
                        const p = s.tool!.params[i];
                        if (p !== undefined) vals[p.name] = v;
                    });
                    for (const [k, v] of open.values) vals[k] = v;
                    values = vals;
                    display = {};
                }
                picked = null;
                options = '';
                showOptions = false;
                if (open.target) applyHintUnit(open.target);
            }
            result = null;
            toolOut = null;
            toolYear = null;
            otherYears = [];
            latestRun++;
            dropUndo();
            error = null;
            matching = null;
            void scriptExamples(s.id).then((ex) => (examples = ex));
            // a worked example always runs (its answer may come from the defaults alone); a bare open waits for Run
            const typed = Object.entries(values).some(([k, v]) => v.trim().length > 0 && v !== (untouched[k] ?? ''));
            if (s.tool !== undefined && (typed || open.answer !== undefined)) void run();
        });
    });

    // Keep the working state in the session so leaving and coming back doesn't lose it.
    $effect(() => {
        const open = session.script;
        if (open === null) return;
        const state = { nonce: open.nonce, values: { ...values }, display: { ...display }, fromProblem: { ...fromProblem }, picked, options, useSettings };
        untrack(() => {
            session.sheet = state;
        });
    });

    // Formula scripts solve live; only the reply to the latest input is shown (a slower earlier solve must not win).
    let timer: ReturnType<typeof setTimeout> | null = null;
    let latest = 0;
    $effect(() => {
        const s = script;
        if (s === null || s.formula === undefined) return;
        const given: [string, string][] = Object.entries(values).filter(([, v]) => v.trim().length > 0);
        const disp = { ...display };
        if (timer !== null) clearTimeout(timer);
        timer = setTimeout(async () => {
            const seq = ++latest;
            try {
                const r = await solveFormula(s.id, given, disp);
                if (seq !== latest) return;
                result = r;
                error = null;
                if (picked === null || !result.found.some((f) => f.name === picked!.name)) {
                    picked = preferred(result);
                }
                if (picked !== null) applyHintUnit(picked.name);
            } catch (e) {
                if (seq !== latest) return;
                result = null;
                error = friendlyError(e, formulaErrorContext(s.formula!.vars, given, disp));
            }
        }, 110);
    });

    /** The question's unit fits a variable when both measure the same thing (rad is not percent, Hz is not rad/s). */
    function unitFits(name: string): boolean {
        const v = script?.formula?.vars.find((x) => x.name === name);
        return hint?.unit != null && hint.dims != null && v !== undefined && v.dims === hint.dims;
    }

    /** Once per opened sheet: show the answer variable in the unit the question asks for, if it fits and is blank. */
    function applyHintUnit(name: string): void {
        if (unitTried || useSettings || hint?.unit == null || !unitFits(name)) return;
        unitTried = true;
        if ((display[name] ?? '').trim() === '') display[name] = hint.unit;
    }

    const unitFromQuestion = $derived(!useSettings && hint?.unit != null && picked !== null && unitFits(picked.name) && display[picked.name] === hint.unit);
    const roundingFromQuestion = $derived(!useSettings && hint?.rounding != null ? hint.rounding : null);

    function describeRounding(p: Precision): string {
        if ('sig' in p) return `${p.sig} significant figure${p.sig === 1 ? '' : 's'}`;
        return p.decimals === 0 ? 'whole numbers' : `${p.decimals} decimal${p.decimals === 1 ? '' : 's'}`;
    }

    /** "Rounded to 1 decimal and shown in degrees, as the question asks" (either part alone when only one applies). */
    const formatNote = $derived.by(() => {
        if (hint === null || (!unitFromQuestion && roundingFromQuestion === null)) return null;
        const parts: string[] = [];
        if (roundingFromQuestion !== null) parts.push(`rounded to ${describeRounding(roundingFromQuestion)}`);
        if (unitFromQuestion) parts.push(`shown in ${hint.unit === 'percent' ? 'percent' : (hint.unit_label ?? hint.unit)}`);
        const sentence = parts.join(' and ');
        return `${sentence[0].toUpperCase()}${sentence.slice(1)}, as the question asks`;
    });
    const canUseQuestion = $derived(useSettings && hint !== null && (hint.rounding != null || (hint.unit != null && picked !== null && unitFits(picked.name))));

    function useMySettings(): void {
        useSettings = true;
        if (picked !== null && hint?.unit != null && display[picked.name] === hint.unit) display[picked.name] = '';
        announceText('Using your Settings');
    }

    function useQuestionFormat(): void {
        useSettings = false;
        unitTried = false;
        if (picked !== null) applyHintUnit(picked.name);
        announceText("Using the question's format");
    }

    /** What the answer slab shows first: the root matching a worked example's official answer, else the variable
     *  the question asks for, else the first solved variable. */
    function preferred(r: SolveResult): { name: string; index: number } | null {
        const open = session.script;
        if (open?.answer !== undefined) {
            for (const f of r.found) {
                const i = f.shown.findIndex((s) => Math.abs(parseFloat(s) - open.answer!) <= 0.005 * Math.abs(open.answer!) + 1e-9);
                if (i >= 0) return { name: f.name, index: i };
            }
        }
        const t = open?.target;
        if (t && r.found.some((f) => f.name === t)) return { name: t, index: 0 };
        return r.found.length > 0 ? { name: r.found[0].name, index: 0 } : null;
    }

    // Same rule as the live solve: only the reply to the latest run is shown, for the answer and for the other years.
    let latestRun = 0;
    async function run(): Promise<void> {
        const s = script;
        if (s === null || s.tool === undefined) return;
        const seq = ++latestRun;
        const entries = Object.entries(values);
        const year = s.tool.params.some((p) => p.name === 'rules') ? (values.rules ?? '').trim() || RULE_YEARS[0] : null;
        toolYear = year;
        otherYears = [];
        const others = year !== null && (RULE_YEARS as string[]).includes(year) ? RULE_YEARS.filter((y) => y !== year) : [];
        const main = runTool(s.id, entries);
        for (const y of others) {
            const withYear = entries.map(([k, v]): [string, string] => (k === 'rules' ? [k, y] : [k, v]));
            runTool(s.id, withYear)
                .then((out) => {
                    if (seq !== latestRun) return;
                    otherYears = [...otherYears, { year: y, out }].sort((a, b) => (RULE_YEARS as string[]).indexOf(a.year) - (RULE_YEARS as string[]).indexOf(b.year));
                })
                .catch(() => {});
        }
        try {
            const out = await main;
            if (seq !== latestRun) return;
            toolOut = out;
            error = null;
        } catch (e) {
            if (seq !== latestRun) return;
            toolOut = null;
            error = friendlyError(e, {
                fields: s.tool.params.map((p) => ({ name: p.name, label: p.label, value: values[p.name] ?? '' })),
            });
        }
    }

    /** The fields an engine error can be about: the variables' boxes with what is typed, and the "Show in" boxes. */
    function formulaErrorContext(vars: { name: string; desc: string }[], given: [string, string][], display: Record<string, string>): ErrorContext {
        const typed = new Map(given);
        return {
            fields: [
                ...vars.filter((v) => typed.has(v.name)).map((v) => ({ name: v.name, label: v.desc, value: typed.get(v.name) ?? '' })),
                ...Object.entries(display)
                    .filter(([, v]) => v.trim().length > 0)
                    .map(([k, v]) => ({ name: `@${k}`, label: 'Show in', value: v })),
            ],
            nameOf: (name) => vars.find((v) => v.name === name)?.desc,
        };
    }

    const NUMBER = /^-?[\d.]+(e[-+]?\d+)?$/;
    /** Same 2 % the engine uses to call an option a match. */
    const CLOSE = 0.02;

    const answer = $derived.by(() => {
        if (script?.tool !== undefined) {
            if (toolOut === null) return null;
            // multi-line outputs: the first line is the value when it is a number, the rest is detail
            const full = toolOut.trimEnd();
            const [head, ...rest] = full.split('\n');
            const numeric = NUMBER.test(head.trim());
            return {
                label: script.title,
                shown: numeric ? head.trim() : full,
                n: numeric ? parseFloat(head) : null,
                others: [] as string[],
                extra: numeric ? rest.join('\n') : '',
                full,
                block: !numeric && rest.length > 0,
            };
        }
        if (result === null || picked === null) return null;
        const fv = result.found.find((f) => f.name === picked!.name);
        const shown = fv?.shown[picked.index];
        if (fv === undefined || shown === undefined) return null;
        const n = parseFloat(shown);
        return { label: fv.name, shown, n: Number.isFinite(n) ? n : null, others: fv.shown.filter((_, i) => i !== picked!.index), extra: '', full: shown, block: false };
    });

    /** What a screen reader hears when the answer changes: the value, never the whole multi-line output. */
    const answerSays = $derived.by(() => {
        if (answer === null) return '';
        const lines = answer.shown.split('\n');
        const more = lines.length > 1 ? ` and ${lines.length - 1} more line${lines.length > 2 ? 's' : ''}` : '';
        return `${answer.label} = ${lines[0]}${more}${formatNote !== null ? `. ${formatNote}` : ''}`;
    });

    // The copy text must belong to the answer on screen, so stale formatting replies are dropped too.
    let latestFormat = 0;
    $effect(() => {
        const a = answer;
        const prec: Precision = roundingFromQuestion ?? (settings.precision.kind === 'sig' ? { sig: settings.precision.n } : { decimals: settings.precision.n });
        const comma = settings.decimalComma;
        const opts = options;
        copied = false;
        const seq = ++latestFormat;
        if (a === null || a.n === null) {
            formatted = a?.shown ?? '';
            matching = null;
            return;
        }
        formatted = '';
        void formatAnswer(a.n, prec, comma)
            .then((f) => {
                if (seq === latestFormat) formatted = f;
            })
            .catch((e: unknown) => {
                if (seq === latestFormat) formatted = String(e);
            });
        matching = null;
        if (opts.trim().length > 0) {
            void matchOptions(a.shown, opts)
                .then((m) => {
                    if (seq === latestFormat) matching = m;
                })
                .catch(() => {
                    if (seq === latestFormat) matching = null;
                });
        }
    });

    // If the pasted options fit another rule year clearly better than the selected one, say so (reused questions
    // often carry the old key). Own sequence: a late reply for earlier input must not show.
    let latestHint = 0;
    $effect(() => {
        const a = answer;
        const opts = options;
        const year = toolYear;
        const others = otherYears;
        const seq = ++latestHint;
        yearHint = null;
        if (a === null || a.n === null || year === null || opts.trim().length === 0 || others.length === 0) return;
        const bestOf = (m: Matching) => m.options.find((o) => o.best && o.rel_diff !== null) ?? null;
        void (async () => {
            try {
                const mine = bestOf(await matchOptions(a.shown, opts));
                if (mine !== null && mine.rel_diff! <= CLOSE) return;
                let found: { year: string; text: string; d: number } | null = null;
                for (const o of others) {
                    const head = o.out.split('\n')[0].trim();
                    if (!NUMBER.test(head)) continue;
                    const b = bestOf(await matchOptions(head, opts));
                    if (b !== null && b.rel_diff! <= CLOSE && (found === null || b.rel_diff! < found.d)) found = { year: o.year, text: b.text, d: b.rel_diff! };
                }
                if (seq !== latestHint || found === null) return;
                yearHint = `The ${found.year} rules match option ${found.text}; this question may use ${found.year === 'legacy' ? 'the old key' : `the ${found.year} key`}.`;
            } catch {
                /* the hint is optional */
            }
        })();
    });

    function announceText(text: string): void {
        announceFlip = !announceFlip;
        announce = text + (announceFlip ? '\u200b' : '');
    }

    /** What the sheet holds as typed: variable (or parameter) names in the script's order, blanks left out. */
    function typedInputs(s: NonNullable<typeof script>): [string, string][] {
        const names = s.formula?.vars.map((v) => v.name) ?? s.tool?.params.map((p) => p.name) ?? [];
        return names.filter((n) => (values[n] ?? '').trim().length > 0).map((n) => [n, values[n]]);
    }

    function copyText(text: string, said: string, via: LogEntry['via']): void {
        if (text.length === 0 || script === null) return;
        const s = script;
        const inputs = typedInputs(s);
        const rules = toolYear;
        navigator.clipboard.writeText(text).then(
            () => {
                copied = true;
                announceText(said);
                recordCopy({ scriptId: s.id, title: s.title, inputs, answer: text, rules, via });
            },
            () => announceText('Copy failed'),
        );
    }

    function copy(via: LogEntry['via'] = 'button'): void {
        copyText(formatted, `Copied ${formatted}`, via);
    }

    function copyAll(): void {
        if (answer !== null) copyText(answer.full, `Copied all ${answer.full.split('\n').length} lines`, 'copy-all');
    }

    function onKeydown(e: KeyboardEvent): void {
        if (e.key === 'Enter' && (e.metaKey || e.ctrlKey) && answer !== null) {
            e.preventDefault();
            copy('shortcut');
        }
    }

    function dropUndo(): void {
        if (undoTimer !== null) clearTimeout(undoTimer);
        undoTimer = null;
        undo = null;
    }
    onDestroy(dropUndo);

    function clear(): void {
        undo = $state.snapshot({ values, display, fromProblem, picked, options, showOptions, useSettings });
        if (undoTimer !== null) clearTimeout(undoTimer);
        undoTimer = setTimeout(dropUndo, 8000);
        const start = Object.fromEntries((script?.tool?.params ?? []).map((p) => [p.name, initialValue(p, settings.rules)]));
        for (const k of Object.keys(values)) values[k] = start[k] ?? '';
        display = {};
        fromProblem = {};
        picked = null;
        options = '';
        toolOut = null;
        toolYear = null;
        otherYears = [];
        latestRun++;
        error = null;
    }

    function undoClear(): void {
        const u = undo;
        if (u === null) return;
        dropUndo();
        values = { ...u.values };
        display = { ...u.display };
        fromProblem = { ...u.fromProblem };
        picked = u.picked;
        options = u.options;
        showOptions = u.showOptions;
        useSettings = u.useSettings;
        const t = script?.tool;
        if (t !== undefined && t.params.some((p) => (values[p.name] ?? '').trim().length > 0 && values[p.name] !== initialValue(p, settings.rules))) void run();
        announceText('Restored');
    }

    /** Free-text parameters with a long explanation (circuits, trusses, phases) get a full row. */
    const isWide = (p: ParamInfo): boolean => p.choices === null && !p.switch && !p.number && (p.help?.length ?? 0) > 40;

    const pinned = $derived(script !== null && settings.pinned.includes(script.id));
    /** Typeset name of one of this formula's variables. */
    const texOf = $derived(new Map(script?.formula?.vars.map((v) => [v.name, v.tex]) ?? []));
</script>

{#if script === null}
    <div class="empty glass">
        <Icon name="spark" size={28} />
        <h2>Paste a problem, or pick a script</h2>
        <p class="muted">
            Paste the question text above and the matching scripts open with its values filled in. Or press
            <kbd>{searchShortcut()}</kbd> and type what you need: <span class="mono">spring rate</span>,
            <span class="mono">discharge</span>, <span class="mono">skidpad score</span>.
        </p>
    </div>
{:else}
    <section class="sheet glass" onkeydown={onKeydown} role="presentation">
        <p class="sr-only" role="status" aria-live="polite">{announce}</p>
        <p class="sr-only" role="status" aria-live="polite" data-live="answer">{answerSays}</p>
        <header>
            <div class="titles">
                <span class="label">{script.topic?.name ?? 'Script'} · {script.kind === 'formula' ? 'solve for the blank' : 'tool'}</span>
                <h2>{script.title}</h2>
            </div>
            <label class="qlabel" title="Label for the session log, e.g. Q7">
                <span class="label">Question #</span>
                <input autocomplete="off" autocorrect="off" autocapitalize="off" spellcheck="false" class="input mono" bind:value={sessionLog.label} placeholder="Q7" maxlength="20" />
            </label>
            <button type="button" class="btn btn-ghost btn-sm" class:pinned onclick={() => togglePin(script.id)} aria-pressed={pinned} title={pinned ? 'Unpin from the rail' : 'Pin to the rail'}>
                <Icon name="pin" size={15} />{pinned ? 'Pinned' : 'Pin'}
            </button>
            {#if undo !== null}
                <button type="button" class="btn btn-sm undo" onclick={undoClear} title="Put back what was cleared">Undo clear</button>
            {/if}
            <button type="button" class="btn btn-ghost btn-sm" onclick={clear} title="Clear all fields">Clear</button>
        </header>

        {#if script.formula !== undefined}
            <div class="eqs" title={script.formula.eqs.join('\n')}>
                {#each script.formula.tex as t, i (i)}<Tex tex={t} display />{/each}
            </div>
            {#if script.formula.notes.length > 0}
                <p class="note">{script.formula.notes}</p>
            {/if}
            <div class="fields">
                {#each script.formula.vars as v (v.name)}
                    {@const solved = result?.found.find((f) => f.name === v.name)}
                    <label class="field" data-var={v.name}>
                        <span class="flabel">
                            <span class="fdesc"><span class="sym"><Tex tex={v.tex} /></span> {v.desc}</span>
                            {#if fromProblem[v.name] !== undefined}
                                <span class="from" title="Taken from the problem: {fromProblem[v.name]}">from the problem<span class="sr-only"> ({fromProblem[v.name]})</span></span>
                            {/if}
                        </span>
                        <input autocomplete="off" autocorrect="off" autocapitalize="off" spellcheck="false"
                            class="input mono"
                            class:filled={values[v.name]?.trim().length > 0}
                            class:solved={solved !== undefined}
                            bind:value={values[v.name]}
                            oninput={() => delete fromProblem[v.name]}
                            placeholder={v.default !== null ? `default ${v.default}` : v.unit === 'dimensionless' ? 'unknown' : `unknown [${v.unit_shown}]`}
                        />
                        {#if solved !== undefined && !(values[v.name] ?? '').trim()}
                            <span class="solved-value mono" data-solved={v.name}><span class="sr-only">Solved: </span>= {solved.shown.join(' · ')}</span>
                        {/if}
                        {#if v.hint !== null}<span class="hint">{v.hint}</span>{/if}
                    </label>
                {/each}
            </div>
        {:else if script.tool !== undefined}
            <p class="doc">{script.tool.summary.length > 0 ? script.tool.summary : script.tool.doc}</p>
            {#if script.tool.summary.length > 0}
                <details class="details">
                    <summary>Details</summary>
                    <p class="doc small">{script.tool.doc}</p>
                </details>
            {/if}
            {#if script.tool.params.some((p) => p.unit !== null)}
                <p class="muted small">A plain number is read in the unit next to its box. To use another unit, type it with the number (like 54 km/h).</p>
            {/if}
            <form
                class="fields"
                onsubmit={(e) => {
                    e.preventDefault();
                    void run();
                }}
            >
                {#each script.tool.params as p (p.name)}
                    {@const required = p.default === null}
                    <label class="field" data-var={p.name} class:wide={isWide(p)}>
                        <span class="flabel">
                            <span class="fdesc">{p.label}</span>
                            {#if required}<span class="req">required</span>{/if}
                        </span>
                        {#if p.choices !== null}
                            <select class="input" aria-required={required} bind:value={values[p.name]}>
                                {#if !p.choices.some((c) => c.value === values[p.name])}<option value="" disabled>Choose…</option>{/if}
                                {#each p.choices as c (c.value)}<option value={c.value}>{c.label}</option>{/each}
                            </select>
                        {:else if p.switch}
                            <span class="switch">
                                <input type="checkbox" checked={values[p.name] === '1'} onchange={(e) => (values[p.name] = e.currentTarget.checked ? '1' : '0')} />
                                <span>{values[p.name] === '1' ? 'Yes' : 'No'}</span>
                            </span>
                        {:else}
                            <span class="inputrow">
                                <input autocomplete="off" autocorrect="off" autocapitalize="off" spellcheck="false"
                                    class="input mono"
                                    aria-required={required}
                                    bind:value={values[p.name]}
                                    placeholder={placeholderFor(p)}
                                />
                                {#if p.unit !== null}<span class="unit">{prettyQuantity(p.unit)}</span>{/if}
                            </span>
                        {/if}
                        {#if p.help !== null}<span class="hint">{p.help}</span>{/if}
                    </label>
                {/each}
                <div class="run"><button type="submit" class="btn btn-primary">Run <kbd>↵</kbd></button></div>
            </form>
        {/if}

        <!-- live-solved formulas: the message is read politely, after a pause in typing; a tool run is an explicit action, so its error is an alert -->
        <div class="live" role="status" aria-live="polite">
            {#if script.tool === undefined && error !== null}
                <p class="note note-bad"><strong>Can't solve that.</strong> {error}</p>
            {/if}
            {#if result !== null && result.conflicts.length > 0}
                <div class="note note-bad">
                    <strong>These values contradict each other.</strong>
                    <ul class="mono small">{#each result.conflicts as c (c)}<li>{c}</li>{/each}</ul>
                </div>
            {/if}
        </div>
        {#if script.tool !== undefined && error !== null}
            <p class="note note-bad" role="alert"><strong>Can't solve that.</strong> {error}</p>
        {/if}

        {#if answer !== null}
            <div class="answer sticky" data-tour="answer">
                <div class="answer-row">
                    <span class="a-label">{#if texOf.has(answer.label)}<Tex tex={texOf.get(answer.label) ?? ''} />{:else}{answer.label}{/if} =</span>
                    <span class="a-value" class:block={answer.block}>{answer.shown}</span>
                    {#if toolYear !== null}<span class="pill" title="Rule set that produced this answer">rules {toolYear}</span>{/if}
                    <span class="a-actions">
                        {#if answer.extra.length > 0}
                            <button type="button" class="btn btn-sm copy-all" onclick={copyAll} title="Copy every line of the output">Copy all</button>
                        {/if}
                        <button type="button" class="btn copy" onclick={() => copy()} title="Copy as {formatted.length > 40 ? 'the text' : formatted} ({shortcut})">
                            <Icon name={copied ? 'check' : 'copy'} size={15} />{copied ? 'Copied' : answer.block ? 'Copy' : `Copy ${formatted}`}<kbd>{shortcut}</kbd>
                        </button>
                    </span>
                </div>
                {#if answer.extra.length > 0}
                    <pre class="a-extra">{answer.extra}</pre>
                {/if}
                {#if otherYears.length > 0}
                    <p class="a-note a-years">Other rules: {otherYears.map((o) => `${o.year}: ${o.out.split('\n')[0].trim()}`).join(' · ')}</p>
                {/if}
                {#if answer.others.length > 0}
                    <p class="a-note">Other root{answer.others.length > 1 ? 's' : ''}: {answer.others.join(' · ')}. Pick one below if the physics says so.</p>
                {/if}
                {#if formatNote !== null}
                    <p class="a-note a-format">
                        {formatNote}.
                        <button type="button" class="linklike" onclick={useMySettings}>Use my Settings instead</button>
                    </p>
                {:else if canUseQuestion}
                    <p class="a-note a-format">
                        Using your Settings.
                        <button type="button" class="linklike" onclick={useQuestionFormat}>Use the question's format</button>
                    </p>
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
                            data-var={fv.name}
                            class:on={picked?.name === fv.name && picked?.index === i}
                            onclick={() => (picked = { name: fv.name, index: i })}
                            title={fv.desc}
                        >
                            <span class="sym"><Tex tex={texOf.get(fv.name) ?? fv.name} /></span><span class="mono">{s}</span>
                        </button>
                    {/each}
                {/each}
            </div>
            {#if result.defaults.length > 0}
                <p class="muted small">Assumed {result.defaults.map((d) => `${d.name} = ${d.shown}`).join(', ')}.</p>
            {/if}
        {/if}

        {#if answer !== null && answer.n !== null}
            <div class="check" data-tour="check">
                <button type="button" class="btn btn-ghost btn-sm" onclick={() => (showOptions = !showOptions)} aria-expanded={showOptions}>
                    <Icon name="chevron" size={14} />Check against options
                </button>
                {#if showOptions}
                    <textarea autocomplete="off" autocapitalize="off" spellcheck="false" class="input mono" rows="3" bind:value={options} placeholder={'a) 73.4 A\nb) 77.9 A\nc) 80.4 A'}></textarea>
                    {#if yearHint !== null}<p class="note note-warn note-year"><strong>Rule year:</strong> {yearHint}</p>{/if}
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
                    <button type="button" class="chip chip-quiet" title={ex.what} onclick={() => openCommand(ex.cmd, ex.answer)}>
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
    .qlabel {
        display: flex;
        flex-direction: column;
        gap: 2px;
        width: 92px;
    }
    .qlabel .input {
        min-height: 30px;
        height: 30px;
        padding: 0 var(--space-2);
    }
    .eqs {
        padding: var(--space-2) var(--space-4);
        border-radius: var(--r-md);
        background: var(--code);
        color: var(--text);
        overflow-x: auto;
    }
    .eqs :global(.katex-display) {
        margin: var(--space-2) 0;
        text-align: left;
    }
    .eqs :global(.katex-display > .katex) {
        text-align: left;
        font-size: 1.15em;
    }
    .sym {
        font-size: 1.05em;
        color: var(--text);
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
        display: flex;
        flex-wrap: wrap;
        align-items: baseline;
        gap: 0 var(--space-2);
        min-width: 0;
    }
    .fdesc {
        flex: 0 1 auto;
        min-width: 0;
        overflow-wrap: anywhere;
    }
    .solved-value {
        font-size: var(--text-sm);
        font-weight: 600;
        color: var(--ink-accent);
        overflow-wrap: anywhere;
    }
    /* an empty live region must not add a flex gap of its own */
    .live {
        display: flex;
        flex-direction: column;
        gap: var(--space-4);
    }
    .live:empty {
        margin-top: calc(var(--space-4) * -1);
    }
    .hint {
        font-size: var(--text-xs);
        color: var(--muted);
        line-height: 1.4;
    }
    .req {
        flex: none;
        font-size: var(--text-xs);
        font-weight: 500;
        color: var(--ink-accent);
    }
    .inputrow {
        display: flex;
        align-items: center;
        gap: var(--space-2);
    }
    .inputrow .input {
        flex: 1;
        min-width: 0;
    }
    .unit {
        flex: none;
        font-family: var(--font-mono);
        font-size: var(--text-sm);
        color: var(--muted);
    }
    .switch {
        display: flex;
        align-items: center;
        gap: var(--space-2);
        min-height: 36px;
        color: var(--text);
    }
    .details summary {
        cursor: pointer;
        color: var(--muted);
        font-size: var(--text-sm);
    }
    .from {
        flex: none;
        padding: 0 6px;
        border-radius: var(--r-pill);
        background: var(--accent-soft);
        color: var(--ink-accent);
        font-size: var(--text-xs);
        font-weight: 500;
    }
    .sr-only {
        position: absolute;
        width: 1px;
        height: 1px;
        overflow: hidden;
        clip: rect(0 0 0 0);
        white-space: nowrap;
    }
    .input.filled {
        border-color: var(--accent-edge);
    }
    .input.solved {
        border-color: var(--accent);
    }
    /* Sticks to the bottom of the scrolling view (.view in SolveView) while its place in the sheet is below the fold.
       The solid layer under the tint keeps the fields that scroll beneath it readable (also in [data-solid]). */
    .answer.sticky {
        position: sticky;
        bottom: 0;
        z-index: 2;
        padding: var(--space-4) var(--space-5);
        border-radius: var(--slab-radius);
        background: var(--slab-ornament), var(--answer-bg), var(--glass-solid);
        box-shadow: var(--slab-ring), var(--slab-shadow), var(--shadow);
        border: var(--slab-border);
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
        overflow-wrap: anywhere;
    }
    .a-value.block {
        color: var(--answer-text);
        background: none;
        font-family: var(--font-mono);
        font-size: var(--text-lg);
        white-space: pre-wrap;
        flex-basis: 100%;
    }
    .a-extra {
        margin: 0;
        font-family: var(--font-mono);
        font-size: var(--text-sm);
        line-height: 1.5;
        white-space: pre-wrap;
        overflow-wrap: anywhere;
        color: var(--answer-note);
        max-height: 30vh;
        overflow-y: auto;
    }
    .pill {
        align-self: center;
        padding: 0 8px;
        border-radius: var(--r-pill);
        border: 1px solid var(--answer-edge);
        color: var(--answer-label);
        font-size: var(--text-xs);
        font-weight: 600;
        white-space: nowrap;
    }
    .a-actions {
        margin-left: var(--space-2);
        align-self: center;
        display: flex;
        gap: var(--space-2);
        align-items: center;
    }
    .copy kbd {
        margin-left: var(--space-2);
        color: inherit;
        border-color: currentColor;
    }
    /* the gold ring of the page is invisible on a gold button over the gold-tinted slab */
    .answer :focus-visible {
        outline-color: var(--answer-focus);
        outline-offset: 3px;
    }
    .undo {
        color: var(--ink-accent);
    }
    .copy {
        background: var(--copy-bg);
        color: var(--copy-text);
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
    .linklike {
        padding: 0;
        margin-left: var(--space-2);
        border: 0;
        background: none;
        color: inherit;
        font: inherit;
        text-decoration: underline;
        cursor: pointer;
    }
    .linklike:hover {
        color: var(--answer-label);
    }
    .a-unit {
        display: flex;
        align-items: center;
        gap: var(--space-2);
        color: var(--answer-note);
        font-size: var(--text-sm);
    }
    /* the slab may be a reversed plate, so its one field has its own colours */
    .a-unit .input {
        background: var(--slab-field);
        border-color: var(--slab-field-edge);
        color: var(--slab-field-text);
        width: 160px;
        min-height: 30px;
        padding: 4px 10px;
    }
    .a-unit .input::placeholder {
        color: var(--slab-field-muted);
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
