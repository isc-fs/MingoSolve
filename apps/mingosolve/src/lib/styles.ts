// The app styles: one skin of the same shell each, every one in a light and a dark mode. The tokens live in app.css
// (ISC, the default) and src/styles/<id>.css, selected by data-style on <html>.
export const STYLES = [
    { id: 'isc', name: 'ISC', light: 'Paper glass', dark: 'Night glass', about: 'The team look: frosted glass over a lit ground, gold answer.' },
    { id: 'prontuario', name: 'Prontuario', light: 'Printed page', dark: 'Bookcloth', about: 'A formula handbook: black on white, the answer as a reversed plate.' },
    { id: 'plano', name: 'Plano', light: 'Drawing sheet', dark: 'Blueprint', about: 'A technical drawing: line weights, square corners, a title-block answer.' },
    { id: 'salidas', name: 'Salidas', light: 'Boarding pass', dark: 'Departures', about: 'Boarding pass and departures board: the answer on a split-flap tile.' },
    { id: 'datasheet', name: 'Datasheet', light: 'Page', dark: 'Reader', about: 'An IC datasheet: steel-blue heads, ruled tables, plain and dense.' },
    { id: 'grafito', name: 'Grafito', light: 'Light', dark: 'Dark', about: 'Plain and calm: neutral greys, one cobalt accent.' },
    { id: 'rosa', name: 'Rosa', light: 'Newsprint', dark: 'Wine', about: 'The maglia rosa: claret ink on rose, the answer in the leader’s pink.' },
] as const;

export type StyleId = (typeof STYLES)[number]['id'];

export function isStyle(id: unknown): id is StyleId {
    return STYLES.some((s) => s.id === id);
}
