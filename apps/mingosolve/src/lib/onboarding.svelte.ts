// The first-launch guided tour: which step is showing, the steps themselves, and what the tour sets up in the app
// (a sample problem and its best match) and puts back when it ends. Tour.svelte draws it.
import { platform, searchShortcut } from './platform';
import { openScript, session, type OpenScript } from './session.svelte';
import { settings } from './settings.svelte';
import type { ViewId } from './stores';

/** Our own invented skidpad problem (not question-bank text). */
export const SAMPLE_PROBLEM =
    'A Formula Student car of mass 240 kg with a lift coefficient times area of 3.2 m² drives the skidpad ' +
    '(centreline radius 9.125 m). With a tyre friction coefficient of 1.4 and air density 1.1 kg/m³, what is the ' +
    'maximum speed?';

/** `sampled`: the tour pasted the sample problem (the user had none). */
export const tour = $state({ active: false, step: 0, sampled: false });

export interface TourStep {
    id: string;
    title: string;
    body: () => string;
    /** CSS selectors of the elements to highlight (their union); none on screen means a centred card. */
    targets: string[];
    prepare?: () => Promise<void>;
}

const copyKeys = (): string => (platform === 'mac' ? '⌘↵' : 'Ctrl+Enter');

let openedNonce: number | null = null;
let startView: ViewId = 'solve';
let startRecent: string[] = [];
let startFocus: HTMLElement | null = null;

/** The control that had focus when the tour started (the card gives focus back to it). */
export const focusBeforeTour = (): HTMLElement | null => startFocus;

/** Poll until `ok()` or `ms` have passed. */
async function until(ok: () => boolean, ms: number): Promise<void> {
    const end = Date.now() + ms;
    while (!ok() && Date.now() < end) await new Promise((r) => setTimeout(r, 40));
}

async function showProblem(): Promise<void> {
    session.activeView = 'solve';
    if (session.problem.trim().length === 0) {
        session.problem = SAMPLE_PROBLEM;
        tour.sampled = true;
    }
}

async function showAnswer(): Promise<void> {
    session.activeView = 'solve';
    if (session.script !== null) return;
    await until(() => Object.keys(session.problemFills).length > 0, 3000);
    const best = Object.keys(session.problemFills)[0];
    if (best === undefined || !tour.active || session.script !== null) return;
    openScript(best);
    openedNonce = (session.script as OpenScript | null)?.nonce ?? null;
}

export const STEPS: TourStep[] = [
    {
        id: 'paste',
        title: 'Paste the problem',
        body: () =>
            `Paste the question text anywhere in the window (${platform === 'mac' ? '⌘V' : 'Ctrl+V'}). The app reads the values and units in it.` +
            (tour.sampled ? " We've pasted a sample so you can follow along." : ''),
        targets: ['[data-tour="problem"]'],
        prepare: showProblem,
    },
    {
        id: 'matches',
        title: 'Matches and values found',
        body: () =>
            'Each chip is a value the app found; the buttons are the scripts that fit, best first, with how many values each fills in. ' +
            'Once a script is open, its fields say "from the problem", and a chip marked "not used" means that script ignored the value.',
        targets: ['[data-tour="chips"]', '[data-tour="matches"]'],
        prepare: showProblem,
    },
    {
        id: 'answer',
        title: 'The answer',
        body: () =>
            `The big number is the answer. Copy it with the button or ${copyKeys()}, then paste it into the quiz. A "rules 2027" pill marks scoring scripts, and "Other roots" lists the other solutions of a quadratic.`,
        targets: ['[data-tour="answer"]'],
        prepare: showAnswer,
    },
    {
        id: 'check',
        title: 'Check against options',
        body: () =>
            'Paste the multiple-choice lines here and the closest option is marked, with units converted. A "Careful" note means two options are close.',
        targets: ['[data-tour="check"]'],
        prepare: showAnswer,
    },
    {
        id: 'find',
        title: 'Finding scripts',
        body: () =>
            `Press ${searchShortcut()} anywhere and type a word, a variable or a whole question. You can also browse Topics, or search the library on the Solve page.`,
        targets: ['[data-tour="search"]'],
    },
    {
        id: 'chain',
        title: 'Chain several formulas',
        body: () => 'When no single script gives the value, Chain links formulas for you. Common names such as v_i, v_f or μ are understood.',
        targets: ['[data-tour="nav-chain"]'],
    },
    {
        id: 'help',
        title: 'Help, rehearsals and this tour',
        body: () =>
            'Help has the shortcuts and what to do if something fails; replay this tour from there or from Settings. The Session log records every copied answer, for rehearsals.',
        targets: ['[data-tour="nav-help"]'],
    },
];

export function startTour(): void {
    if (tour.active) return;
    startView = session.activeView;
    startRecent = [...settings.recent];
    startFocus = document.activeElement instanceof HTMLElement ? document.activeElement : null;
    openedNonce = null;
    session.paletteOpen = false;
    tour.sampled = false;
    tour.step = 0;
    tour.active = true;
}

export function goToStep(i: number): void {
    tour.step = Math.min(Math.max(0, i), STEPS.length - 1);
}

/** End the tour (skipped, finished or failed): remember it was seen and put back what it set up. */
export function endTour(): void {
    settings.tourDone = true;
    tour.active = false;
    if (session.script !== null && session.script.nonce === openedNonce) {
        session.script = null;
        session.sheet = null;
        settings.recent = startRecent;
    }
    if (tour.sampled && session.problem === SAMPLE_PROBLEM) session.problem = '';
    tour.sampled = false;
    openedNonce = null;
    session.activeView = startView;
}
