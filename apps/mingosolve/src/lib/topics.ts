// Thin invoke wrappers for src-tauri/src/topics.rs.
import { invoke } from '@tauri-apps/api/core';
import type { TopicInfo, WorkedExample } from './types';

export function listTopics(): Promise<TopicInfo[]> {
    return invoke('list_topics');
}

export function scriptExamples(script: string): Promise<WorkedExample[]> {
    return invoke('script_examples', { script });
}
