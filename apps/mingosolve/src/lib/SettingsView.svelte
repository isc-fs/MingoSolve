<!--
    Settings: look (style, dark or light, glass or solid), default rule year for scoring scripts, how answers are copied, updates
    and about. Everything saves automatically.
-->
<script lang="ts">
    import { onMount } from 'svelte';
    import { getVersion } from '@tauri-apps/api/app';

    import { platform, searchShortcut } from './platform';
    import { radioArrows } from './radio';
    import { settings, TEXT_SIZES, type TextSize, type Theme } from './settings.svelte';
    import { startTour } from './onboarding.svelte';
    import { STYLES } from './styles';
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

    const zoomKeys = platform === 'mac' ? '⌘+ and ⌘−' : 'Ctrl+ and Ctrl−';

    const current = $derived(STYLES.find((s) => s.id === settings.style) ?? STYLES[0]);
    const themes: { id: Theme; label: string; hint: string }[] = $derived([
        { id: 'system', label: 'Follow the OS', hint: 'Dark at night, light by day' },
        { id: 'dark', label: 'Dark', hint: current.dark },
        { id: 'light', label: 'Light', hint: current.light },
    ]);
</script>

<div class="view">
    <header>
        <h1>Settings</h1>
        <p class="muted">Saved automatically.</p>
    </header>

    <section class="glass panel look">
        <h2>Look</h2>
        <div class="styles" role="radiogroup" aria-label="Style" use:radioArrows>
            {#each STYLES as s (s.id)}
                <button type="button" role="radio" class="style" tabindex={settings.style === s.id ? 0 : -1} aria-checked={settings.style === s.id} class:on={settings.style === s.id} onclick={() => (settings.style = s.id)}>
                    <span class="pair" aria-hidden="true">
                        {#each ['light', 'dark'] as mode (mode)}
                            <span class="pv" data-style={s.id} data-theme={mode}>
                                <span class="pv-pane"><span class="pv-h">Skidpad</span><span class="pv-line"></span><span class="pv-line short"></span></span>
                                <span class="pv-slab"><span class="pv-val">4.92 s</span><span class="pv-copy">Copy</span></span>
                            </span>
                        {/each}
                    </span>
                    <span class="style-name"><strong>{s.name}</strong><span class="muted small">{s.light} · {s.dark}</span></span>
                    <span class="small style-about">{s.about}</span>
                </button>
            {/each}
        </div>
        <div class="seg" role="radiogroup" aria-label="Theme" use:radioArrows>
            {#each themes as t (t.id)}
                <button type="button" role="radio" tabindex={settings.theme === t.id ? 0 : -1} aria-checked={settings.theme === t.id} class:on={settings.theme === t.id} onclick={() => (settings.theme = t.id)}>
                    <strong>{t.label}</strong><span class="muted small">{t.hint}</span>
                </button>
            {/each}
        </div>
        <p class="label" id="text-size-label">Text size</p>
        <div class="seg four" role="radiogroup" aria-labelledby="text-size-label" use:radioArrows>
            {#each TEXT_SIZES as n (n)}
                <button type="button" role="radio" tabindex={settings.textSize === n ? 0 : -1} aria-checked={settings.textSize === n} class:on={settings.textSize === n} onclick={() => (settings.textSize = n as TextSize)}>
                    <strong>{n} %</strong>
                </button>
            {/each}
        </div>
        <p class="muted small">{zoomKeys} also zoom the whole window.</p>
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
        <div class="row">
            <button type="button" class="btn" id="take-tour" onclick={startTour}>Take the tour</button>
            <span class="muted small">A short walk through the app, on the real screen.</span>
        </div>
        <p>ISC MingoSolve {version !== '' ? `v${version}` : ''} · ISC Racing Team</p>
        <p class="muted small">
            Scripts are checked against past FS-Quiz questions (fs-quiz.eu, Open Database License). Shortcuts:
            <kbd>{searchShortcut()}</kbd> find a script, paste anywhere to start from a problem, <kbd>⌘↵</kbd> copy the answer (Ctrl+Enter on Windows and Linux).
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
    .look {
        grid-column: 1 / -1;
    }
    .styles {
        display: grid;
        grid-template-columns: repeat(auto-fill, minmax(220px, 1fr));
        gap: var(--space-3);
        margin-bottom: var(--space-2);
    }
    .style {
        appearance: none;
        display: flex;
        flex-direction: column;
        gap: var(--space-2);
        padding: var(--space-2) var(--space-2) var(--space-3);
        border-radius: var(--r-lg);
        border: 1px solid transparent;
        background: transparent;
        cursor: pointer;
        text-align: left;
        transition:
            border-color var(--motion),
            background var(--motion);
    }
    .style:hover {
        background: var(--hover);
    }
    .style.on {
        border-color: var(--accent);
        box-shadow: 0 0 0 3px var(--accent-soft);
    }
    .style-name {
        display: flex;
        align-items: baseline;
        justify-content: space-between;
        gap: var(--space-2);
        padding: 0 var(--space-1);
    }
    .style-about {
        padding: 0 var(--space-1);
        color: var(--text-2);
        line-height: 1.4;
    }
    /* a live sample of the style in each mode: its own tokens apply under the nested data-style / data-theme */
    .pair {
        display: grid;
        grid-template-columns: 1fr 1fr;
        border-radius: calc(var(--r-lg) - 4px);
        overflow: hidden;
    }
    .pv {
        position: relative;
        display: flex;
        flex-direction: column;
        justify-content: space-between;
        gap: 6px;
        height: 104px;
        padding: 8px;
        color: var(--text);
        background: var(--glow), var(--ground);
        font-family: var(--font-sans);
    }
    .pv-pane {
        display: flex;
        flex-direction: column;
        gap: 4px;
        padding: 6px 7px;
        border: 1px solid var(--glass-edge);
        border-radius: min(var(--r-lg), 8px);
        background: var(--glass);
    }
    .pv-h {
        font-family: var(--font-display);
        font-weight: var(--display-weight);
        letter-spacing: var(--display-tracking);
        text-transform: var(--display-case);
        color: var(--heading);
        font-size: 0.6875rem;
        line-height: 1.2;
    }
    .pv-line {
        height: 3px;
        border-radius: 2px;
        background: var(--text-2);
        opacity: 0.35;
    }
    .pv-line.short {
        width: 55%;
        background: var(--ink-accent);
        opacity: 0.8;
    }
    .pv-slab {
        display: flex;
        align-items: center;
        justify-content: space-between;
        gap: 4px;
        padding: 5px 6px;
        border: var(--slab-border);
        border-radius: min(var(--slab-radius), 8px);
        background: var(--answer-bg), var(--glass-solid);
    }
    .pv-val {
        font-family: var(--answer-font);
        font-weight: var(--answer-weight);
        font-variant-numeric: var(--answer-numeric);
        color: var(--value-text);
        background: var(--value-bg);
        padding: 1px 3px;
        border-radius: 2px;
        font-size: 0.875rem;
        line-height: 1;
        white-space: nowrap;
    }
    .pv-copy {
        padding: 2px 6px;
        border-radius: min(var(--r-pill), 999px);
        background: var(--copy-bg);
        color: var(--copy-text);
        font-size: 0.625rem;
        font-weight: 600;
    }
    .seg {
        display: grid;
        grid-template-columns: repeat(3, 1fr);
        gap: var(--space-2);
    }
    .seg.four {
        grid-template-columns: repeat(4, 1fr);
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
