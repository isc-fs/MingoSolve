<!--
    Settings: default rule year for scoring tools, how answers are copied (precision, decimal comma), updates and
    about (version, FS-Quiz ODbL attribution). Changes save automatically.
-->
<script lang="ts">
    import { onMount } from 'svelte';
    import { getVersion } from '@tauri-apps/api/app';

    import { settings } from './settings.svelte';
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
        checked = update === null ? 'You have the latest version (or the update server is unreachable).' : null;
    }
</script>

<section class="view">
    <header>
        <div>
            <h2>Settings</h2>
            <p>Saved automatically.</p>
        </div>
    </header>

    <div class="card stack">
        <div class="card-header"><h3>Rules</h3></div>
        <label class="field narrow">
            <span>Default rule set for scoring and rule tools</span>
            <select bind:value={settings.rules}>
                <option value="2027">2027 (FS-Rules 2027 v1.0)</option>
                <option value="2026">2026 (FS-Rules 2026 v1.1)</option>
                <option value="legacy">legacy (reproduces most past FS-Quiz keys)</option>
            </select>
        </label>
    </div>

    <div class="card stack">
        <div class="card-header"><h3>Answer format</h3></div>
        <div class="row">
            <label class="field">
                <span>Round to</span>
                <select bind:value={settings.precision.kind}>
                    <option value="sig">significant figures</option>
                    <option value="decimals">decimals</option>
                </select>
            </label>
            <label class="field">
                <span>How many</span>
                <input type="number" min="0" max="12" bind:value={settings.precision.n} />
            </label>
        </div>
        <label class="toggle">
            <input type="checkbox" bind:checked={settings.decimalComma} />
            Copy with a decimal comma (<span class="mono">77,9</span> instead of <span class="mono">77.9</span>)
        </label>
    </div>

    <div class="card stack">
        <div class="card-header"><h3>Updates</h3></div>
        <div class="row">
            <button type="button" class="btn" onclick={check} disabled={checking}>{checking ? 'Checking…' : 'Check for updates'}</button>
            {#if checked !== null}<span class="muted">{checked}</span>{/if}
        </div>
        <p class="muted small">Pin one version for quiz day: don't install updates during a quiz window.</p>
    </div>

    <div class="card stack">
        <div class="card-header"><h3>About</h3></div>
        <p>ISC MingoSolve {version !== '' ? `v${version}` : ''}, ISC Racing Team.</p>
        <p class="muted">
            Formulas and past questions are derived from the FS-Quiz database
            (<a href="https://fs-quiz.eu" target="_blank" rel="noreferrer">fs-quiz.eu</a>), available under the Open
            Database License (ODbL). Practice timed quizzes in MingoQuiz.
        </p>
    </div>
</section>

<style>
    .narrow {
        max-width: 420px;
    }
    .small {
        font-size: var(--text-xs);
    }
    a {
        color: var(--accent);
    }
</style>
