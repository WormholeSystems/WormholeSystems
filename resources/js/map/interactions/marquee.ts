import { systemsInBox } from '@/map/core/geometry/hitTest';
import type { Vec2 } from '@/map/core/types';
import type { MapStore } from '@/map/store/mapStore';
import type { Gesture, GestureContext } from './gestures';

/**
 * The selection marquee on the map background. Claims a plain left press on
 * empty canvas in the manual layout; in the locked (tree) layout it needs a
 * selection modifier (Shift/Ctrl — Cmd covers macOS, where Ctrl+click is a
 * right-click), matching the old wantsSelection() split with panning.
 *
 * Selection is committed live while the box is drawn and is sticky: releasing
 * keeps it, and nodes dragged out of the box later stay selected. Systems are
 * hit-tested against their rendered anchors (renderPosition), so the tree
 * layout selects what is actually on screen. A tap without dragging clears
 * the selection, like the old zero-size box / background click did.
 */
export function createMarqueeGesture(store: MapStore): Gesture {
    let start: Vec2 = { x: 0, y: 0 };

    function positionEntries(): [number, Vec2][] {
        const entries: [number, Vec2][] = [];
        for (const id of store.systems.keys()) {
            const position = store.renderPosition(id);
            if (position) {
                entries.push([id, position]);
            }
        }
        return entries;
    }

    return {
        kind: 'marquee',
        hysteresis: 4,

        onStart(ctx: GestureContext): boolean {
            if (ctx.event.button !== 0 || ctx.nodeId !== null) return false;

            const wantsSelection = ctx.modifiers.shift || ctx.modifiers.ctrlOrMeta;
            if (store.isLayoutLocked.value && !wantsSelection) return false;

            start = ctx.basePoint;
            return true;
        },

        onMove(ctx: GestureContext): void {
            const box = { start, end: ctx.basePoint };
            store.marquee.value = box;
            store.setSelection(systemsInBox(positionEntries(), box));
        },

        onEnd(): void {
            store.marquee.value = null;
        },

        onTap(): void {
            store.clearSelection();
        },

        onCancel(): void {
            store.marquee.value = null;
        },
    };
}
