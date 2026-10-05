// Tool descriptions as the engine's list_tools returns them (data/tool_params.toml), for the sheet's tests.
import type { ToolInfo } from '../lib/types';

const rulesChoices = [
    { value: 'legacy', label: 'Older quiz keys (legacy)' },
    { value: '2026', label: '2026' },
    { value: '2027', label: '2027' },
];
export const score: ToolInfo = {
    name: 'event_score',
    doc: 'Manual dynamic event score (2026/2027 D 9.1.1). rules=legacy reproduces older quiz keys.',
    summary: 'Points a team gets in a dynamic event, from its corrected time and the fastest corrected time.',
    params: [
        {
            name: 'event',
            number: false,
            default: null,
            label: 'Event',
            unit: null,
            help: null,
            choices: [
                { value: 'skidpad', label: 'Skidpad' },
                { value: 'endurance', label: 'Endurance' },
            ],
            switch: false,
        },
        { name: 't_team', number: true, default: null, label: 'Your corrected time', unit: 's', help: null, choices: null, switch: false },
        { name: 't_min', number: true, default: null, label: 'Fastest corrected time', unit: 's', help: 'The best corrected time of any team in this event.', choices: null, switch: false },
        { name: 'rules', number: false, default: '', label: 'Rules year', unit: null, help: null, choices: rulesChoices, switch: false },
        { name: 'pmax', number: true, default: '0', label: 'Maximum points', unit: null, help: 'Leave empty to use the rulebook value for this event.', choices: null, switch: false },
        { name: 'finish', number: true, default: '1', label: 'Car finished the event', unit: null, help: null, choices: null, switch: true },
        { name: 'kappa', number: true, default: '56e6', label: 'Conductor conductivity', unit: 'S/m', help: null, choices: null, switch: false },
    ],
};
export const corrected: ToolInfo = {
    name: 'corrected_time',
    doc: 'Raw time + DOO/OC (per event) + 60 s per disobeyed flag.',
    summary: 'Adds the penalties to a raw run time.',
    params: [
        { name: 't_raw', number: true, default: null, label: 'Raw time', unit: 's', help: null, choices: null, switch: false },
        {
            name: 'event',
            number: false,
            default: 'autocross',
            label: 'Event',
            unit: null,
            help: null,
            choices: [
                { value: 'skidpad', label: 'Skidpad' },
                { value: 'autocross', label: 'Autocross' },
            ],
            switch: false,
        },
    ],
};
