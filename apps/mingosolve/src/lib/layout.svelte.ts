// Window-width layout state: below the breakpoint the rail shrinks to icons and the calculator folds into its button.
// The user's own open/closed choice for the calculator (settings.calcOpen) is left alone and applies above it.
export const NARROW_BELOW = 1100;

const query = window.matchMedia(`(max-width: ${NARROW_BELOW - 0.02}px)`);

export const layout = $state({
    narrow: query.matches,
    /** The calculator was opened on purpose while narrow; forgotten once the window is wide again. */
    calcPeek: false,
});

query.addEventListener('change', (e) => {
    layout.narrow = e.matches;
    if (!e.matches) layout.calcPeek = false;
});
