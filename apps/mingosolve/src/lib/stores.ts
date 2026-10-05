// Views reachable from the rail. Solve is home; Settings is pinned at the bottom.

export type ViewId = 'solve' | 'topics' | 'chain' | 'rules' | 'settings';

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
    { id: 'settings', label: 'Settings', description: 'Look, rules year, answer format' },
];
