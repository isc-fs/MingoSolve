// Thin invoke wrappers for src-tauri/src/tools.rs.
import { invoke } from '@tauri-apps/api/core';
import type { ToolInfo } from './types';

export function listTools(): Promise<ToolInfo[]> {
    return invoke('list_tools');
}

export function runTool(name: string, args: [string, string][]): Promise<string> {
    return invoke('run_tool', { name, args });
}
