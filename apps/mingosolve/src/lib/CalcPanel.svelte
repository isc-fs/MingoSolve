<!--
    Calculator, docked on the right and collapsible: unit-aware expressions (0.5*280kg*(100km/h)**2 -> kJ). Enter
    evaluates; results stack newest first; click one to copy its number. Past questions load and evaluate in one click.
-->
<script lang="ts">
    import Disclosure from './Disclosure.svelte';
    import Icon from './Icon.svelte';
    import { calc } from './solve';
    import { openCommand, session } from './session.svelte';
    import { layout } from './layout.svelte';
    import { settings } from './settings.svelte';
    import { calcExamples } from './topics';
    import type { WorkedExample } from './types';

    interface Entry {
        id: number;
        expr: string;
        /** The result, or the engine's message when it failed. */
        out: string;
        failed: boolean;
    }

    let history = $state<Entry[]>([]);
    let copied = $state<number | null>(null);
    let examples = $state<WorkedExample[] | null>(null);
    let entryId = 0;
    let copiedTimer: ReturnType<typeof setTimeout> | undefined;
    let seenRun = session.calcRun;
    /** One polite message for screen readers: the latest result or error, or the number just copied. */
    let said = $state('');
    // Folded into its button below the width breakpoint; settings.calcOpen is the user's choice above it.
    const shown = $derived(layout.narrow ? layout.calcPeek : settings.calcOpen);

    function hide(): void {
        if (layout.narrow) layout.calcPeek = false;
        else settings.calcOpen = false;
    }
    function show(): void {
        if (layout.narrow) layout.calcPeek = true;
        else settings.calcOpen = true;
    }

    // a past question opened from anywhere (openCommand) fills the box and evaluates it
    $effect(() => {
        if (session.calcRun !== seenRun) {
            seenRun = session.calcRun;
            void evaluate();
        }
    });

    async function evaluate(): Promise<void> {
        const expr = session.calcInput.trim();
        if (expr.length === 0) return;
        let out: string;
        try {
            out = await calc(expr);
        } catch (e) {
            out = `error: ${String(e)}`;
        }
        const failed = out.startsWith('error:');
        entryId += 1;
        said = failed ? `Error: ${out.slice('error:'.length).trim()}` : `${expr} = ${out}`;
        history = [{ id: entryId, expr, out: failed ? out.slice('error:'.length).trim() : out, failed }, ...history].slice(0, 40);
    }

    async function copy(h: Entry): Promise<void> {
        try {
            await navigator.clipboard.writeText(h.out.split(' ')[0]);
        } catch {
            return;
        }
        copied = h.id;
        said = 'Copied';
        clearTimeout(copiedTimer);
        copiedTimer = setTimeout(() => {
            copied = null;
            said = '';
        }, 1600);
    }

    // the list is fetched the first time the fold is open (also when it was left open last time)
    $effect(() => {
        if (shown && settings.calcPastOpen && examples === null) void calcExamples().then((ex) => (examples = ex));
    });
</script>

{#if shown}
    <aside class="calc glass" aria-label="Calculator">
        <div class="head">
            <Icon name="calc" />
            <h3>Calculator</h3>
            <button type="button" class="btn btn-ghost btn-sm" onclick={hide} aria-label="Hide calculator">
                <Icon name="x" size={14} />
            </button>
        </div>
        <input autocomplete="off" autocorrect="off" autocapitalize="off" spellcheck="false"
            class="input mono"
            placeholder="2*pi*9.125m/4.9s -> km/h"
            bind:value={session.calcInput}
            onkeydown={(e) => e.key === 'Enter' && evaluate()}
            aria-label="Expression"
        />
        <Disclosure label="Past questions" id="calc-past" bind:open={settings.calcPastOpen}>
            <div class="chips">
                {#each examples ?? [] as ex (ex.id)}
                    <button type="button" class="chip chip-quiet" title={ex.what || ex.cmd} onclick={() => openCommand(ex.cmd, ex.answer)}>
                        Q{ex.id}{ex.what ? ` · ${ex.what.length > 24 ? ex.what.slice(0, 24) + '…' : ex.what}` : ''}
                    </button>
                {/each}
            </div>
        </Disclosure>
        <ul>
            {#each history as h (h.id)}
                <li>
                    {#if h.failed}
                        <div class="entry bad">
                            <span class="mono muted small">{h.expr}</span>
                            <span class="problem">{h.out}</span>
                        </div>
                    {:else}
                        <button type="button" class="entry" title="Copy the number" onclick={() => copy(h)}>
                            <span class="mono muted small">{h.expr}</span>
                            <span class="mono result">{h.out}</span>
                            {#if copied === h.id}<span class="copied small">Copied</span>{/if}
                        </button>
                    {/if}
                </li>
            {/each}
        </ul>
        <p class="sr-only" role="status" aria-live="polite">{said}</p>
        {#if history.length === 0}
            <p class="muted small">Units work everywhere: <span class="mono">km/h</span>, <span class="mono">rpm</span>, <span class="mono">bar</span>, <span class="mono">Ah</span>, <span class="mono">g0</span>. End with <span class="mono">-&gt; unit</span> to convert.</p>
        {/if}
    </aside>
{:else}
    <button type="button" class="open glass" onclick={show} aria-label="Show calculator" title="Calculator">
        <Icon name="calc" />
    </button>
{/if}

<style>
    .calc {
        width: 280px;
        flex: 0 0 280px;
        padding: var(--space-4);
        border-width: 0 0 0 1px;
        border-radius: 0;
        box-shadow: none;
        display: flex;
        flex-direction: column;
        gap: var(--space-3);
        min-height: 0;
    }
    .head {
        display: flex;
        align-items: center;
        gap: var(--space-2);
        color: var(--ink-accent);
    }
    .head h3 {
        flex: 1;
        color: var(--text);
    }
    ul {
        list-style: none;
        margin: 0;
        padding: 0;
        overflow-y: auto;
        display: flex;
        flex-direction: column;
        gap: 6px;
    }
    .entry {
        appearance: none;
        width: 100%;
        text-align: left;
        padding: var(--space-2) var(--space-3);
        border-radius: var(--r-md);
        border: 1px solid var(--well-edge);
        background: var(--well);
        cursor: pointer;
        display: flex;
        flex-direction: column;
        gap: 2px;
    }
    .entry:hover {
        border-color: var(--accent-edge);
    }
    .result {
        color: var(--ink-accent);
        font-size: var(--text-lg);
    }
    .entry.bad {
        cursor: default;
        border-color: var(--bad);
        background: var(--bad-soft);
    }
    .problem {
        color: var(--text);
        font-size: var(--text-sm);
        line-height: 1.4;
    }
    .copied {
        color: var(--ink-accent);
        font-weight: 600;
    }
    .chips {
        display: flex;
        flex-wrap: wrap;
        gap: 6px;
        max-height: 180px;
        overflow-y: auto;
    }
    .chips .chip {
        border: none;
        cursor: pointer;
        font-family: var(--font-sans);
    }
    .sr-only {
        position: absolute;
        width: 1px;
        height: 1px;
        overflow: hidden;
        clip: rect(0 0 0 0);
        white-space: nowrap;
    }
    .open {
        align-self: flex-start;
        margin: var(--space-3) var(--space-3) 0 0;
        width: 44px;
        height: 44px;
        border-radius: 14px;
        display: grid;
        place-items: center;
        cursor: pointer;
        color: var(--ink-accent);
    }
</style>
