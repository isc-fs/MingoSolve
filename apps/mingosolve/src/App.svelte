<!--
    ISC MingoSolve: root layout. A lit, grainy ground behind three glass columns: the rail, the active view, and the
    calculator. Loads settings and the script catalogue, applies the theme, checks for updates on launch unless the setting is off (best effort).
    Global keys: Cmd/Ctrl+K opens the palette; Cmd/Ctrl+/ toggles Help; pasting text outside any field starts a problem in Solve.
-->
<script lang="ts">
    import { onMount } from 'svelte';

    import { loadSettings, registerAutosaveEffect, settings } from './lib/settings.svelte';
    import { platform } from './lib/platform';
    import { registerThemeEffect } from './lib/theme.svelte';
    import { loadCatalog } from './lib/catalog.svelte';
    import { session, toggleHelp } from './lib/session.svelte';
    import { loadSessionLog } from './lib/sessionlog.svelte';
    import { refreshRulebooks } from './lib/rulebook.svelte';
    import { checkForUpdate, type AvailableUpdate } from './lib/updater';

    import Rail from './lib/Rail.svelte';
    import Palette from './lib/Palette.svelte';
    import UpdateBanner from './lib/UpdateBanner.svelte';
    import CalcPanel from './lib/CalcPanel.svelte';
    import SolveView from './lib/SolveView.svelte';
    import TopicsView from './lib/TopicsView.svelte';
    import ChainView from './lib/ChainView.svelte';
    import RulesView from './lib/RulesView.svelte';
    import SettingsView from './lib/SettingsView.svelte';
    import HelpView from './lib/HelpView.svelte';
    import SessionLogView from './lib/SessionLogView.svelte';

    let ready = $state(false);
    let failed = $state<string | null>(null);
    let update = $state<AvailableUpdate | null>(null);

    onMount(async () => {
        await Promise.all([loadSettings(), loadSessionLog()]);
        try {
            await loadCatalog();
        } catch (e) {
            failed = String(e);
        }
        ready = true;
        void refreshRulebooks();
        if (settings.autoUpdateCheck) checkForUpdate().then((u) => (update = u));
    });

    registerAutosaveEffect();
    registerThemeEffect();

    function onKey(e: KeyboardEvent): void {
        if ((e.metaKey || e.ctrlKey) && e.key.toLowerCase() === 'k') {
            e.preventDefault();
            session.paletteOpen = !session.paletteOpen;
        } else if ((e.metaKey || e.ctrlKey) && e.key === '/') {
            e.preventDefault();
            session.paletteOpen = false;
            toggleHelp();
        }
    }

    function onPaste(e: ClipboardEvent): void {
        const t = e.target as HTMLElement | null;
        if (t !== null && (t.tagName === 'INPUT' || t.tagName === 'TEXTAREA' || t.isContentEditable)) return;
        const text = e.clipboardData?.getData('text') ?? '';
        if (text.trim().length === 0) return;
        e.preventDefault();
        session.problem = text;
        session.activeView = 'solve';
    }
</script>

<svelte:window onkeydown={onKey} onpaste={onPaste} />

<div class="ground" aria-hidden="true"></div>
{#if platform === 'mac'}<div class="titlebar" data-tauri-drag-region></div>{/if}

<div class="app">
    <div class="shell">
        <Rail />
        <main>
            <UpdateBanner {update} />
            {#if !ready}
                <p class="loading muted">Loading…</p>
            {:else if failed !== null}
                <p class="note note-bad loading"><strong>The engine didn't load.</strong> {failed}</p>
            {:else if session.activeView === 'solve'}
                <SolveView />
            {:else if session.activeView === 'topics'}
                <TopicsView />
            {:else if session.activeView === 'chain'}
                <ChainView />
            {:else if session.activeView === 'rules'}
                <RulesView />
            {:else if session.activeView === 'log'}
                <SessionLogView />
            {:else if session.activeView === 'help'}
                <HelpView />
            {:else if session.activeView === 'settings'}
                <SettingsView bind:update />
            {/if}
        </main>
        <CalcPanel />
    </div>
</div>

<Palette />

<style>
    .app {
        position: relative;
        z-index: 1;
        display: flex;
        flex-direction: column;
        height: 100vh;
    }
    .shell {
        flex: 1;
        display: flex;
        gap: 0;
        min-height: 0;
    }
    main {
        flex: 1;
        display: flex;
        flex-direction: column;
        min-width: 0;
        min-height: 0;
        padding-top: var(--space-3);
    }
    :global(:root[data-platform='mac']) main {
        padding-top: 28px;
    }
    .loading {
        margin: var(--space-6);
    }
    .titlebar {
        position: fixed;
        inset: 0 0 auto 0;
        height: 28px;
        z-index: 30;
    }
</style>
