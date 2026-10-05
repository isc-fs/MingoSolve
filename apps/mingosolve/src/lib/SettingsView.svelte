<!--
    Settings: look (theme, glass or solid), default rule year for scoring scripts, how answers are copied, updates
    and about. Everything saves automatically.
-->
<script lang="ts">
    import { onMount } from 'svelte';
    import { getVersion } from '@tauri-apps/api/app';

    import { settings, type Theme } from './settings.svelte';
    import { checkForUpdate, type AvailableUpdate } from './updater';

    let { update = $bindable() }: { update: AvailableUpdate | null } = $props();

    let version = $state('');
    let checking = $state(false);
    let checked = $state<string | null>(null);

    onMount(async () => {
        try {
            version = await getVersion();
        } catch {
            version = '';
        }
    });

    async function check(): Promise<void> {
        checking = true;
        update = await checkForUpdate();
        checking = false;
        checked = update === null ? 'You have the latest version, or the update server is unreachable.' : null;
    }

    const themes: { id: Theme; label: string; hint: string }[] = [
        { id: 'system', label: 'Follow the OS', hint: 'Dark at night, light by day' },
        { id: 'dark', label: 'Night glass', hint: 'Dark' },
        { id: 'light', label: 'Paper glass', hint: 'Light' },
    ];
</script>

<div class="view">
    <header>
        <h1>Settings</h1>
        <p class="muted">Saved automatically.</p>
    </header>

    <section class="glass panel">
        <h2>Look</h2>
        <div class="seg" role="radiogroup" aria-label="Theme">
            {#each themes as t (t.id)}
                <button type="button" role="radio" aria-checked={settings.theme === t.id} class:on={settings.theme === t.id} onclick={() => (settings.theme = t.id)}>
                    <strong>{t.label}</strong><span class="muted small">{t.hint}</span>
                </button>
            {/each}
        </div>
        <label class="toggle">
            <input type="checkbox" bind:checked={settings.solid} />
            <span>Solid surfaces instead of glass <span class="muted">(easier to read on busy desktops; also follows the OS "reduce transparency" setting)</span></span>
        </label>
    </section>

    <section class="glass panel">
        <h2>Rules</h2>
        <label class="field">
            <span class="label">Default rule set for scoring scripts</span>
            <select class="input" bind:value={settings.rules}>
                <option value="2027">2027 (FS-Rules 2027 v1.0)</option>
                <option value="2026">2026 (FS-Rules 2026 v1.1)</option>
                <option value="legacy">Legacy (rules up to 2025)</option>
            </select>
        </label>
    </section>

    <section class="glass panel">
        <h2>Copied answers</h2>
        <div class="row">
            <label class="field">
                <span class="label">Round to</span>
                <select class="input" bind:value={settings.precision.kind}>
                    <option value="sig">significant figures</option>
                    <option value="decimals">decimals</option>
                </select>
            </label>
            <label class="field narrow">
                <span class="label">How many</span>
                <input autocomplete="off" autocorrect="off" autocapitalize="off" spellcheck="false" class="input" type="number" min="0" max="12" bind:value={settings.precision.n} />
            </label>
        </div>
        <label class="toggle">
            <input type="checkbox" bind:checked={settings.decimalComma} />
            <span>Decimal comma (<span class="mono">77,9</span> instead of <span class="mono">77.9</span>)</span>
        </label>
    </section>

    <section class="glass panel">
        <h2>Updates</h2>
        <label class="toggle">
            <input type="checkbox" bind:checked={settings.autoUpdateCheck} />
            <span>Check for updates automatically when the app starts</span>
        </label>
        <div class="row">
            <button type="button" class="btn" onclick={check} disabled={checking}>{checking ? 'Checking…' : 'Check for updates'}</button>
            {#if checked !== null}<span class="muted small">{checked}</span>{/if}
        </div>
    </section>

    <section class="glass panel">
        <h2>About</h2>
        <p>ISC MingoSolve {version !== '' ? `v${version}` : ''} · ISC Racing Team</p>
        <p class="muted small">
            Scripts are checked against past FS-Quiz questions (fs-quiz.eu, Open Database License). Shortcuts:
            <kbd>⌘K</kbd> find a script, paste anywhere to start from a problem, <kbd>Esc</kbd> clears a script.
        </p>
    </section>
</div>

<style>
    .view {
        flex: 1;
        min-height: 0;
        overflow-y: auto;
        /* panels flow into columns on wide windows; content stays centred and at most 1400 px wide */
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
    }
    .seg {
        display: grid;
        grid-template-columns: repeat(3, 1fr);
        gap: var(--space-2);
    }
    .seg button {
        appearance: none;
        display: flex;
        flex-direction: column;
        gap: 2px;
        padding: var(--space-3);
        border-radius: var(--r-md);
        border: 1px solid var(--well-edge);
        background: var(--well);
        cursor: pointer;
        text-align: left;
    }
    .seg button.on {
        border-color: var(--accent);
        box-shadow: 0 0 0 3px var(--accent-soft);
    }
    .toggle {
        display: flex;
        gap: var(--space-2);
        align-items: flex-start;
        cursor: pointer;
        line-height: 1.5;
    }
    .toggle input {
        accent-color: var(--accent);
        margin-top: 4px;
    }
    .row {
        display: flex;
        gap: var(--space-3);
        align-items: flex-end;
    }
    .field {
        display: flex;
        flex-direction: column;
        gap: 6px;
        max-width: 420px;
    }
    .narrow {
        width: 110px;
    }
</style>
