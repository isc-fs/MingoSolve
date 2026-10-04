<!--
    ISC MingoSolve: root layout. Loads persistent settings, checks for updates (best effort), and routes between
    views with the sidebar on the left and the calculator docked on the right. Cmd/Ctrl+K jumps to the Solve
    search from anywhere.
-->
<script lang="ts">
    import { onMount } from 'svelte';

    import { loadSettings, registerAutosaveEffect } from './lib/settings.svelte';
    import { session } from './lib/session.svelte';
    import { checkForUpdate, type AvailableUpdate } from './lib/updater';
    import type { ViewId } from './lib/stores';

    import Sidebar from './lib/Sidebar.svelte';
    import UpdateBanner from './lib/UpdateBanner.svelte';
    import CalcPanel from './lib/CalcPanel.svelte';
    import SolveView from './lib/SolveView.svelte';
    import ChainView from './lib/ChainView.svelte';
    import ToolsView from './lib/ToolsView.svelte';
    import ExamplesView from './lib/ExamplesView.svelte';
    import RulesView from './lib/RulesView.svelte';
    import SettingsView from './lib/SettingsView.svelte';

    let settingsReady = $state<boolean>(false);
    let update = $state<AvailableUpdate | null>(null);

    function selectView(id: ViewId): void {
        session.activeView = id;
    }

    function onKey(e: KeyboardEvent): void {
        if ((e.metaKey || e.ctrlKey) && e.key.toLowerCase() === 'k') {
            e.preventDefault();
            session.activeView = 'solve';
            session.focusSearch += 1;
        }
    }

    onMount(async () => {
        await loadSettings();
        settingsReady = true;
        checkForUpdate().then((u) => {
            update = u;
        });
    });

    registerAutosaveEffect();
</script>

<svelte:window onkeydown={onKey} />

<div class="app">
    <UpdateBanner {update} />

    <div class="shell">
        <Sidebar activeView={session.activeView} onSelect={selectView} />

        <main>
            {#if !settingsReady}
                <div class="loading">Loading settings…</div>
            {:else if session.activeView === 'solve'}
                <SolveView />
            {:else if session.activeView === 'chain'}
                <ChainView />
            {:else if session.activeView === 'tools'}
                <ToolsView />
            {:else if session.activeView === 'examples'}
                <ExamplesView />
            {:else if session.activeView === 'rules'}
                <RulesView />
            {:else if session.activeView === 'settings'}
                <SettingsView bind:update />
            {/if}
        </main>

        <CalcPanel />
    </div>
</div>

<style>
    .app {
        display: flex;
        flex-direction: column;
        height: 100vh;
        overflow: hidden;
    }

    .shell {
        display: flex;
        flex: 1;
        min-height: 0;
    }

    main {
        flex: 1;
        display: flex;
        flex-direction: column;
        min-width: 0;
        min-height: 0;
    }

    .loading {
        padding: var(--space-6);
        color: var(--text-muted);
    }
</style>
