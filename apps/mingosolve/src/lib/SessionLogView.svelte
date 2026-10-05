<!--
    Session log: every copied answer, newest first, for rehearsal. A mock quiz resets the timer and tags the entries
    that follow. Export CSV downloads the whole log (columns in docs/cheat-sheet.md); the separator can be a semicolon
    for spreadsheets whose locale uses a decimal comma. A separate view from Help because it is a working surface
    (table, export, clear), while Help is reference.
-->
<script lang="ts">
    import { entriesToCsv, inputsCell, LOG_COLUMNS, type Delimiter } from './csv';
    import { clearLog, endMockQuiz, sessionLog, startMockQuiz } from './sessionlog.svelte';
    import { settings } from './settings.svelte';

    let confirming = $state(false);
    let separator = $state<Delimiter | null>(null);
    let note = $state('');

    // Spreadsheets in decimal-comma locales (Spanish Excel) split on semicolons, so offer that when answers copy with a comma.
    const delimiter = $derived<Delimiter>(separator ?? (settings.decimalComma ? ';' : ','));
    const newest = $derived([...sessionLog.entries].reverse());

    const pad = (n: number): string => String(n).padStart(2, '0');
    const stamp = (): string => {
        const d = new Date();
        return `${d.getFullYear()}${pad(d.getMonth() + 1)}${pad(d.getDate())}-${pad(d.getHours())}${pad(d.getMinutes())}`;
    };
    const clock = (iso: string): string => new Date(iso).toLocaleTimeString([], { hour: '2-digit', minute: '2-digit', second: '2-digit' });
    const elapsed = (s: number | null): string => (s === null ? '' : s >= 60 ? `${Math.floor(s / 60)} min ${s % 60} s` : `${s} s`);

    function exportCsv(): void {
        const blob = new Blob([entriesToCsv(sessionLog.entries, delimiter)], { type: 'text/csv;charset=utf-8' });
        const url = URL.createObjectURL(blob);
        const a = document.createElement('a');
        a.href = url;
        a.download = `mingosolve-session-${stamp()}.csv`;
        document.body.appendChild(a);
        a.click();
        a.remove();
        setTimeout(() => URL.revokeObjectURL(url), 1000);
        note = `Exported ${sessionLog.entries.length} entries.`;
    }

    // The webview may not offer downloads on every platform; pasting into a sheet always works.
    async function copyCsv(): Promise<void> {
        try {
            await navigator.clipboard.writeText(entriesToCsv(sessionLog.entries, delimiter).replace(/^﻿/, ''));
            note = 'CSV copied: paste it into a spreadsheet.';
        } catch {
            note = 'Could not copy.';
        }
    }

    function clear(): void {
        clearLog();
        confirming = false;
        note = 'Log cleared.';
    }
</script>

