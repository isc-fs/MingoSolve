// Which desktop the app runs on, and the keyboard shortcut texts that follow from it.
export type Platform = 'mac' | 'windows' | 'linux';

export function platformOf(userAgent: string): Platform {
    return userAgent.includes('Mac') ? 'mac' : userAgent.includes('Windows') ? 'windows' : 'linux';
}

export const platform: Platform = platformOf(navigator.userAgent);

/** The shortcut that opens the script search: ⌘K on macOS, Ctrl+K elsewhere. */
export function searchShortcut(p: Platform = platform): string {
    return p === 'mac' ? '⌘K' : 'Ctrl+K';
}
