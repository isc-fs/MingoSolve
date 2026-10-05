/// <reference types="svelte" />
/// <reference types="vite/client" />

interface ImportMetaEnv {
    /** Set only for end-to-end test builds: URL of the test IPC bridge. */
    readonly VITE_IPC_BRIDGE?: string;
}
