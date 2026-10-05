// Platform detection from real user-agent strings, and the shortcut text for each.
import { describe, expect, it } from 'vitest';

import { platformOf, searchShortcut } from './platform';

const MAC = 'Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/605.1.15 (KHTML, like Gecko)';
const WINDOWS = 'Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/124.0 Safari/537.36';
const LINUX = 'Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/124.0 Safari/537.36';

describe('platform', () => {
    it('is recognised from the user agent the webview reports', () => {
        expect(platformOf(MAC)).toBe('mac');
        expect(platformOf(WINDOWS)).toBe('windows');
        expect(platformOf(LINUX)).toBe('linux');
    });

    it('shows ⌘K on macOS and Ctrl K on Windows and Linux', () => {
        expect(searchShortcut('mac')).toBe('⌘K');
        expect(searchShortcut('windows')).toBe('Ctrl+K');
        expect(searchShortcut('linux')).toBe('Ctrl+K');
    });
});
