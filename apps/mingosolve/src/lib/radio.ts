// Svelte action for the ARIA radio pattern: the radios of a role="radiogroup" are one Tab stop (the checked one has
// tabindex 0, the rest -1, set in the markup) and the arrow keys move to the next or previous radio and select it.
export function radioArrows(node: HTMLElement): { destroy(): void } {
    function onKeydown(e: KeyboardEvent): void {
        const step = e.key === 'ArrowRight' || e.key === 'ArrowDown' ? 1 : e.key === 'ArrowLeft' || e.key === 'ArrowUp' ? -1 : 0;
        if (step === 0) return;
        const radios = [...node.querySelectorAll<HTMLElement>('[role="radio"]')];
        const at = radios.indexOf(document.activeElement as HTMLElement);
        if (at < 0) return;
        e.preventDefault();
        const next = radios[(at + step + radios.length) % radios.length];
        next.focus();
        next.click();
    }
    node.addEventListener('keydown', onKeydown);
    return { destroy: () => node.removeEventListener('keydown', onKeydown) };
}
