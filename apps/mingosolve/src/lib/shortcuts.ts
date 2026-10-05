// Every keyboard shortcut in the app, one row each, for the Help view. Each is wired in the component named in
// `where`; shortcuts.test.ts dispatches the keys of the main ones.
export interface Shortcut {
    id: string;
    /** Key labels per platform (macOS, everywhere else). */
    mac: string;
    other: string;
    does: string;
    where: string;
}

export const SHORTCUTS: Shortcut[] = [
    { id: 'palette', mac: '⌘K', other: 'Ctrl+K', does: 'Find a script by name, topic or variable (again to close)', where: 'anywhere' },
    { id: 'help', mac: '⌘/', other: 'Ctrl+/', does: 'Open Help (again to go back)', where: 'anywhere' },
    { id: 'paste', mac: '⌘V', other: 'Ctrl+V', does: 'Paste a problem: scripts that fit open with its values filled in', where: 'anywhere outside a text field' },
    { id: 'copy', mac: '⌘↵', other: 'Ctrl+Enter', does: 'Copy the answer in quiz format (also from inside a field)', where: 'script sheet' },
    { id: 'run', mac: '↵', other: 'Enter', does: 'Run a tool script', where: 'tool fields' },
    { id: 'palette-move', mac: '↑ ↓', other: '↑ ↓', does: 'Move through the palette results', where: 'palette' },
    { id: 'palette-open', mac: '↵', other: 'Enter', does: 'Open the highlighted script (a long sentence searches it as a problem)', where: 'palette' },
    { id: 'palette-close', mac: 'Esc', other: 'Esc', does: 'Close the palette', where: 'palette' },
    { id: 'library-topics', mac: '↑ ↓  Home  End', other: '↑ ↓  Home  End', does: 'Move through the topics (the scripts follow)', where: 'library' },
    { id: 'library-enter', mac: '→ or ↵', other: '→ or Enter', does: 'Go into the list of scripts; ← comes back to the topics', where: 'library' },
    { id: 'library-search', mac: 'any letter', other: 'any letter', does: 'Start a search across all topics; Esc clears it, ↵ opens the first result', where: 'library' },
    { id: 'calc', mac: '↵', other: 'Enter', does: 'Evaluate the calculator expression', where: 'calculator' },
];

export const keysFor = (s: Shortcut, platform: 'mac' | 'windows' | 'linux'): string => (platform === 'mac' ? s.mac : s.other);
