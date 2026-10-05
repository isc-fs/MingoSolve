<!--
    Rules: offline search of the official FS-Rules. The rulebook is copyrighted by Formula Student Germany, so it is
    not bundled: the user loads the PDF they downloaded, once per year, and it is indexed on this computer.
-->
<script lang="ts">
    import { onMount } from 'svelte';

    import Icon from './Icon.svelte';
    import { loadRulebook, pickRulebookPdf, removeRulebook, searchRules } from './rulebook';
    import { highlight, refreshRulebooks, RULE_YEARS, rulebooks, searchYear } from './rulebook.svelte';
    import { session } from './session.svelte';
    import { settings } from './settings.svelte';
    import type { RuleHit } from './types';

    const DOWNLOAD_PAGE = 'https://www.formulastudent.de/rules/';
    const DOWNLOAD_2027 = 'https://www.formulastudent.de/fileadmin/user_upload/all/2027/rules/FS_Rules_2027_v1.0.pdf';

    let hits = $state<RuleHit[]>([]);
    let searched = $state(false);
    let busy = $state<string | null>(null);
    let error = $state<string | null>(null);
    let copied = $state(false);
    let year = $state('');

    const loaded = $derived(rulebooks.loaded);
    const statusOf = (y: string) => loaded.find((r) => r.year === y);
    const searchIn = $derived(year !== '' && statusOf(year) !== undefined ? year : searchYear(settings.rules));

    onMount(() => void refreshRulebooks());

    // Only the reply for the text on screen counts; a slower search for earlier text is dropped.
    let timer: ReturnType<typeof setTimeout> | null = null;
    let latest = 0;
    $effect(() => {
        const q = session.rulesQuery;
        const y = searchIn;
        if (timer !== null) clearTimeout(timer);
        timer = setTimeout(async () => {
            const seq = ++latest;
            if (q.trim().length === 0 || y === null) {
                hits = [];
                searched = false;
                return;
            }
            try {
                const found = await searchRules(q, y, 30);
                if (seq !== latest) return;
                hits = found;
                searched = true;
            } catch (e) {
                if (seq !== latest) return;
                hits = [];
                searched = false;
                error = String(e);
            }
        }, 120);
    });

    async function load(y: string): Promise<void> {
        error = null;
        busy = y;
        try {
            const path = await pickRulebookPdf();
            if (path === null) return;
            await loadRulebook(path, y);
            await refreshRulebooks();
            year = y;
        } catch (e) {
            error = `The ${y} rulebook was not loaded: ${String(e)}`;
        } finally {
            busy = null;
        }
    }

    async function remove(y: string): Promise<void> {
        error = null;
        try {
            await removeRulebook(y);
        } catch (e) {
            error = String(e);
        }
        await refreshRulebooks();
        hits = [];
    }

    async function copyLink(): Promise<void> {
        try {
            await navigator.clipboard.writeText(DOWNLOAD_2027);
            copied = true;
            setTimeout(() => (copied = false), 1800);
        } catch {
            copied = false;
        }
    }

    const when = (s: number): string => new Date(s * 1000).toLocaleDateString();
</script>

