<!--
    Solve (home). Paste the problem text at the top: the scripts that fit appear as a row of matches under it, each
    with how many of the problem's values it fills in; the best one is a click away and the sheet keeps the full
    width. With nothing pasted and no script open, the script library sits below. Pasting anywhere outside a
    field lands here (App.svelte).
-->
<script lang="ts">
    import Icon from './Icon.svelte';
    import Library from './Library.svelte';
    import ScriptSheet from './ScriptSheet.svelte';
    import { catalog, shortName } from './catalog.svelte';
    import { findQuestion } from './finder';
    import { openScript, prettyQuantity, session } from './session.svelte';
    import type { Hit } from './types';

    let hits = $state<Hit[]>([]);
    let quantities = $state<string[]>([]);

    // Only the reply for the text on screen counts; a slower search for earlier text is dropped.
    let timer: ReturnType<typeof setTimeout> | null = null;
    let latest = 0;
    $effect(() => {
        const q = session.problem;
        if (timer !== null) clearTimeout(timer);
        timer = setTimeout(async () => {
            const seq = ++latest;
            if (q.trim().length === 0) {
                hits = [];
                quantities = [];
                session.problemFills = {};
                return;
            }
            const found = await findQuestion(q).catch(() => null);
            if (seq !== latest) return;
            if (found === null) {
                hits = [];
                quantities = [];
                session.problemFills = {};
                return;
            }
            hits = found.hits;
            quantities = found.quantities;
            session.problemFills = Object.fromEntries(found.hits.map((h) => [h.id, { values: h.prefill, target: h.target }]));
        }, 160);
    });

    // With a script open, a chip is "used" when one of the sheet's fields holds that value (as pre-filled or typed).
    const sheetValues = $derived(
        session.script !== null && session.sheet?.nonce === session.script.nonce
            ? Object.values(session.sheet.values)
                  .filter((v) => v.trim().length > 0)
                  .map((v) => prettyQuantity(v).replace(/\s+/g, ''))
            : null,
    );
    const used = (q: string): boolean | null => (sheetValues === null ? null : sheetValues.includes(q.replace(/\s+/g, '')));

    const hasProblem = $derived(session.problem.trim().length > 0);
</script>

<div class="view">
    <div class="problem glass">
        <div class="phead">
            <label class="label" for="problem">Problem</label>
            {#if hasProblem}
                <button type="button" class="btn btn-ghost btn-sm" onclick={() => (session.problem = '')} aria-label="Clear the problem">
                    <Icon name="x" size={14} />Clear
                </button>
            {/if}
        </div>
        <textarea autocomplete="off" autocapitalize="off" spellcheck="false"
            id="problem"
            class="input"
            rows={hasProblem ? 2 : 3}
            bind:value={session.problem}
            placeholder="Paste the question text (or paste anywhere in the window). Values with units like 3.8 V, 100 km/h or 0.08 Ω are picked up and filled in."
        ></textarea>
        {#if quantities.length > 0}
            <div class="chips" aria-label="Values found in the problem">
                {#each quantities as q (q)}
                    {@const u = used(q)}
                    <span class="chip" class:used={u === true} class:unused={u === false}>
                        {#if u === true}<Icon name="check" size={12} />{/if}
                        {q}
                        {#if u === true}<span class="sr-only">used in this script</span>{/if}
                        {#if u === false}<span class="not-used">not used</span>{/if}
                    </span>
                {/each}
            </div>
        {/if}
        {#if hasProblem}
            <div class="matches" aria-label="Scripts that fit">
                <span class="label">Matches</span>
                {#each hits as h, i (h.kind + h.id)}
                    <button
                        type="button"
                        class="match"
                        class:best={i === 0}
                        class:on={session.script?.id === h.id}
                        title={catalog.scripts.get(h.id)?.title ?? h.id}
                        onclick={() => openScript(h.id)}
                    >
                        <span>{shortName(h.id)}</span>
                        {#if h.prefill.length > 0}<span class="fills">{h.prefill.length}</span>{/if}
                    </button>
                {/each}
                {#if hits.length === 0}<span class="muted small">No script fits yet. Try <kbd>⌘K</kbd> with a keyword.</span>{/if}
            </div>
        {/if}
    </div>

    {#if session.script === null && !hasProblem}
        <Library />
    {:else}
        <ScriptSheet />
    {/if}
</div>

<style>
    .view {
        flex: 1;
        min-height: 0;
        overflow-y: auto;
        padding: 0 var(--space-5) var(--space-6);
        display: flex;
        flex-direction: column;
        gap: var(--space-4);
    }
    .problem {
        padding: var(--space-3) var(--space-4) var(--space-4);
        border-radius: var(--r-xl);
        display: flex;
        flex-direction: column;
        gap: var(--space-2);
    }
    .phead {
        display: flex;
        align-items: center;
        justify-content: space-between;
        min-height: 28px;
    }
    .problem textarea {
        background: transparent;
        border-color: transparent;
        padding: 0;
        font-size: var(--text-lg);
        min-height: 48px;
    }
    .problem textarea:focus {
        box-shadow: none;
        border-color: transparent;
    }
    .chips,
    .matches {
        display: flex;
        flex-wrap: wrap;
        align-items: center;
        gap: var(--space-2);
    }
    .matches {
        padding-top: var(--space-2);
        border-top: 1px solid var(--glass-edge);
    }
    .chip.used {
        background: var(--good-soft);
        color: var(--good);
    }
    .chip.unused {
        background: transparent;
        color: var(--text-2);
        border: 1px dashed var(--warn);
    }
    .not-used {
        font-family: var(--font-sans);
        color: var(--warn);
        font-weight: 600;
    }
    .sr-only {
        position: absolute;
        width: 1px;
        height: 1px;
        overflow: hidden;
        clip: rect(0 0 0 0);
        white-space: nowrap;
    }
    .match {
        appearance: none;
        display: inline-flex;
        align-items: center;
        gap: var(--space-2);
        height: 32px;
        padding: 0 var(--space-2) 0 var(--space-3);
        border-radius: var(--r-pill);
        border: 1px solid var(--well-edge);
        background: var(--well);
        cursor: pointer;
        font-size: var(--text-sm);
        transition: border-color var(--motion);
    }
    .match:hover,
    .match.best {
        border-color: var(--accent-edge);
    }
    .match.on {
        border-color: var(--accent);
        box-shadow: 0 0 0 3px var(--accent-soft);
    }
    .fills {
        min-width: 22px;
        height: 22px;
        padding: 0 6px;
        border-radius: var(--r-pill);
        display: grid;
        place-items: center;
        background: var(--accent-soft);
        color: var(--ink-accent);
        font-family: var(--font-mono);
        font-size: 11px;
        font-weight: 600;
    }
</style>
