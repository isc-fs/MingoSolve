// Applies the look: Night glass or Paper glass on <html data-theme>, glass or solid surfaces, and whether the
// window itself is translucent (macOS vibrancy, Windows Mica: see tauri.macos/windows.conf.json; Linux is opaque).
import { getCurrentWindow } from '@tauri-apps/api/window';
import { settings } from './settings.svelte';

const dark = window.matchMedia('(prefers-color-scheme: dark)');
let systemDark = $state(dark.matches);
dark.addEventListener('change', (e) => (systemDark = e.matches));

export const platform: 'mac' | 'windows' | 'linux' = navigator.userAgent.includes('Mac')
    ? 'mac'
    : navigator.userAgent.includes('Windows')
      ? 'windows'
      : 'linux';

export function registerThemeEffect(): void {
    const root = document.documentElement;
    root.dataset.platform = platform;
    if (platform !== 'linux') root.dataset.native = 'vibrancy';
    $effect(() => {
        const resolved = settings.theme === 'system' ? (systemDark ? 'dark' : 'light') : settings.theme;
        root.dataset.theme = resolved;
        if (settings.solid) root.dataset.solid = '';
        else delete root.dataset.solid;
        // the native material follows the window theme; null = follow the OS
        void getCurrentWindow()
            .setTheme(settings.theme === 'system' ? null : settings.theme)
            .catch(() => {});
    });
}
