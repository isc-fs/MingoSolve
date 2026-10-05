// Geometry for the guided tour: where the card goes so it never covers the highlighted element.
export interface Box {
    left: number;
    top: number;
    width: number;
    height: number;
}

export interface Placement {
    left: number;
    top: number;
    /** The card still overlaps the target (nothing fit): the caller may scroll the target and try again. */
    overlaps: boolean;
}

const overlap = (a: Box, b: Box): number =>
    Math.max(0, Math.min(a.left + a.width, b.left + b.width) - Math.max(a.left, b.left)) *
    Math.max(0, Math.min(a.top + a.height, b.top + b.height) - Math.max(a.top, b.top));

/** Grow a box by `by` on every side, kept inside the viewport. */
export function inflate(b: Box, by: number, vw: number, vh: number): Box {
    const left = Math.max(0, b.left - by);
    const top = Math.max(0, b.top - by);
    return { left, top, width: Math.min(vw, b.left + b.width + by) - left, height: Math.min(vh, b.top + b.height + by) - top };
}

/** Centre in the viewport with no target; otherwise below, above, right, then left of it, whichever fits first. */
export function placeCard(
    target: Box | null,
    card: { width: number; height: number },
    vp: { width: number; height: number },
    gap = 14,
    margin = 12,
): Placement {
    const clampX = (x: number): number => Math.min(Math.max(margin, x), Math.max(margin, vp.width - card.width - margin));
    const clampY = (y: number): number => Math.min(Math.max(margin, y), Math.max(margin, vp.height - card.height - margin));
    if (target === null) {
        return { left: clampX((vp.width - card.width) / 2), top: clampY((vp.height - card.height) / 2), overlaps: false };
    }
    const cx = target.left + target.width / 2 - card.width / 2;
    const cy = target.top + target.height / 2 - card.height / 2;
    const below = target.top + target.height + gap;
    const above = target.top - gap - card.height;
    const right = target.left + target.width + gap;
    const left = target.left - gap - card.width;
    const candidates = [
        { left: clampX(cx), top: below, fits: below + card.height <= vp.height - margin },
        { left: clampX(cx), top: above, fits: above >= margin },
        { left: right, top: clampY(cy), fits: right + card.width <= vp.width - margin },
        { left, top: clampY(cy), fits: left >= margin },
    ];
    const free = candidates.find((c) => c.fits);
    if (free !== undefined) return { left: free.left, top: free.top, overlaps: false };
    // nothing fits: take the least overlapping spot, kept inside the viewport
    let best: Placement = { left: clampX(cx), top: clampY(below), overlaps: true };
    let bestArea = Infinity;
    for (const c of candidates) {
        const p = { left: clampX(c.left), top: clampY(c.top) };
        const area = overlap({ ...p, width: card.width, height: card.height }, target);
        if (area < bestArea) {
            bestArea = area;
            best = { ...p, overlaps: area > 0 };
        }
    }
    return best;
}
