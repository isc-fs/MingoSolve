// Sidebar views, in the MingoCAN pattern: Solve pinned at the top (start here), Settings pinned at the bottom,
// the rest grouped in sections.

export type ViewId = 'solve' | 'chain' | 'tools' | 'examples' | 'rules' | 'settings';

export type NavSection = 'work' | 'reference';

export interface ViewMeta {
    id: ViewId;
    label: string;
    /** One-line plain-language description under the label and as the tooltip. */
    description: string;
    section?: NavSection;
}

export const VIEWS: ViewMeta[] = [
    { id: 'solve', label: 'Solve', description: 'Paste a question, solve a formula' },
    { id: 'chain', label: 'Chain', description: 'Link formulas to reach a value', section: 'work' },
    { id: 'tools', label: 'Tools', description: 'Scoring, rule tables, circuits', section: 'work' },
    { id: 'examples', label: 'Past questions', description: 'Solved FS-Quiz questions', section: 'reference' },
    { id: 'rules', label: 'Rules', description: 'Points, limits, 2027 changes', section: 'reference' },
    { id: 'settings', label: 'Settings', description: 'Rule year, answer format, about' },
];

export const NAV_SECTIONS: { section: NavSection; label: string }[] = [
    { section: 'work', label: 'Work' },
    { section: 'reference', label: 'Reference' },
];
