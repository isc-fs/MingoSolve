// End-to-end journeys: the real UI (vite) against the real Rust command functions (test IPC bridge), in Chromium
// and WebKit (the engine macOS's WKWebView uses). Run: npm run e2e
import { defineConfig, devices } from '@playwright/test';

const BRIDGE = 'http://127.0.0.1:8799';

export default defineConfig({
    testDir: 'e2e',
    fullyParallel: true,
    retries: process.env.CI ? 1 : 0,
    reporter: [['list']],
    use: { baseURL: 'http://localhost:5175', viewport: { width: 1280, height: 820 }, trace: 'retain-on-failure' },
    projects: [
        { name: 'chromium', use: { ...devices['Desktop Chrome'], viewport: { width: 1280, height: 820 } } },
        { name: 'webkit', use: { ...devices['Desktop Safari'], viewport: { width: 1280, height: 820 } } },
    ],
    webServer: [
        {
            command: 'cargo run -q -p mingosolve --example ipc_bridge',
            env: { BRIDGE_TRACE: process.env.BRIDGE_TRACE ?? '' },
            cwd: '../..',
            url: `${BRIDGE}/health`,
            timeout: 600_000,
            reuseExistingServer: !process.env.CI,
        },
        {
            // the production bundle built in test mode: what ships, minus the bridge switch
            command: 'npx vite build --outDir dist-e2e && npx vite preview --outDir dist-e2e --port 5175 --strictPort',
            env: { VITE_IPC_BRIDGE: BRIDGE },
            url: 'http://localhost:5175',
            reuseExistingServer: !process.env.CI,
        },
    ],
});
