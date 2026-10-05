<!--
    Help: how to use the app under time pressure, in short sections. Quick start, finding scripts, reading the
    answer, checking options, known-wrong keys, every keyboard shortcut (from shortcuts.ts, platform-aware), the
    session log, and what to do if the app fails. The printable version is docs/cheat-sheet.md. Same centred,
    column-flowing layout as Settings.
-->
<script lang="ts">
    import { SHORTCUTS, keysFor } from './shortcuts';
    import { session } from './session.svelte';
    import { platform } from './theme.svelte';

    const sections = [
        { id: 'help-quick', label: 'Quick start' },
        { id: 'help-finding', label: 'Finding scripts' },
        { id: 'help-reading', label: 'Reading the answer' },
        { id: 'help-options', label: 'Checking options' },
        { id: 'help-keys', label: 'Known-wrong keys' },
        { id: 'help-shortcuts', label: 'Shortcuts' },
        { id: 'help-log', label: 'Rehearsing' },
        { id: 'help-fail', label: 'If the app fails' },
    ];

    function jump(id: string): void {
        document.getElementById(id)?.scrollIntoView({ block: 'start' });
    }
</script>

<div class="view">
    <header>
        <h1>Help</h1>
        <p class="muted">
            Everything you need during a quiz, in one page. A printable version is in the repository:
            <span class="mono">docs/cheat-sheet.md</span>.
        </p>
        <nav aria-label="Help sections">
            {#each sections as s (s.id)}
                <button type="button" class="chip chip-quiet" onclick={() => jump(s.id)}>{s.label}</button>
            {/each}
        </nav>
    </header>

    <section class="glass panel" id="help-quick" aria-labelledby="h-quick">
        <h2 id="h-quick">Quick start</h2>
        <ol>
            <li><strong>Paste</strong> the question text anywhere in the window (<kbd>{platform === 'mac' ? '⌘V' : 'Ctrl+V'}</kbd>).</li>
            <li><strong>Open</strong> the best match: the first button under the problem. Its fields arrive filled in.</li>
            <li><strong>Check</strong> the values it took, then <strong>copy</strong> the answer with <kbd>{platform === 'mac' ? '⌘↵' : 'Ctrl+Enter'}</kbd> and paste it into the quiz.</li>
        </ol>
        <p class="muted">A blank field is an unknown: the script solves for whatever you leave empty.</p>
    </section>

    <section class="glass panel" id="help-finding" aria-labelledby="h-finding">
        <h2 id="h-finding">Finding scripts</h2>
        <ul>
            <li><strong>Paste a problem.</strong> Each match shows how many of the problem's values it filled in. The best match is first.</li>
            <li><strong>Search the library</strong> on the Solve page, or browse <strong>Topics</strong>. Type a word such as <span class="mono">spring</span>, or a variable such as <span class="mono">k_s</span>.</li>
            <li><strong>Press <kbd>{platform === 'mac' ? '⌘K' : 'Ctrl+K'}</kbd></strong> from anywhere and type <span class="mono">discharge</span> or <span class="mono">skidpad score</span>. A whole sentence there is searched as a problem.</li>
            <li>Every script lists the past questions it solves under <strong>Worked examples</strong>: one click loads the inputs.</li>
        </ul>
    </section>

    <section class="glass panel wide" id="help-reading" aria-labelledby="h-reading">
        <h2 id="h-reading">Reading the answer</h2>
        <dl>
            <dt>Several values in the list</dt>
            <dd>
                A quadratic has two roots. The big box shows one; the others appear as <span class="mono">Other roots: …</span>
                and as buttons under the fields. Pick the one the physics allows (a negative time or current is not an answer).
            </dd>
            <dt><span class="mono">Assumed g = 9.81.</span></dt>
            <dd>
                A constant you did not give was filled in with its default (g, air density…). If the problem says another
                value, type it in the field and the line disappears.
            </dd>
            <dt><span class="tag">from the problem</span></dt>
            <dd>
                The value was read from the pasted text; hover it to see which words. Typing in the field removes the tag.
                If the number is not what the problem says, fix it before copying.
            </dd>
            <dt><span class="tag">not used</span></dt>
            <dd>
                A chip under the problem: that value was found in the text but this script ignored it. A not-used value is
                a sign you opened the wrong script, or that the question has a detail you must add yourself.
            </dd>
            <dt><span class="tag">rules 2027</span> and <span class="mono">Other rules: …</span></dt>
            <dd>
                Scoring scripts use the rule year in Settings. The line shows what 2026 and legacy would answer. Old questions
                often keep an old key: if your options match another year, use that one.
            </dd>
            <dt><strong>These values contradict each other.</strong></dt>
            <dd>
                The values you gave cannot all be true together (for example a speed that does not fit the distance and time).
                The app lists the broken equations. Do not copy: re-read the problem for a wrong number or a wrong unit.
            </dd>
        </dl>
    </section>

    <section class="glass panel" id="help-options" aria-labelledby="h-options">
        <h2 id="h-options">Checking options</h2>
        <p>
            Open <strong>Check against options</strong> under the answer and paste the multiple-choice lines
            (<span class="mono">a) 73.4 A</span> per line). The closest option is marked with its distance in percent.
        </p>
        <ul>
            <li>Within 2 % counts as a match. Units are converted, so <span class="mono">0.5 kW</span> matches <span class="mono">500 W</span>.</li>
            <li><strong>Careful</strong> means two options are close, or one was skipped (other unit). Re-check the question.</li>
            <li><strong>Rule year</strong> says another year's rules fit an option much better than the selected one.</li>
        </ul>
    </section>

    <section class="glass panel" id="help-keys" aria-labelledby="h-keys">
        <h2 id="h-keys">Known-wrong keys</h2>
        <p>
            A few official answers disagree with the physics or with the rules (a key that leaves out a term, or a number no
            variant reproduces). If the app's answer matches <em>no</em> option, do not force it: choose by elimination.
        </p>
        <p class="muted">
            The list, with the question number and what the key did, is <span class="mono">data/known_keys.toml</span> in the
            repository. The app does not flag these questions on screen yet, so look the number up there.
        </p>
    </section>

    <section class="glass panel wide" id="help-shortcuts" aria-labelledby="h-shortcuts">
        <h2 id="h-shortcuts">Shortcuts</h2>
        <table>
            <caption class="sr-only">Keyboard shortcuts</caption>
            <thead><tr><th scope="col">Keys</th><th scope="col">Does</th><th scope="col">Where</th></tr></thead>
            <tbody>
                {#each SHORTCUTS as s (s.id)}
                    <tr data-shortcut={s.id}>
                        <td><kbd>{keysFor(s, platform)}</kbd></td>
                        <td>{s.does}</td>
                        <td class="muted">{s.where}</td>
                    </tr>
                {/each}
            </tbody>
        </table>
        <p class="muted">
            <strong>Undo clear</strong> is a button, not a key: after <strong>Clear</strong> it stays at the top of the sheet for
            8 seconds.
        </p>
    </section>

    <section class="glass panel" id="help-log" aria-labelledby="h-log">
        <h2 id="h-log">Rehearsing</h2>
        <p>
            Every copied answer is written to the <strong>Session log</strong> with the time since you pasted the problem.
            Type a label such as <span class="mono">Q7</span> in <strong>Question #</strong> at the top of the sheet to find it
            later. <strong>Start a mock quiz</strong> there resets the timer and tags the following entries; export the log to
            compare answers and times with a teammate.
        </p>
        <button type="button" class="btn" onclick={() => (session.activeView = 'log')}>Open the session log</button>
    </section>

    <section class="glass panel" id="help-fail" aria-labelledby="h-fail">
        <h2 id="h-fail">If the app fails</h2>
        <p>Do not troubleshoot during the quiz. Both fallbacks read the same data and give the same answers. From the repository folder:</p>
        <p class="label">Command line (needs Rust)</p>
        <pre class="mono">cargo run --release
cargo run --release -- speed s=75m t=3.5s @v_avg=km/h</pre>
        <p class="label">Python (needs uv)</p>
        <pre class="mono">cd legacy/python && uv sync && uv run fsq</pre>
        <p class="muted">Installing and updating: <span class="mono">docs/INSTALL.md</span>. Report problems afterwards with your system and what you saw.</p>
    </section>
</div>

<style>
    .view {
        flex: 1;
        min-height: 0;
        overflow-y: auto;
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
        display: flex;
        flex-direction: column;
        gap: var(--space-2);
    }
    nav {
        display: flex;
        flex-wrap: wrap;
        gap: var(--space-2);
    }
    nav .chip {
        cursor: pointer;
    }
    .panel {
        padding: var(--space-5);
        border-radius: var(--r-xl);
        display: flex;
        flex-direction: column;
        gap: var(--space-3);
        line-height: 1.55;
        scroll-margin-top: var(--space-3);
    }
    .panel.wide {
        grid-column: 1 / -1;
    }
    .panel > .btn {
        align-self: flex-start;
    }
    ol,
    ul {
        margin: 0;
        padding-left: var(--space-5);
        display: flex;
        flex-direction: column;
        gap: var(--space-2);
    }
    p {
        margin: 0;
    }
    dl {
        margin: 0;
        display: grid;
        grid-template-columns: minmax(150px, 0.34fr) 1fr;
        gap: var(--space-3) var(--space-5);
    }
    dt {
        font-weight: 600;
    }
    dd {
        margin: 0;
        color: var(--text-2);
    }
    @media (max-width: 760px) {
        dl {
            grid-template-columns: 1fr;
            gap: var(--space-1);
        }
        dd {
            margin-bottom: var(--space-3);
        }
    }
    .tag {
        display: inline-block;
        padding: 1px var(--space-2);
        border-radius: var(--r-pill);
        border: 1px solid var(--accent-edge);
        background: var(--accent-soft);
        font-size: var(--text-sm);
        font-weight: 500;
    }
    table {
        width: 100%;
        border-collapse: collapse;
    }
    th,
    td {
        text-align: left;
        padding: var(--space-2) var(--space-3);
        border-bottom: 1px solid var(--well-edge);
        vertical-align: top;
    }
    th {
        font-size: var(--text-sm);
        color: var(--muted);
        font-weight: 600;
    }
    td:first-child {
        white-space: nowrap;
    }
    pre {
        margin: 0;
        padding: var(--space-3) var(--space-4);
        border-radius: var(--r-md);
        background: var(--code);
        overflow-x: auto;
        font-size: var(--text-sm);
    }
</style>
