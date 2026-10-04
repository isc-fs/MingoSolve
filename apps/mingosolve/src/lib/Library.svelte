<!--
    The script library: topics on the left, the chosen topic's scripts on the right (in columns on wide windows).
    Arrow keys move through topics and the scripts follow at once; → enters the list; typing anywhere in it jumps to
    the filter, which ranks every script across topics (same search as ⌘K). Used on Solve's home and in Topics.
-->
<script lang="ts">
    import { tick } from 'svelte';

    import Icon from './Icon.svelte';
    import { catalog, searchScripts, symbols, type Script } from './catalog.svelte';
    import { openScript, session, togglePin } from './session.svelte';
    import { settings } from './settings.svelte';

    let { autofocus = false }: { autofocus?: boolean } = $props();

    const PINNED = '__pinned';
    let filter = $state('');
    let selected = $state<string>(session.topic ?? catalog.topics[0]?.id ?? '');
    let filterInput = $state<HTMLInputElement | null>(null);
    let indexEl = $state<HTMLElement | null>(null);
    let listEl = $state<HTMLElement | null>(null);

    const scriptsOf = (ids: string[]): Script[] => ids.flatMap((id) => catalog.scripts.get(id) ?? []);
    const groups = $derived([
        ...(settings.pinned.length > 0 ? [{ id: PINNED, name: 'Pinned', hue: 'gold', scripts: scriptsOf(settings.pinned) }] : []),
        ...catalog.topics.map((t) => ({ id: t.id, name: t.name, hue: t.hue, scripts: scriptsOf(t.scripts.map((s) => s.id)) })),
    ]);
    const searching = $derived(filter.trim().length > 0);
    const found = $derived(searching ? searchScripts(filter, 60, [...settings.pinned, ...settings.recent]) : []);
    const matchCount = $derived(new Map(groups.map((g) => [g.id, found.filter((s) => g.scripts.includes(s)).length])));
    const current = $derived(groups.find((g) => g.id === selected) ?? groups[0]);
    const shown = $derived(searching ? found : (current?.scripts ?? []));

    $effect(() => {
        if (session.topic !== null) {
            selected = session.topic;
            session.topic = null;
        }
    });

    $effect(() => {
        if (autofocus) void tick().then(() => filterInput?.focus());
    });

    function pick(id: string): void {
        selected = id;
        filter = '';
    }

    function focusTopic(i: number): void {
        const g = groups[Math.max(0, Math.min(groups.length - 1, i))];
        pick(g.id);
        void tick().then(() => indexEl?.querySelector<HTMLElement>(`[data-topic="${g.id}"]`)?.focus());
    }

    function indexKey(e: KeyboardEvent, i: number): void {
        if (e.key === 'ArrowDown') focusTopic(i + 1);
        else if (e.key === 'ArrowUp') focusTopic(i - 1);
        else if (e.key === 'Home') focusTopic(0);
        else if (e.key === 'End') focusTopic(groups.length - 1);
        else if (e.key === 'ArrowRight' || e.key === 'Enter') listEl?.querySelector<HTMLElement>('.script')?.focus();
        else return;
        e.preventDefault();
    }

    function listKey(e: KeyboardEvent): void {
        const rows = [...(listEl?.querySelectorAll<HTMLElement>('.script') ?? [])];
        const i = rows.indexOf(document.activeElement as HTMLElement);
        if (e.key === 'ArrowDown') rows[Math.min(i + 1, rows.length - 1)]?.focus();
        else if (e.key === 'ArrowUp') rows[Math.max(i - 1, 0)]?.focus();
        else if (e.key === 'ArrowLeft' && !searching) indexEl?.querySelector<HTMLElement>(`[data-topic="${selected}"]`)?.focus();
        else return;
        e.preventDefault();
    }

    // typing a letter anywhere in the library starts a search
    function typeToFilter(e: KeyboardEvent): void {
        if (e.target === filterInput || e.metaKey || e.ctrlKey || e.altKey || e.key.length !== 1 || e.key === ' ') return;
        filterInput?.focus();
    }

    function filterKey(e: KeyboardEvent): void {
        if (e.key === 'Escape' && filter.length > 0) {
            filter = '';
            e.stopPropagation();
        } else if (e.key === 'ArrowDown' || (e.key === 'Enter' && searching)) {
            const first = listEl?.querySelector<HTMLElement>('.script');
            if (e.key === 'Enter' && found[0] !== undefined) openScript(found[0].id);
            else first?.focus();
            e.preventDefault();
        }
    }
