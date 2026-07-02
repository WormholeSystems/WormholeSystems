import { describe, expect, it } from 'vitest';
import { claimGesture, exceedsHysteresis, type Gesture, type GestureContext } from './gestures';

function fakeContext(overrides: Partial<GestureContext> = {}): GestureContext {
    return {
        event: { button: 0 } as PointerEvent,
        basePoint: { x: 0, y: 0 },
        nodeId: null,
        hitConnectHandle: false,
        hitDragHandle: false,
        modifiers: { shift: false, ctrlOrMeta: false },
        ...overrides,
    };
}

function fakeGesture(kind: Gesture['kind'], claims: (ctx: GestureContext) => boolean): Gesture {
    return {
        kind,
        onStart: claims,
        onMove: () => undefined,
        onEnd: () => undefined,
        onCancel: () => undefined,
    };
}

describe('claimGesture', () => {
    it('gives the pointer to the first gesture that claims it', () => {
        const linkDrag = fakeGesture('link-drag', (ctx) => ctx.hitConnectHandle);
        const nodeDrag = fakeGesture('node-drag', (ctx) => ctx.hitDragHandle);
        const marquee = fakeGesture('marquee', (ctx) => ctx.nodeId === null);

        const gestures = [linkDrag, nodeDrag, marquee];

        expect(claimGesture(gestures, fakeContext({ hitConnectHandle: true, hitDragHandle: true, nodeId: 7 }))).toBe(linkDrag);
        expect(claimGesture(gestures, fakeContext({ hitDragHandle: true, nodeId: 7 }))).toBe(nodeDrag);
        expect(claimGesture(gestures, fakeContext())).toBe(marquee);
    });

    it('returns null when no gesture claims the pointer', () => {
        const nodeDrag = fakeGesture('node-drag', (ctx) => ctx.hitDragHandle);
        expect(claimGesture([nodeDrag], fakeContext({ nodeId: 7 }))).toBeNull();
    });

    it('never asks gestures after the claiming one', () => {
        let asked = false;
        const first = fakeGesture('pan', () => true);
        const second = fakeGesture('marquee', () => {
            asked = true;
            return true;
        });

        claimGesture([first, second], fakeContext());
        expect(asked).toBe(false);
    });
});

describe('exceedsHysteresis', () => {
    it('treats movement below the threshold as a tap', () => {
        expect(exceedsHysteresis({ x: 0, y: 0 }, { x: 2, y: 2 }, 4)).toBe(false);
    });

    it('commits once the pointer travelled the threshold distance', () => {
        expect(exceedsHysteresis({ x: 0, y: 0 }, { x: 4, y: 0 }, 4)).toBe(true);
        expect(exceedsHysteresis({ x: 0, y: 0 }, { x: 3, y: 3 }, 4)).toBe(true);
    });

    it('commits immediately with a zero threshold', () => {
        expect(exceedsHysteresis({ x: 0, y: 0 }, { x: 0, y: 0 }, 0)).toBe(true);
    });
});
