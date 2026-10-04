<!--
    Past questions: FS-Quiz questions solved with the engine, with the official answer and the exact inputs.
    "Open" loads one into Solve / Chain / Tools pre-filled; a warning marks official keys known to be off.
-->
<script lang="ts">
    import { onMount } from 'svelte';

    import { listExamples } from './finder';
    import { listFormulas } from './solve';
    import { listTools } from './tools';
    import { openCommand } from './commands';
    import { session } from './session.svelte';
    import type { ExampleInfo } from './types';

    let examples = $state<ExampleInfo[]>([]);
    let filter = $state('');
    let formulaKeys = new Set<string>();
    let toolNames = new Set<string>();

    const shown = $derived.by(() => {
        const words = filter.toLowerCase().split(/\s+/).filter((w) => w.length > 0);
        return examples.filter((e) => {
            const hay = `q${e.id} ${e.what} ${e.cmd}`.toLowerCase();
            return words.every((w) => hay.includes(w));
        });
    });

    onMount(async () => {
        examples = await listExamples();
        formulaKeys = new Set((await listFormulas()).map((f) => f.key));
        toolNames = new Set((await listTools()).map((t) => t.name));
        const req = session.request;
        if (req !== null && req.view === 'examples') {
            session.request = null;
            filter = req.id;
        }
    });
</script>

<section class="view">
    <header>
        <div>
            <h2>Past questions</h2>
            <p>
                Solved FS-Quiz questions with the official answer. Use them to see how a question type is entered;
                “Open” loads it pre-filled. Data: FS-Quiz (fs-quiz.eu), ODbL.
            </p>
        </div>
    </header>

    <input class="input" placeholder="filter: skidpad, Q378, discharge…" bind:value={filter} />

    <ul class="list">
        {#each shown as e (e.id)}
            <li class="card card-tight stack stack-sm">
                <div class="row row-spread">
                    <span><strong>Q{e.id}</strong> {e.what}</span>
                    <span class="row">
                        <span class="pill pill-success mono">{e.answer}</span>
                        <button type="button" class="btn btn-sm" onclick={() => openCommand(e.cmd, formulaKeys, toolNames)}>Open</button>
                    </span>
                </div>
                <code class="mono small">{e.cmd}</code>
                {#if e.warning !== null}
                    <div class="banner banner-warning small"><strong>Official key is off.</strong> {e.warning}</div>
                {/if}
            </li>
        {/each}
    </ul>
</section>

<style>
    .list {
        list-style: none;
        margin: 0;
        padding: 0;
        display: flex;
        flex-direction: column;
        gap: var(--space-2);
    }
    .small {
        font-size: var(--text-xs);
    }
    code {
        color: var(--text-secondary);
        word-break: break-all;
    }
</style>