</script>

<!-- svelte-ignore a11y_no_static_element_interactions -->
<div class="library" onkeydown={typeToFilter}>
    <div class="search">
        <Icon name="search" size={16} />
        <input autocomplete="off" autocorrect="off" autocapitalize="off" spellcheck="false"
            bind:this={filterInput}
            bind:value={filter}
            onkeydown={filterKey}
            placeholder="Search every script: spring rate, TSAL, gear ratio, k_s…"
            aria-label="Search the library"
        />
        {#if searching}<span class="muted small">{found.length} found · Enter opens the first</span>{/if}
    </div>

    <div class="panes">
        <div class="index glass" role="listbox" aria-label="Topics" aria-orientation="vertical" bind:this={indexEl}>
            {#each groups as g, i (g.id)}
                {@const n = searching ? (matchCount.get(g.id) ?? 0) : g.scripts.length}
                <div
                    role="option"
                    tabindex={g.id === current?.id && !searching ? 0 : -1}
                    aria-selected={g.id === current?.id && !searching}
                    data-topic={g.id}
                    data-hue={g.hue}
                    class="topic-row"
                    class:on={g.id === current?.id && !searching}
                    class:dim={searching && n === 0}
                    onclick={() => pick(g.id)}
                    onkeydown={(e) => indexKey(e, i)}
                >
                    <span class="dot"></span>
                    <span class="name">{g.name}</span>
                    <span class="count mono">{n}</span>
                </div>
            {/each}
        </div>

        <section class="detail glass" aria-label={searching ? 'Search results' : current?.name}>
            <header>
                {#if searching}
                    <h2>Results for “{filter.trim()}”</h2>
                {:else if current !== undefined}
                    <h2>{current.name}</h2>
                    {@const blurb = catalog.topics.find((t) => t.id === current.id)?.blurb}
                    {#if blurb}<p class="muted small">{blurb}</p>{/if}
                {/if}
            </header>
            <!-- svelte-ignore a11y_no_noninteractive_element_interactions -->
            <ul bind:this={listEl} onkeydown={listKey}>
                {#each shown as s (s.id)}
                    {@const pinned = settings.pinned.includes(s.id)}
                    <li data-hue={s.topic?.hue ?? 'green'}>
                        <button type="button" class="script" onclick={() => openScript(s.id)}>
                            <span class="title">
                                {#if searching}<span class="dot"></span>{/if}{s.title}
                                {#if s.kind === 'tool'}<span class="chip chip-quiet">tool</span>{/if}
                            </span>
                            <span class="sub mono">{searching ? `${s.topic?.name ?? ''}${symbols(s) ? ' · ' + symbols(s) : ''}` : symbols(s)}</span>
                        </button>
                        <button type="button" class="pin" class:pinned onclick={() => togglePin(s.id)} aria-pressed={pinned} aria-label={pinned ? `Unpin ${s.title}` : `Pin ${s.title}`} title={pinned ? 'Unpin from the rail' : 'Pin to the rail'}>
                            <Icon name="pin" size={14} />
                        </button>
                    </li>
                {/each}
                {#if searching && found.length === 0}
                    <li class="empty muted">Nothing matches. Try another word, or a variable symbol like <span class="mono">k_s</span>.</li>
                {/if}
            </ul>
        </section>
    </div>
</div>

<style>
    .library {
        container-type: inline-size;
        display: flex;
        flex-direction: column;
        gap: var(--space-3);
        min-height: 0;
    }
    .search {
        display: flex;
        align-items: center;
        gap: var(--space-2);
        padding: 0 var(--space-3);
        border-radius: var(--r-lg);
        background: var(--well);
        border: 1px solid var(--well-edge);
        color: var(--muted);
    }
    .search:focus-within {
        border-color: var(--field-focus);
    }
    .search input {
        flex: 1;
        min-width: 0;
        padding: 12px 0;
        border: none;
        background: transparent;
        color: var(--text);
        font: inherit;
        outline: none;
    }
    .panes {
        display: grid;
        grid-template-columns: 248px 1fr;
        gap: var(--space-3);
        align-items: start;
    }
    @container (max-width: 560px) {
        .panes {
            grid-template-columns: 1fr;
        }
    }
    .index {
        display: flex;
        flex-direction: column;
        gap: 2px;
        padding: var(--space-2);
        border-radius: var(--r-xl);
        position: sticky;
        top: 0;
    }
    .topic-row {
        display: flex;
        align-items: center;
        gap: var(--space-2);
        height: 38px;
        padding: 0 var(--space-3);
        border-radius: var(--r-md);
        cursor: pointer;
        color: var(--text-2);
        font-size: var(--text-sm);
        position: relative;
    }
    .topic-row:hover {
        background: var(--hover);
    }
    .topic-row.on {
        background: var(--selected);
        color: var(--text);
        font-weight: 600;
    }
    .topic-row:focus-visible {
        outline: 2px solid var(--field-focus);
        outline-offset: 2px;
    }
    .topic-row.dim {
        opacity: 0.4;
    }
    .name {
        flex: 1;
    }
    .count {
        font-size: var(--text-xs);
        padding: 1px 8px;
        border-radius: var(--r-pill);
        background: var(--well);
        color: var(--muted);
    }
    .dot {
        width: 8px;
        height: 8px;
        border-radius: 50%;
        flex: none;
        display: inline-block;
        background: var(--hue-green);
    }
    [data-hue='teal'] .dot,
    [data-hue='teal'].topic-row .dot {
        background: var(--hue-teal);
    }
    [data-hue='gold'] .dot,
    [data-hue='gold'].topic-row .dot {
        background: var(--hue-gold);
    }
    .detail {
        padding: var(--space-4) var(--space-4) var(--space-3);
        border-radius: var(--r-xl);
        min-width: 0;
    }
    .detail header {
        padding: 0 var(--space-2) var(--space-3);
    }
    .detail h2 {
        font-size: var(--text-lg);
    }
    ul {
        list-style: none;
        margin: 0;
        padding: 0;
        columns: 300px;
        column-gap: var(--space-3);
    }
    li {
        break-inside: avoid;
        display: flex;
        align-items: stretch;
        border-radius: var(--r-md);
    }
    li:hover {
        background: var(--hover);
    }
    .script {
        appearance: none;
        flex: 1;
        min-width: 0;
        display: flex;
        flex-direction: column;
        gap: 2px;
        padding: 8px var(--space-2);
        border: none;
        border-radius: var(--r-md);
        background: transparent;
        color: var(--text);
        text-align: left;
        cursor: pointer;
        font: inherit;
        font-size: var(--text-sm);
    }
    .script:focus-visible {
        outline: 2px solid var(--field-focus);
        outline-offset: -2px;
    }
    .title {
        display: flex;
        align-items: center;
        gap: var(--space-2);
    }
    .sub {
        font-size: var(--text-xs);
        color: var(--muted);
        white-space: nowrap;
        overflow: hidden;
        text-overflow: ellipsis;
    }
    .pin {
        appearance: none;
        border: none;
        background: transparent;
        color: var(--muted);
        padding: 0 var(--space-2);
        border-radius: var(--r-md);
        cursor: pointer;
        opacity: 0;
    }
    li:hover .pin,
    .pin:focus-visible,
    .pin.pinned {
        opacity: 1;
    }
    .pin.pinned {
        color: var(--ink-accent);
    }
    .empty {
        padding: var(--space-3) var(--space-2);
        font-size: var(--text-sm);
    }
</style>
