<!--
    Guided tour: dims the app, highlights the real element each step is about and shows a short card next to it
    (centred when the element is not on screen). Next / Back / Skip, Enter and the arrows move, Esc skips. The
    card is a modal dialog that traps focus and gives it back to where it was; the dimming is decorative. It never
    blocks the app: any failure ends the tour.
-->
<script lang="ts">
    import { onMount, tick, untrack } from 'svelte';

    import { STEPS, endTour, focusBeforeTour, goToStep, tour } from './onboarding.svelte';
    import { inflate, placeCard, type Box } from './tourLayout';

    const PAD = 6;
    const step = $derived(STEPS[tour.step]);
    const last = $derived(tour.step === STEPS.length - 1);

    let card = $state<HTMLElement | null>(null);
    let spot = $state<Box | null>(null);
    let pos = $state({ left: 0, top: 0 });
    let returnTo: HTMLElement | null = null;
    let returnId = '';
    let scrolled = -1;
    let retried = -1;
    let token = 0;

    function fail(): void {
        endTour();
    }

    function visibleTargets(): Element[] {
        return step.targets.flatMap((sel) => [...document.querySelectorAll(sel)]).filter((el) => {
            const r = el.getBoundingClientRect();
            return r.width > 0 && r.height > 0;
        });
    }

    function union(els: Element[]): Box {
        const rs = els.map((e) => e.getBoundingClientRect());
        const left = Math.min(...rs.map((r) => r.left));
        const top = Math.min(...rs.map((r) => r.top));
        return {
            left,
            top,
            width: Math.max(...rs.map((r) => r.right)) - left,
            height: Math.max(...rs.map((r) => r.bottom)) - top,
        };
    }

    function measure(): void {
        try {
            if (card === null) return;
            let els = visibleTargets();
            if (els.length > 0 && scrolled !== tour.step) {
                scrolled = tour.step;
                els[0].scrollIntoView({ block: 'nearest', inline: 'nearest' });
                els = visibleTargets();
            }
            const vp = { width: window.innerWidth, height: window.innerHeight };
            const box = els.length > 0 ? inflate(union(els), PAD, vp.width, vp.height) : null;
            const size = { width: card.offsetWidth, height: card.offsetHeight };
            let placed = placeCard(box, size, vp);
            if (placed.overlaps && els.length > 0 && retried !== tour.step) {
                // no free side: bring the target to the top of its scroller so the card fits below it
                retried = tour.step;
                els[0].scrollIntoView({ block: 'start', inline: 'nearest' });
                els = visibleTargets();
                const again = els.length > 0 ? inflate(union(els), PAD, vp.width, vp.height) : null;
                placed = placeCard(again, size, vp);
                spot = again;
            } else {
                spot = box;
            }
            pos = { left: placed.left, top: placed.top };
        } catch {
            fail();
        }
    }

    // each step: set up what it shows (navigate, paste the sample, open the best match), then keep the highlight on it
    $effect(() => {
        const i = tour.step;
        const mine = ++token;
        spot = null;
        void (async () => {
            try {
                await untrack(() => STEPS[i].prepare?.());
                if (mine !== token) return;
                await tick();
                measure();
                if (!card?.contains(document.activeElement)) card?.focus();
            } catch {
                fail();
            }
        })();
    });

    onMount(() => {
        returnTo = focusBeforeTour();
        returnId = returnTo?.id ?? '';
        card?.focus();
        const timer = setInterval(measure, 120);
        window.addEventListener('resize', measure);
        document.addEventListener('scroll', measure, true);
        return () => {
            clearInterval(timer);
            window.removeEventListener('resize', measure);
            document.removeEventListener('scroll', measure, true);
            // the tour may have switched views, so the view it started from is drawn again: look the control up by id
            void tick().then(() => {
                const back = returnTo?.isConnected ? returnTo : returnId !== '' ? document.getElementById(returnId) : null;
                (back ?? document.querySelector<HTMLElement>('main'))?.focus();
            });
        };
    });

    function next(): void {
        if (last) endTour();
        else goToStep(tour.step + 1);
    }

    function focusables(): HTMLElement[] {
        return card === null ? [] : [...card.querySelectorAll<HTMLElement>('button:not([disabled])')];
    }

    function onKey(e: KeyboardEvent): void {
        if (e.metaKey || e.ctrlKey || e.altKey) return;
        const onButton = e.target instanceof HTMLElement && e.target.tagName === 'BUTTON';
        if (e.key === 'Escape') {
            endTour();
        } else if (e.key === 'ArrowRight' || (e.key === 'Enter' && !onButton)) {
            next();
        } else if (e.key === 'ArrowLeft') {
            goToStep(tour.step - 1);
        } else if (e.key === 'Tab') {
            const f = focusables();
            if (f.length === 0) return;
            const i = f.indexOf(document.activeElement as HTMLElement);
            const to = e.shiftKey ? (i <= 0 ? f[f.length - 1] : f[i - 1]) : i === -1 || i === f.length - 1 ? f[0] : f[i + 1];
            to.focus();
        } else {
            return;
        }
        e.preventDefault();
        e.stopPropagation();
    }

    function onFocusIn(e: FocusEvent): void {
        if (card !== null && e.target instanceof Node && !card.contains(e.target)) card.focus();
    }