<div class="view">
    <header>
        <h1>Session log</h1>
        <p class="muted">Each copied answer is added here, with the time since you pasted the problem. Kept on this computer; the last 500 entries.</p>
    </header>

    <section class="glass panel" aria-labelledby="h-mock">
        <h2 id="h-mock">Mock quiz</h2>
        {#if sessionLog.quiz !== null}
            <p>Running: <span class="mono">{sessionLog.quiz.id}</span>. New entries carry this id.</p>
            <div class="row">
                <button type="button" class="btn" onclick={startMockQuiz}>Restart the timer</button>
                <button type="button" class="btn btn-ghost" onclick={endMockQuiz}>End mock quiz</button>
            </div>
        {:else}
            <p class="muted">Start one to rehearse: the timer resets and every entry that follows is tagged with the same id.</p>
            <div class="row"><button type="button" class="btn btn-primary" onclick={startMockQuiz}>Start a mock quiz</button></div>
        {/if}
    </section>

    <section class="glass panel" aria-labelledby="h-export">
        <h2 id="h-export">Export</h2>
        <div class="row">
            <button type="button" class="btn" onclick={exportCsv} disabled={sessionLog.entries.length === 0}>Export CSV</button>
            <button type="button" class="btn btn-ghost" onclick={copyCsv} disabled={sessionLog.entries.length === 0}>Copy CSV</button>
            <label class="field">
                <span class="label">Separator</span>
                <select class="input" value={delimiter} onchange={(e) => (separator = e.currentTarget.value as Delimiter)}>
                    <option value=",">Comma (Google Sheets, English Excel)</option>
                    <option value=";">Semicolon (Excel with a decimal comma)</option>
                </select>
            </label>
        </div>
        <p class="muted small">
            Columns: <span class="mono">{LOG_COLUMNS.join(', ')}</span>. <span class="mono">inputs</span> is
            <span class="mono">name=value; name=value</span> as typed; <span class="mono">elapsed_s</span> is seconds since the problem was
            pasted; <span class="mono">method</span> is button, shortcut or copy-all. Timestamps are UTC.
        </p>
        <p class="sr-only" role="status" aria-live="polite">{note}</p>
        {#if note !== ''}<p class="small" aria-hidden="true">{note}</p>{/if}
    </section>

    <section class="glass panel wide" aria-labelledby="h-entries">
        <div class="head">
            <h2 id="h-entries">Entries ({sessionLog.entries.length})</h2>
            {#if confirming}
                <span class="row">
                    <span>Delete all {sessionLog.entries.length} entries?</span>
                    <button type="button" class="btn btn-sm" onclick={clear}>Yes, clear the log</button>
                    <button type="button" class="btn btn-ghost btn-sm" onclick={() => (confirming = false)}>Cancel</button>
                </span>
            {:else}
                <button type="button" class="btn btn-ghost btn-sm" onclick={() => (confirming = true)} disabled={sessionLog.entries.length === 0}>Clear log</button>
            {/if}
        </div>
        {#if newest.length === 0}
            <p class="muted">Nothing yet. Copy an answer from any script and it appears here.</p>
        {:else}
            <div class="scroll">
                <table>
                    <caption class="sr-only">Copied answers, newest first</caption>
                    <thead>
                        <tr>
                            <th scope="col">Time</th>
                            <th scope="col">Question</th>
                            <th scope="col">Script</th>
                            <th scope="col">Answer</th>
                            <th scope="col">Inputs</th>
                            <th scope="col">Elapsed</th>
                        </tr>
                    </thead>
                    <tbody>
                        {#each newest as e, i (e.at + i)}
                            <tr>
                                <td class="mono nowrap">{clock(e.at)}{#if e.quiz !== null}<span class="muted small"> · {e.quiz}</span>{/if}</td>
                                <td class="mono">{e.question}</td>
                                <td>{e.title}{#if e.rules !== null}<span class="muted small"> · rules {e.rules}</span>{/if}</td>
                                <td class="mono answer">{e.answer}</td>
                                <td class="mono small">{inputsCell(e.inputs)}</td>
                                <td class="nowrap">{elapsed(e.elapsedS)}</td>
                            </tr>
                        {/each}
                    </tbody>
                </table>
            </div>
        {/if}
    </section>
</div>

<style>
    .view {
        flex: 1;
        min-height: 0;
        overflow-y: auto;
        padding: var(--space-5) max(var(--space-5), calc((100% - 1400px) / 2)) var(--space-6);
        display: grid;
        grid-template-columns: repeat(auto-fill, minmax(min(100%, 440px), 1fr));
        align-content: start;
        align-items: start;
        gap: var(--space-3);
    }
    header {
        grid-column: 1 / -1;
        padding: 0 var(--space-2);
    }
    .panel {
        padding: var(--space-5);
        border-radius: var(--r-xl);
        display: flex;
        flex-direction: column;
        gap: var(--space-3);
        line-height: 1.55;
    }
    .panel.wide {
        grid-column: 1 / -1;
    }
    p {
        margin: 0;
    }
    .row {
        display: flex;
        flex-wrap: wrap;
        gap: var(--space-3);
        align-items: flex-end;
    }
    .head {
        display: flex;
        align-items: center;
        justify-content: space-between;
        gap: var(--space-3);
        flex-wrap: wrap;
    }
    .field {
        display: flex;
        flex-direction: column;
        gap: 6px;
    }
    .scroll {
        overflow-x: auto;
    }
    table {
        width: 100%;
        border-collapse: collapse;
    }
    th,
    td {
        text-align: left;
        padding: var(--space-2) var(--space-3);
        border-bottom: 1px solid var(--well-edge);
        vertical-align: top;
    }
    th {
        font-size: var(--text-sm);
        color: var(--muted);
        font-weight: 600;
    }
    .nowrap {
        white-space: nowrap;
    }
    .answer {
        font-weight: 600;
        word-break: break-word;
    }
</style>
