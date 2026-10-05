// The guided tour on its own: keyboard, focus, what it sets up and puts back, and where the card goes. The target
// elements are stand-ins with fixed rectangles (jsdom has no layout).
import { render, screen, waitFor } from '@testing-library/svelte';
import userEvent from '@testing-library/user-event';
import { afterEach, beforeEach, describe, expect, it } from 'vitest';

import Tour from './Tour.svelte';
import { SAMPLE_PROBLEM, STEPS, endTour, startTour, tour } from './onboarding.svelte';
import { session } from './session.svelte';
import { defaultSettings, settings } from './settings.svelte';

const rects: Record<string, [number, number, number, number]> = {
    problem: [100, 50, 400, 60],
    search: [12, 80, 212, 36],
};

beforeEach(() => {
    Object.assign(settings, defaultSettings());
    Object.assign(session, { activeView: 'help', script: null, problem: '', problemFills: {}, sheet: null });
    tour.active = false;
    Object.defineProperty(HTMLElement.prototype, 'offsetWidth', { configurable: true, get: () => 380 });
    Object.defineProperty(HTMLElement.prototype, 'offsetHeight', { configurable: true, get: () => 190 });
    for (const [name, [left, top, width, height]] of Object.entries(rects)) {
        const el = document.createElement('div');
        el.dataset.tour = name;
        el.getBoundingClientRect = () => ({ left, top, width, height, right: left + width, bottom: top + height, x: left, y: top, toJSON: () => ({}) });
        el.scrollIntoView = () => {};
        document.body.append(el);
    }
});

afterEach(() => {
    document.querySelectorAll('[data-tour]').forEach((e) => e.remove());
    delete (HTMLElement.prototype as { offsetWidth?: number }).offsetWidth;
    delete (HTMLElement.prototype as { offsetHeight?: number }).offsetHeight;
});

const dialog = () => screen.getByRole('dialog');
const counter = () => screen.getByText(/^Step \d+ of \d+$/).textContent;

async function open(): Promise<void> {
    startTour();
    render(Tour);
    await waitFor(() => expect(document.activeElement).toBe(dialog()));
}

describe('the card', () => {
    it('is a labelled modal dialog that takes focus, with a polite step announcement', async () => {
        await open();
        expect(dialog().getAttribute('aria-modal')).toBe('true');
        expect(document.getElementById(dialog().getAttribute('aria-labelledby')!)!.textContent).toBe(STEPS[0].title);
        expect(document.getElementById(dialog().getAttribute('aria-describedby')!)!.textContent).toBe(STEPS[0].body());
        expect(counter()).toBe(`Step 1 of ${STEPS.length}`);
        const live = screen.getByRole('status');
        expect(live.getAttribute('aria-live')).toBe('polite');
        expect(live.textContent).toBe(`Step 1 of ${STEPS.length}: ${STEPS[0].title}`);
        // the dimmed backdrop is decorative
        expect(document.querySelector('.dim')!.getAttribute('aria-hidden')).toBe('true');
    });

    it('highlights the target with a margin and keeps the card off it', async () => {
        await open();
        const hole = await waitFor(() => {
            const h = document.querySelector<HTMLElement>('.hole');
            expect(h).not.toBeNull();
            return h!;
        });
        const [l, t, w, h] = rects.problem;
        expect(hole.style.left).toBe(`${l - 8}px`);
        expect(hole.style.top).toBe(`${t - 8}px`);
        expect(hole.style.width).toBe(`${w + 16}px`);
        expect(hole.style.height).toBe(`${h + 16}px`);
        expect(dialog().classList.contains('centered')).toBe(false);
        await waitFor(() => expect(parseFloat(dialog().style.top)).toBeGreaterThanOrEqual(t + h + 8));
    });

    it("is centred, with no highlight, when the step's target is not on screen", async () => {
        await open();
        await userEvent.keyboard('{ArrowRight}{ArrowRight}'); // the answer slab does not exist here
        await waitFor(() => expect(counter()).toBe(`Step 3 of ${STEPS.length}`));
        await waitFor(() => expect(dialog().classList.contains('centered')).toBe(true));
        expect(document.querySelector('.hole')).toBeNull();
        expect(parseFloat(dialog().style.left)).toBe((window.innerWidth - 380) / 2);
        expect(parseFloat(dialog().style.top)).toBe((window.innerHeight - 190) / 2);
    });
});