</script>

<svelte:window onkeydowncapture={onKey} onfocusin={onFocusIn} />

<div class="tour">
    <div class="dim" class:none={spot === null} aria-hidden="true">
        {#if spot !== null}
            <div class="hole" style:left="{spot.left}px" style:top="{spot.top}px" style:width="{spot.width}px" style:height="{spot.height}px"></div>
        {/if}
    </div>

    <div
        bind:this={card}
        class="card glass"
        class:centered={spot === null}
        role="dialog"
        aria-modal="true"
        aria-labelledby="tour-title"
        aria-describedby="tour-body"
        tabindex="-1"
        data-tour-card
        style:left="{pos.left}px"
        style:top="{pos.top}px"
    >
        <p class="label" data-tour-count>Step {tour.step + 1} of {STEPS.length}</p>
        <h2 id="tour-title">{step.title}</h2>
        <p id="tour-body">{step.body()}</p>
        <div class="actions">
            {#if !last}<button type="button" class="btn btn-ghost" onclick={endTour}>Skip tour</button>{/if}
            <span class="spacer"></span>
            {#if tour.step > 0}<button type="button" class="btn" onclick={() => goToStep(tour.step - 1)}>Back</button>{/if}
            <button type="button" class="btn btn-primary" onclick={next}>{last ? 'Done' : 'Next'}</button>
        </div>
    </div>

    <div class="sr-only" role="status" aria-live="polite">Step {tour.step + 1} of {STEPS.length}: {step.title}</div>
</div>

<style>
    .tour {
        position: fixed;
        inset: 0;
        z-index: 100;
    }
    .dim {
        position: absolute;
        inset: 0;
    }
    .dim.none {
        background: rgba(4, 12, 8, 0.62);
    }
    .hole {
        position: absolute;
        border-radius: var(--r-md);
        box-shadow: 0 0 0 100vmax rgba(4, 12, 8, 0.62), 0 0 0 2px #fff;
        transition: left var(--motion), top var(--motion), width var(--motion), height var(--motion);
    }
    .card {
        position: absolute;
        width: min(380px, calc(100vw - 24px));
        padding: var(--space-4) var(--space-5);
        border-radius: var(--r-xl);
        display: flex;
        flex-direction: column;
        gap: var(--space-2);
        line-height: 1.5;
        background: color-mix(in srgb, var(--glass-solid) 92%, transparent);
        outline: none;
        transition: left var(--motion), top var(--motion);
    }
    :global(:root[data-solid]) .card {
        background: var(--glass-solid);
    }
    @media (prefers-reduced-transparency: reduce) {
        .card {
            background: var(--glass-solid);
        }
    }
    .card:focus-visible {
        box-shadow: 0 0 0 3px var(--field-focus), var(--shadow);
    }
    .actions {
        display: flex;
        align-items: center;
        gap: var(--space-2);
        margin-top: var(--space-2);
    }
    .spacer {
        flex: 1;
    }
    @media (prefers-reduced-motion: reduce) {
        .hole,
        .card {
            transition: none;
        }
    }
</style>
