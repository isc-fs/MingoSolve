// Where the tour card goes: properties that must hold whatever the target and window, checked on a grid.
import { describe, expect, it } from 'vitest';

import { inflate, placeCard, type Box } from './tourLayout';

const vp = { width: 1280, height: 720 };
const card = { width: 380, height: 190 };
const hits = (a: Box, b: Box): boolean =>
    a.left < b.left + b.width && b.left < a.left + a.width && a.top < b.top + b.height && b.top < a.top + a.height;

describe('placeCard', () => {
    it('centres the card when there is no target', () => {
        const p = placeCard(null, card, vp);
        expect(p.left).toBe((1280 - 380) / 2);
        expect(p.top).toBe((720 - 190) / 2);
    });

    it('stays inside the window and never overlaps the target when it says so', () => {
        let tried = 0;
        let apart = 0;
        for (let left = 0; left <= 1100; left += 137) {
            for (let top = 0; top <= 640; top += 71) {
                for (const [width, height] of [[160, 40], [420, 90], [236, 300], [900, 120]]) {
                    const target: Box = { left, top, width: Math.min(width, 1280 - left), height: Math.min(height, 720 - top) };
                    const p = placeCard(target, card, vp);
                    const box = { left: p.left, top: p.top, width: card.width, height: card.height };
                    expect(box.left, JSON.stringify(target)).toBeGreaterThanOrEqual(0);
                    expect(box.top).toBeGreaterThanOrEqual(0);
                    expect(box.left + box.width).toBeLessThanOrEqual(vp.width);
                    expect(box.top + box.height).toBeLessThanOrEqual(vp.height);
                    // the flag does not lie: it is false exactly when the boxes are apart
                    expect(p.overlaps, JSON.stringify(target)).toBe(hits(box, target));
                    tried++;
                    if (!p.overlaps) apart++;
                }
            }
        }
        expect(tried).toBeGreaterThan(100);
        expect(apart).toBeGreaterThan(tried / 2);
    });

    it('prefers below, then above, then beside', () => {
        const mid = { left: 400, top: 250, width: 300, height: 60 };
        expect(placeCard(mid, card, vp).top).toBeGreaterThan(310);
        const bottom = { left: 400, top: 600, width: 300, height: 60 };
        expect(placeCard(bottom, card, vp).top + card.height).toBeLessThan(600);
        const rail = { left: 0, top: 0, width: 236, height: 720 };
        const side = placeCard(rail, card, vp);
        expect(side.left).toBeGreaterThanOrEqual(236);
        expect(side.overlaps).toBe(false);
    });

    it('reports an overlap when the target fills the window', () => {
        expect(placeCard({ left: 0, top: 0, width: 1280, height: 720 }, card, vp).overlaps).toBe(true);
    });
});

describe('inflate', () => {
    it('grows a box and stops at the window edge', () => {
        expect(inflate({ left: 100, top: 50, width: 400, height: 60 }, 6, 1280, 720)).toEqual({ left: 94, top: 44, width: 412, height: 72 });
        expect(inflate({ left: 2, top: 0, width: 1278, height: 720 }, 6, 1280, 720)).toEqual({ left: 0, top: 0, width: 1280, height: 720 });
    });
});