<div class="view">
    <header>
        <h1>Rules</h1>
        <p class="muted">Find a rule by its number (<span class="mono">T 2.9.2</span>) or by words (<span class="mono">tread depth</span>). Works offline once a rulebook is loaded.</p>
    </header>

    {#if error !== null}
        <p class="note note-bad" role="alert">{error}</p>
    {/if}

    {#if loaded.length === 0}
        <section class="glass panel empty" aria-label="No rulebook loaded">
            <h2>Load the rulebook once</h2>
            <p>
                The FS-Rules are copyrighted by Formula Student Germany, so MingoSolve doesn't ship them. Download the official PDF,
                load it here, and it is indexed on this computer only.
            </p>
            <ol>
                <li>Download the PDF for your year from <span class="mono">{DOWNLOAD_PAGE}</span>, for example <span class="mono url">{DOWNLOAD_2027}</span>.</li>
                <li>Choose <strong>Load PDF</strong> for that year below and pick the file.</li>
            </ol>
            <div class="row">
                <button type="button" class="btn btn-ghost btn-sm" onclick={copyLink}>
                    <Icon name={copied ? 'check' : 'copy'} size={14} />{copied ? 'Copied' : 'Copy the 2027 link'}
                </button>
            </div>
        </section>
    {/if}

    <section class="glass panel" aria-label="Rulebooks">
        <h2>Rulebooks</h2>
        <ul class="years">
            {#each RULE_YEARS as y (y)}
                {@const st = statusOf(y)}
                <li>
                    <strong class="year">{y}</strong>
                    <span class="state muted small">
                        {#if busy === y}
                            Reading the PDF…
                        {:else if st !== undefined}
                            {st.entries} rules, {st.pages} pages, from {st.source} (loaded {when(st.loaded_at)})
                        {:else}
                            Not loaded
                        {/if}
                    </span>
                    <button type="button" class="btn btn-sm" onclick={() => load(y)} disabled={busy !== null} aria-label="{st !== undefined ? 'Replace' : 'Load'} the {y} PDF">
                        {st !== undefined ? 'Replace PDF' : 'Load PDF'}
                    </button>
                    {#if st !== undefined}
                        <button type="button" class="btn btn-ghost btn-sm" onclick={() => remove(y)} disabled={busy !== null} aria-label="Remove the {y} rulebook">Remove</button>
                    {/if}
                </li>
            {/each}
        </ul>
    </section>

    {#if loaded.length > 0}
        <section class="glass panel" aria-label="Search the rules">
            <div class="bar">
                <Icon name="search" />
                <input
                    class="input"
                    type="search"
                    autocomplete="off"
                    autocorrect="off"
                    autocapitalize="off"
                    spellcheck="false"
                    aria-label="Search the rules"
                    placeholder="T 2.9.2, skidpad, tread depth, insulation monitoring…"
                    bind:value={session.rulesQuery}
                />
                {#if loaded.length > 1}
                    <select class="input year-pick" aria-label="Rules year" value={searchIn} onchange={(e) => (year = e.currentTarget.value)}>
                        {#each loaded as r (r.year)}
                            <option value={r.year}>{r.year}</option>
                        {/each}
                    </select>
                {/if}
            </div>

            {#if hits.length > 0}
                <ul class="hits" aria-label="Rule results">
                    {#each hits as h (h.year + h.id)}
                        <li class="hit">
                            <div class="head">
                                <span class="chip">{h.id}</span>
                                <strong>{h.title}</strong>
                                <span class="muted small">page {h.page} · {h.year}</span>
                            </div>
                            <p class="snippet">{#each highlight(h.snippet, h.matches) as p, i (i)}{#if p.mark}<mark>{p.text}</mark>{:else}{p.text}{/if}{/each}</p>
                        </li>
                    {/each}
                </ul>
            {:else if searched}
                <p class="muted">No rule matches. Try fewer or different words, or a rule number like <span class="mono">EV 4.10</span>.</p>
            {:else}
                <p class="muted">Results appear as you type.</p>
            {/if}
        </section>
    {/if}
</div>

<style>
    .view {
        flex: 1;
        min-height: 0;
        overflow-y: auto;
        padding: var(--space-5) max(var(--space-5), calc((100% - 1100px) / 2)) var(--space-6);
        display: flex;
        flex-direction: column;
        gap: var(--space-3);
    }
    header {
        padding: 0 var(--space-2);
    }
    .panel {
        padding: var(--space-5);
        border-radius: var(--r-xl);
        display: flex;
        flex-direction: column;
        gap: var(--space-3);
    }
    .empty ol {
        margin: 0;
        padding-left: var(--space-5);
        display: flex;
        flex-direction: column;
        gap: var(--space-2);
        line-height: 1.5;
    }
    .url {
        overflow-wrap: anywhere;
    }
    .row {
        display: flex;
        gap: var(--space-3);
    }
    .years {
        list-style: none;
        margin: 0;
        padding: 0;
        display: flex;
        flex-direction: column;
        gap: var(--space-2);
    }
    .years li {
        display: flex;
        align-items: center;
        gap: var(--space-3);
    }
    .year {
        width: 48px;
        font-family: var(--font-mono);
    }
    .state {
        flex: 1;
        min-width: 0;
    }
    .bar {
        display: flex;
        align-items: center;
        gap: var(--space-3);
        color: var(--muted);
    }
    .year-pick {
        width: 96px;
        flex: none;
    }
    .hits {
        list-style: none;
        margin: 0;
        padding: 0;
        display: flex;
        flex-direction: column;
        gap: var(--space-3);
    }
    .hit {
        padding: var(--space-3);
        border-radius: var(--r-md);
        background: var(--well);
        border: 1px solid var(--well-edge);
    }
    .head {
        display: flex;
        align-items: center;
        gap: var(--space-3);
        flex-wrap: wrap;
    }
    .head .muted {
        margin-left: auto;
    }
    .snippet {
        margin: var(--space-2) 0 0;
        line-height: 1.55;
        color: var(--text-2);
    }
    mark {
        background: var(--accent-soft);
        color: var(--text);
        border-radius: 3px;
        padding: 0 2px;
        font-weight: 600;
    }
</style>
