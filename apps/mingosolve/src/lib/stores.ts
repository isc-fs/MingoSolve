// Views reachable from the rail. Solve is home; Session log, Help and Settings are pinned at the bottom.

export type ViewId = 'solve' | 'topics' | 'chain' | 'rules' | 'log' | 'help' | 'settings';

export interface ViewMeta {
    id: ViewId;
    label: string;
    description: string;
}

export const VIEWS: ViewMeta[] = [
    { id: 'solve', label: 'Solve', description: 'Paste a problem or open a script' },
    { id: 'topics', label: 'Topics', description: 'Browse every script by domain' },
    { id: 'chain', label: 'Chain', description: 'Link formulas to reach a value' },
    { id: 'rules', label: 'Rules', description: 'Search the official FS-Rules offline' },
    { id: 'log', label: 'Session log', description: 'Every copied answer, with timing; mock quiz and CSV export' },
    { id: 'help', label: 'Help', description: 'How to read answers, shortcuts, what to do if the app fails' },
    { id: 'settings', label: 'Settings', description: 'Look, rules year, answer format' },
];
