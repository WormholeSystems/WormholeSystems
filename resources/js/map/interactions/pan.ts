import type { Vec2 } from '@/map/core/types';
import type { MapStore } from '@/map/store/mapStore';
import type { Ref } from 'vue';
import type { Gesture, GestureContext } from './gestures';

/**
 * Panning the scroll container. Mirrors the old useMapPanning buttons exactly:
 * the middle button always pans, and in the locked (tree) layout a plain left
 * drag pans too — unless a selection modifier (Shift/Ctrl/Cmd) is held, which
 * belongs to the marquee. Movement is applied as raw screen pixels against the
 * scroll offsets captured at gesture start (no base-unit conversion).
 *
 * A left tap on the background (the old `background_press` click-without-drag
 * test, < 4px of travel) clears the selection.
 */
export function createPanGesture(store: MapStore, surface: Ref<HTMLElement | null>): Gesture {
    let startClient: Vec2 = { x: 0, y: 0 };
    let startScroll = { left: 0, top: 0 };
    let cursorSet = false;

    function resetCursor(): void {
        if (cursorSet) {
            cursorSet = false;
            const element = surface.value;
            if (element) {
                element.style.cursor = '';
            }
        }
    }

    return {
        kind: 'pan',
        hysteresis: 4,

        onStart(ctx: GestureContext): boolean {
            const element = surface.value;
            if (!element) return false;

            const wantsSelection = ctx.modifiers.shift || ctx.modifiers.ctrlOrMeta;
            const canPan = ctx.event.button === 1 || (ctx.event.button === 0 && store.isLayoutLocked.value && !wantsSelection);
            if (!canPan) return false;

            startClient = { x: ctx.event.clientX, y: ctx.event.clientY };
            startScroll = { left: element.scrollLeft, top: element.scrollTop };
            return true;
        },

        onMove(ctx: GestureContext): void {
            const element = surface.value;
            if (!element) return;

            if (!cursorSet) {
                cursorSet = true;
                element.style.cursor = 'grabbing';
            }

            element.scrollLeft = startScroll.left - (ctx.event.clientX - startClient.x);
            element.scrollTop = startScroll.top - (ctx.event.clientY - startClient.y);
        },

        onEnd(): void {
            resetCursor();
        },

        onTap(ctx: GestureContext): void {
            resetCursor();
            if (ctx.event.button === 0 && ctx.nodeId === null) {
                store.clearSelection();
            }
        },

        onCancel(): void {
            resetCursor();
        },
    };
}
