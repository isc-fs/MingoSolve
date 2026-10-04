<!--
    Rules: the constants the tools use, per rule set (2027, 2026, legacy keys), plus what changed in 2027. Values
    come from the same embedded data the engine scores with, so this page can't drift from the answers.
-->
<script lang="ts">
    import { onMount } from 'svelte';

    import Markdown from './Markdown.svelte';
    import { ruleSets, rulesChanges } from './rules';
    import { settings } from './settings.svelte';
    import type { RuleSet } from './types';

    let sets = $state<Record<string, RuleSet>>({});
    let changes = $state('');
    let year = $state<string>(settings.rules);

    onMount(async () => {
        sets = await ruleSets();
        changes = await rulesChanges();
    });

    const years = $derived(Object.keys(sets).filter((y) => y !== 'penalties').sort().reverse());
    const r = $derived((sets[year] ?? {}) as Record<string, unknown>);
    const pen = $derived((sets.penalties ?? {}) as Record<string, unknown>);

    const asTable = (v: unknown): Record<string, unknown> => (v ?? {}) as Record<string, unknown>;
    const fmt = (v: unknown): string => (typeof v === 'object' && v !== null ? JSON.stringify(v) : String(v));
</script>

<section class="view">
    <header>
        <div>
            <h2>Rules</h2>
            <p>
                Constants behind the scoring and rule tools. <strong>2027</strong> is FS-Rules 2027 v1.0;
                <strong>legacy</strong> reproduces most past FS-Quiz keys.
            </p>
        </div>
    </header>

    <div class="row">
        {#each years as y (y)}
            <button type="button" class="btn btn-sm" class:btn-primary={y === year} onclick={() => (year = y)}>{y}</button>
        {/each}
        <span class="muted small">{fmt(r.source ?? '')}</span>
    </div>

    <div class="grid">
        <div class="card card-tight">
            <div class="card-header"><h3>Maximum points</h3></div>
            <table>
                <tbody>
                    {#each Object.entries(asTable(r.points)) as [k, v] (k)}
                        <tr><td class="mono">{k}</td><td>{fmt(v)}</td></tr>
                    {/each}
                </tbody>
            </table>
        </div>

        {#if r.tmax_pmin !== undefined}
            <div class="card card-tight">
                <div class="card-header"><h3>Dynamic scoring (D 9.1.1)</h3></div>
                <table>
                    <thead><tr><th>event</th><th>Tmax / Tmin</th><th>Pmin / Pmax</th></tr></thead>
                    <tbody>
                        {#each Object.entries(asTable(r.tmax_pmin)) as [k, v] (k)}
                            <tr><td class="mono">{k}</td><td>{fmt(asTable(v).tmax)}</td><td>{fmt(asTable(v).pmin)}</td></tr>
                        {/each}
                    </tbody>
                </table>
            </div>
        {/if}

        {#if r.legacy_scoring !== undefined}
            <div class="card card-tight">
                <div class="card-header"><h3>Legacy scoring</h3></div>
                <table>
                    <thead><tr><th>event</th><th>Tmax</th><th>scale</th><th>floor</th><th>exp</th><th>div</th></tr></thead>
                    <tbody>
                        {#each Object.entries(asTable(r.legacy_scoring)) as [k, v] (k)}
                            {@const c = asTable(v)}
                            <tr>
                                <td class="mono">{k}</td><td>{fmt(c.tmax_factor)}</td><td>{fmt(c.scale)}</td>
                                <td>{fmt(c.floor)}</td><td>{fmt(c.exponent)}</td><td>{fmt(c.divisor)}</td>
                            </tr>
                        {/each}
                    </tbody>
                </table>
            </div>
        {/if}

        <div class="card card-tight">
            <div class="card-header"><h3>Statics, non-finalists</h3></div>
            <table>
                <tbody>
                    {#each Object.entries(asTable(r.static_nonfinalist)) as [k, v] (k)}
                        <tr><td class="mono">{k}</td><td>{fmt(v)}</td></tr>
                    {/each}
                </tbody>
            </table>
        </div>

        <div class="card card-tight">
            <div class="card-header"><h3>PCB TS–LV spacing [mm]</h3></div>
            <table>
                <thead>
                    <tr><th>≤ V</th>{#each (asTable(r.pcb_spacing).modes as string[]) ?? [] as m (m)}<th>{m}</th>{/each}</tr>
                </thead>
                <tbody>
                    {#each (asTable(r.pcb_spacing).rows as number[][]) ?? [] as row, i (i)}
                        <tr>{#each row as c, j (j)}<td>{c}</td>{/each}</tr>
                    {/each}
                </tbody>
            </table>
        </div>

        <div class="card card-tight">
            <div class="card-header"><h3>Penalties (all years)</h3></div>
            <table>
                <thead><tr><th>event</th><th>DOO</th><th>OC</th><th>post A</th><th>post B</th></tr></thead>
                <tbody>
                    {#each Object.entries(asTable(pen.doo_oc)) as [k, v] (k)}
                        <tr>
                            <td class="mono">{k}</td>
                            <td>{fmt(asTable(v).doo)} s</td>
                            <td>{asTable(v).oc === undefined ? 'DQ' : `${fmt(asTable(v).oc)} s`}</td>
                            <td>{fmt(asTable(asTable(pen.post_inspection).A)[k])} s</td>
                            <td>{fmt(asTable(asTable(pen.post_inspection).B)[k])} s</td>
                        </tr>
                    {/each}
                </tbody>
            </table>
            <p class="muted small">Flag {fmt(pen.flag)} s · out of order {fmt(pen.out_of_order)} s · weight ±{fmt(pen.weight_tolerance_kg)} kg then {fmt(pen.weight_per_kg)} pts/kg</p>
        </div>
    </div>

    <div class="card">
        <Markdown source={changes} />
    </div>
</section>

<style>
    .grid {
        display: grid;
        grid-template-columns: repeat(auto-fill, minmax(300px, 1fr));
        gap: var(--space-3);
        align-items: start;
    }
    table {
        border-collapse: collapse;
        width: 100%;
        font-size: var(--text-sm);
    }
    th,
    td {
        border-bottom: 1px solid var(--border);
        padding: var(--space-1) var(--space-2);
        text-align: left;
    }
    .small {
        font-size: var(--text-xs);
    }
</style>