describe('keyboard', () => {
    it('moves with the arrows and Enter, and Escape skips', async () => {
        await open();
        await userEvent.keyboard('{ArrowRight}');
        expect(counter()).toBe(`Step 2 of ${STEPS.length}`);
        await userEvent.keyboard('{Enter}');
        expect(counter()).toBe(`Step 3 of ${STEPS.length}`);
        await userEvent.keyboard('{ArrowLeft}');
        expect(counter()).toBe(`Step 2 of ${STEPS.length}`);
        await userEvent.keyboard('{Escape}');
        expect(tour.active).toBe(false);
        expect(settings.tourDone).toBe(true);
    });

    it('does not go before the first step, and Enter on a button presses it once', async () => {
        await open();
        await userEvent.keyboard('{ArrowLeft}');
        expect(counter()).toBe(`Step 1 of ${STEPS.length}`);
        screen.getByRole('button', { name: 'Next' }).focus();
        await userEvent.keyboard('{Enter}');
        expect(counter()).toBe(`Step 2 of ${STEPS.length}`);
    });

    it('Done on the last step finishes the tour', async () => {
        await open();
        for (let i = 1; i < STEPS.length; i++) await userEvent.keyboard('{ArrowRight}');
        expect(screen.queryByRole('button', { name: 'Skip tour' })).toBeNull();
        await userEvent.click(screen.getByRole('button', { name: 'Done' }));
        expect(tour.active).toBe(false);
        expect(settings.tourDone).toBe(true);
    });
});

describe('focus', () => {
    it('Tab and Shift+Tab cycle inside the card, and focus pulled outside comes back', async () => {
        await open();
        await userEvent.keyboard('{ArrowRight}'); // step 2: Skip, Back, Next
        const seen: string[] = [];
        for (let i = 0; i < 7; i++) {
            await userEvent.tab();
            seen.push((document.activeElement as HTMLElement).textContent!);
            expect(dialog().contains(document.activeElement)).toBe(true);
        }
        expect(new Set(seen)).toEqual(new Set(['Skip tour', 'Back', 'Next']));
        await userEvent.tab({ shift: true });
        expect(dialog().contains(document.activeElement)).toBe(true);
        const outside = document.createElement('button');
        document.body.append(outside);
        outside.focus();
        expect(document.activeElement).toBe(dialog());
        outside.remove();
    });

    it('keeps focus in the card when the button that had it disappears (Back on step 2 hides on step 1)', async () => {
        await open();
        await userEvent.keyboard('{ArrowRight}');
        await userEvent.click(screen.getByRole('button', { name: 'Back' }));
        await waitFor(() => expect(dialog().contains(document.activeElement)).toBe(true));
    });
});

describe('the sample problem', () => {
    it('is pasted when the user has none, and cleared when the tour ends untouched', async () => {
        await open();
        await waitFor(() => expect(session.problem).toBe(SAMPLE_PROBLEM));
        await waitFor(() => expect(document.getElementById('tour-body')!.textContent).toContain('pasted a sample'));
        endTour();
        expect(session.problem).toBe('');
    });

    it('is kept when the user edited it', async () => {
        await open();
        await waitFor(() => expect(session.problem).toBe(SAMPLE_PROBLEM));
        session.problem = `${SAMPLE_PROBLEM} Give your answer in km/h.`;
        endTour();
        expect(session.problem).toBe(`${SAMPLE_PROBLEM} Give your answer in km/h.`);
    });

    it("never overwrites the user's own problem", async () => {
        session.problem = 'my own question 5 kg';
        await open();
        await userEvent.keyboard('{ArrowRight}');
        expect(session.problem).toBe('my own question 5 kg');
        expect(document.getElementById('tour-body')!.textContent).not.toContain('pasted a sample');
        endTour();
        expect(session.problem).toBe('my own question 5 kg');
    });

    it('opens the best match for the answer steps, then closes it and restores Recent and the view', async () => {
        settings.recent = ['battery_load'];
        session.problemFills = { cornering_downforce: { values: [['m', '240 kg']], target: null } };
        await open();
        await userEvent.keyboard('{ArrowRight}{ArrowRight}');
        await waitFor(() => expect(session.script?.id).toBe('cornering_downforce'));
        expect(session.activeView).toBe('solve');
        endTour();
        expect(session.script).toBeNull();
        expect(settings.recent).toEqual(['battery_load']);
        expect(session.activeView).toBe('help'); // where the tour was started
    });
});
