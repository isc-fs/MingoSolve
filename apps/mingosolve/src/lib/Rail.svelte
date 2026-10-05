<!--
    Left rail: brand, search (⌘K opens the palette), the views, pinned scripts and recent ones, Session log, Help and
    Settings at the bottom. One click reaches any view or any pinned/recent script. On macOS the top strip sits under the
    translucent title bar and drags the window.
-->
<script lang="ts">
    import { onMount } from 'svelte';
    import { getVersion } from '@tauri-apps/api/app';

    import Icon from './Icon.svelte';
    import { VIEWS, type ViewId } from './stores';
    import { catalog, shortName } from './catalog.svelte';
    import { openScript, session } from './session.svelte';
    import { settings } from './settings.svelte';
    import { platform } from './theme.svelte';

    let version = $state('');
    onMount(async () => {
        try {
            version = await getVersion();
        } catch {
            version = '';
        }
    });

    const icons: Record<ViewId, 'solve' | 'topics' | 'chain' | 'log' | 'help' | 'settings'> = {
        solve: 'solve',
        topics: 'topics',
        chain: 'chain',
        log: 'log',
        help: 'help',
        settings: 'settings',
    };
    const bottomIds: ViewId[] = ['log', 'help', 'settings'];
    const main = VIEWS.filter((v) => !bottomIds.includes(v.id));
    const bottom = VIEWS.filter((v) => bottomIds.includes(v.id));
    const isMac = platform === 'mac';

    const recent = $derived(settings.recent.filter((r) => !settings.pinned.includes(r)).slice(0, 5));
    const title = (id: string): string => catalog.scripts.get(id)?.title ?? id;
</script>

<aside class="rail glass" class:mac={isMac}>
    {#if isMac}<div class="drag" data-tauri-drag-region></div>{/if}

    <div class="brand" data-tauri-drag-region>
        <span class="mark" aria-hidden="true"></span>
        <div>
            <strong>MingoSolve</strong>
            <span class="mono small muted">{version !== '' ? `v${version}` : 'ISC'} · rules {settings.rules}</span>
        </div>
    </div>

    <button type="button" class="search" onclick={() => (session.paletteOpen = true)}>
        <Icon name="search" size={16} />
        <span>Find a script</span>
        <kbd>{isMac ? '⌘K' : 'Ctrl K'}</kbd>
    </button>

    <nav aria-label="Views">
        {#each main as v (v.id)}
            <button
                type="button"
                class="item"
                class:on={session.activeView === v.id}
                aria-current={session.activeView === v.id ? 'page' : undefined}
                title={v.description}
                onclick={() => (session.activeView = v.id)}
            >
                <Icon name={icons[v.id]} />
                <span>{v.label}</span>
            </button>
        {/each}
    </nav>

    {#if settings.pinned.length > 0}
        <p class="label section">Pinned</p>
        <div class="list">
            {#each settings.pinned as id (id)}
                <button type="button" class="script" class:on={session.script?.id === id} title={title(id)} onclick={() => openScript(id)}>
                    <Icon name="pin" size={14} />
                    <span>{shortName(id)}</span>
                </button>
            {/each}
        </div>
    {/if}

    {#if recent.length > 0}
        <p class="label section">Recent</p>
        <div class="list">
            {#each recent as id (id)}
                <button type="button" class="script" class:on={session.script?.id === id} title={title(id)} onclick={() => openScript(id)}>
                    <Icon name="clock" size={14} />
                    <span>{shortName(id)}</span>
                </button>
            {/each}
        </div>
    {/if}

    <div class="spacer"></div>

    <nav aria-label="Help and settings">
        {#each bottom as v (v.id)}
            <button
                type="button"
                class="item"
                class:on={session.activeView === v.id}
                aria-current={session.activeView === v.id ? 'page' : undefined}
                title={v.id === 'help' ? `${v.description} (${isMac ? '⌘/' : 'Ctrl+/'})` : v.description}
                onclick={() => (session.activeView = v.id)}
            >
                <Icon name={icons[v.id]} />
                <span>{v.label}</span>
            </button>
        {/each}
    </nav>
</aside>

<style>
    /* full height and flush with the window edge, so the window's own corners are the rail's */
    .rail {
        width: 236px;
        flex: 0 0 236px;
        padding: var(--space-3);
        border-width: 0 1px 0 0;
        border-radius: 0;
        box-shadow: none;
        display: flex;
        flex-direction: column;
        gap: var(--space-1);
        min-height: 0;
        overflow-y: auto;
    }
    /* room for the window buttons (trafficLightPosition in tauri.macos.conf.json) */
    .rail.mac {
        padding-top: 48px;
    }
    .drag {
        position: absolute;
        inset: 0 0 auto 0;
        height: 48px;
    }
    .brand {
        display: flex;
        align-items: center;
        gap: var(--space-3);
        padding: var(--space-2) var(--space-2) var(--space-3);
    }
    /* the flat ISC mark, coloured by the theme (the glossy tile is only the Dock icon) */
    .mark {
        width: 34px;
        height: 34px;
        flex: none;
        background: var(--ink-accent);
        -webkit-mask: url('/mark.png') center / contain no-repeat;
        mask: url('/mark.png') center / contain no-repeat;
    }
    .brand strong {
        display: block;
        font-family: var(--font-display);
        font-weight: 600;
        font-size: 17px;
        letter-spacing: 0.01em;
    }
    .search {
        appearance: none;
        display: flex;
        align-items: center;
        gap: var(--space-2);
        height: 38px;
        margin-bottom: var(--space-2);
        padding: 0 var(--space-3);
        border-radius: var(--r-md);
        border: 1px solid var(--field-edge);
        background: var(--field);
        color: var(--muted);
        cursor: pointer;
        font-size: var(--text-sm);
        text-align: left;
    }
    .search span {
        flex: 1;
    }
    .search:hover {
        border-color: var(--accent-edge);
        color: var(--text-2);
    }
    nav,
    .list {
        display: flex;
        flex-direction: column;
        gap: 2px;
    }
    .item,
    .script {
        appearance: none;
        display: flex;
        align-items: center;
        gap: var(--space-3);
        padding: 9px var(--space-3);
        border: none;
        border-radius: var(--r-md);
        background: transparent;
        color: var(--text-2);
        cursor: pointer;
        text-align: left;
        font-size: var(--text-base);
        transition: background var(--motion);
    }
    .script {
        gap: var(--space-2);
        padding: 7px var(--space-3);
        font-size: var(--text-sm);
    }
    .script span {
        overflow: hidden;
        text-overflow: ellipsis;
        white-space: nowrap;
    }
    .item:hover,
    .script:hover {
        background: var(--hover);
        color: var(--text);
    }
    .item.on,
    .script.on {
        background: var(--selected);
        color: var(--text);
        font-weight: 600;
    }
    .item.on :global(svg) {
        color: var(--ink-accent);
    }
    .section {
        margin: var(--space-4) var(--space-3) var(--space-1);
    }
    .spacer {
        flex: 1;
        min-height: var(--space-4);
    }
</style>
