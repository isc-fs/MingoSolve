<!--
    Minimal markdown for the bundled docs (headings, tables, lists, paragraphs, `code`, **bold**, links).
    Renders through Svelte nodes, never {@html}, so doc text can't inject markup.
-->
<script lang="ts">
    type Inline = { kind: 'text' | 'code' | 'bold' | 'link'; text: string; href?: string };
    type Block =
        | { kind: 'h'; level: number; text: Inline[] }
        | { kind: 'p'; text: Inline[] }
        | { kind: 'ul'; items: Inline[][] }
        | { kind: 'table'; head: Inline[][]; rows: Inline[][][] };

    let { source }: { source: string } = $props();

    function inline(s: string): Inline[] {
        const out: Inline[] = [];
        const re = /`([^`]+)`|\*\*([^*]+)\*\*|\[([^\]]+)\]\(([^)]+)\)/g;
        let last = 0;
        for (const m of s.matchAll(re)) {
            if (m.index! > last) out.push({ kind: 'text', text: s.slice(last, m.index) });
            if (m[1] !== undefined) out.push({ kind: 'code', text: m[1] });
            else if (m[2] !== undefined) out.push({ kind: 'bold', text: m[2] });
            else out.push({ kind: 'link', text: m[3], href: m[4] });
            last = m.index! + m[0].length;
        }
        if (last < s.length) out.push({ kind: 'text', text: s.slice(last) });
        return out;
    }

    const cells = (l: string): string[] => l.trim().replace(/^\||\|$/g, '').split('|').map((c) => c.trim());

    const blocks = $derived.by(() => {
        const out: Block[] = [];
        const lines = source.split('\n');
        let i = 0;
        while (i < lines.length) {
            const l = lines[i];
            if (l.trim() === '') {
                i++;
            } else if (/^#{1,4} /.test(l)) {
                const level = l.indexOf(' ');
                out.push({ kind: 'h', level, text: inline(l.slice(level + 1)) });
                i++;
            } else if (l.trim().startsWith('|')) {
                const rows: string[] = [];
                while (i < lines.length && lines[i].trim().startsWith('|')) rows.push(lines[i++]);
                const body = rows.filter((r, k) => k !== 1 || !/^[\s|:-]+$/.test(r));
                out.push({ kind: 'table', head: cells(body[0]).map(inline), rows: body.slice(1).map((r) => cells(r).map(inline)) });
            } else if (/^\s*- /.test(l)) {
                const items: Inline[][] = [];
                while (i < lines.length && /^\s*- /.test(lines[i])) {
                    let item = lines[i++].replace(/^\s*- /, '');
                    while (i < lines.length && /^\s{2,}\S/.test(lines[i]) && !/^\s*- /.test(lines[i])) item += ' ' + lines[i++].trim();
                    items.push(inline(item));
                }
                out.push({ kind: 'ul', items });
            } else {
                let para = l;
                i++;
                while (i < lines.length && lines[i].trim() !== '' && !/^(#|\||\s*- )/.test(lines[i])) para += ' ' + lines[i++].trim();
                out.push({ kind: 'p', text: inline(para) });
            }
        }
        return out;
    });
</script>

{#snippet text(parts: Inline[])}
    {#each parts as p, k (k)}
        {#if p.kind === 'code'}<code class="mono">{p.text}</code>
        {:else if p.kind === 'bold'}<strong>{p.text}</strong>
        {:else if p.kind === 'link'}<a href={p.href} target="_blank" rel="noreferrer">{p.text}</a>
        {:else}{p.text}{/if}
    {/each}
{/snippet}

<div class="md">
    {#each blocks as b, k (k)}
        {#if b.kind === 'h'}
            {#if b.level <= 1}<h3>{@render text(b.text)}</h3>{:else}<h4>{@render text(b.text)}</h4>{/if}
        {:else if b.kind === 'p'}
            <p>{@render text(b.text)}</p>
        {:else if b.kind === 'ul'}
            <ul>{#each b.items as it, j (j)}<li>{@render text(it)}</li>{/each}</ul>
        {:else}
            <table>
                <thead><tr>{#each b.head as c, j (j)}<th>{@render text(c)}</th>{/each}</tr></thead>
                <tbody>
                    {#each b.rows as r, j (j)}<tr>{#each r as c, m (m)}<td>{@render text(c)}</td>{/each}</tr>{/each}
                </tbody>
            </table>
        {/if}
    {/each}
</div>

<style>
    .md {
        line-height: 1.55;
    }
    h3,
    h4 {
        margin: var(--space-4) 0 var(--space-2);
    }
    code {
        background: var(--code-bg);
        padding: 0 4px;
        border-radius: var(--radius-sm, 4px);
        font-size: var(--text-sm);
    }
    table {
        border-collapse: collapse;
        width: 100%;
        font-size: var(--text-sm);
        margin: var(--space-2) 0;
    }
    th,
    td {
        border: 1px solid var(--border);
        padding: var(--space-1) var(--space-2);
        text-align: left;
        vertical-align: top;
    }
    th {
        background: var(--bg-soft);
    }
    a {
        color: var(--accent);
    }
</style>
