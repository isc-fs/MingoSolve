<!--
    Topics: every script, filed by engineering domain. Each topic card lists its scripts; one click opens a script
    in Solve. A topic chosen from Solve's shortcut list opens expanded and scrolled into view.
-->
<script lang="ts">
    import { tick } from 'svelte';

    import { catalog } from './catalog.svelte';
    import { openScript, session } from './session.svelte';

    let filter = $state('');

    const topics = $derived.by(() => {
        const w = filter.toLowerCase().trim();
        if (w.length === 0) return catalog.topics;
        return catalog.topics
            .map((t) => ({ ...t, scripts: t.scripts.filter((s) => `${s.title} ${s.id} ${t.name}`.toLowerCase().includes(w)) }))
            .filter((t) => t.scripts.length > 0);
    });

    $effect(() => {
        const id = session.topic;
        if (id === null) return;
        void tick().then(() => {
            document.getElementById(`topic-${id}`)?.scrollIntoView({ block: 'start', behavior: 'smooth' });
            session.topic = null;
        });
    });
</script>

<div class="view">
    <header class="head">
        <div>
            <h1>Topics</h1>
            <p class="muted">{catalog.scripts.size} scripts. Formulas solve for whatever you leave blank; tools run a procedure.</p>
        </div>
        <input autocomplete="off" autocorrect="off" autocapitalize="off" spellcheck="false" class="input filter" placeholder="Filter scripts" bind:value={filter} aria-label="Filter scripts" />
    </header>

    <div class="grid">
        {#each topics as t (t.id)}
            <section class="topic glass" id="topic-{t.id}" data-hue={t.hue}>
                <div class="top">
                    <span class="dot"></span>
                    <h2>{t.name}</h2>
                    <span class="mono small muted">{t.scripts.length}</span>
                </div>
                <p class="muted small blurb">{t.blurb}</p>
                <ul>
                    {#each t.scripts as s (s.id)}
                        <li>
                            <button type="button" class="script" onclick={() => openScript(s.id)}>
                                <span>{s.title}</span>
                                {#if s.kind === 'tool'}<span class="chip chip-quiet">tool</span>{/if}
                            </button>
                        </li>
                    {/each}
                </ul>
            </section>
        {/each}
    </div>
</div>

<style>
    .view {
        flex: 1;
        min-height: 0;
        overflow-y: auto;
        padding: var(--space-5) var(--space-5) var(--space-6);
        display: flex;
        flex-direction: column;
        gap: var(--space-4);
    }
    .head {
        display: flex;
        align-items: flex-end;
        gap: var(--space-4);
        padding: 0 var(--space-2);
    }
    .head > div {
        flex: 1;
    }
    .filter {
        max-width: 260px;
    }
    .grid {
        display: grid;
        grid-template-columns: repeat(auto-fill, minmax(300px, 1fr));
        gap: var(--space-3);
        align-items: start;
    }
    .topic {
        padding: var(--space-4);
        border-radius: var(--r-xl);
        scroll-margin-top: var(--space-4);
    }
    .top {
        display: flex;
        align-items: center;
        gap: var(--space-2);
    }
    .top h2 {
        flex: 1;
        font-size: var(--text-lg);
    }
    .dot {
        width: 10px;
        height: 10px;
        border-radius: 50%;
        background: var(--hue-green);
    }
    .topic[data-hue='teal'] .dot {
        background: var(--hue-teal);
    }
    .topic[data-hue='gold'] .dot {
        background: var(--hue-gold);
    }
    .blurb {
        margin: var(--space-1) 0 var(--space-3);
    }
    ul {
        list-style: none;
        margin: 0;
        padding: 0;
        display: flex;
        flex-direction: column;
        gap: 2px;
    }
    .script {
        appearance: none;
        width: 100%;
        display: flex;
        align-items: center;
        justify-content: space-between;
        gap: var(--space-2);
        padding: 7px var(--space-3);
        border: none;
        border-radius: var(--r-sm);
        background: transparent;
        cursor: pointer;
        text-align: left;
        font-size: var(--text-sm);
        color: var(--text-2);
    }
    .script:hover {
        background: var(--hover);
        color: var(--text);
    }
</style>
