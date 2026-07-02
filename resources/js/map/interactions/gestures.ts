import { clientToBase, type ViewportFrame } from '@/map/core/coords';
import type { Vec2 } from '@/map/core/types';
import type { MapStore } from '@/map/store/mapStore';
import type { GestureKind } from '@/map/store/viewState';
import { useEventListener } from '@vueuse/core';
import type { Ref } from 'vue';

/**
 * The single pointer-gesture arbiter for the map surface. Exactly one gesture
 * can own the pointer at a time: on pointerdown the arbiter asks the gestures
 * in registration order and the first one whose onStart() returns true claims
 * the pointer. Below the gesture's hysteresis threshold nothing is committed —
 * a release fires onTap() (if any) and the browser's own click pipeline keeps
 * working (Inertia Links, alias popovers, …). Once the pointer travels past
 * the threshold the gesture commits: the surface captures the pointer, the
 * store's activeGesture flips to the gesture's kind, text selection is
 * suppressed, and onMove()/onEnd() drive it to completion.
 */
export type GestureContext = {
    event: PointerEvent;
    /** The pointer position in base (unscaled canvas) units. */
    basePoint: Vec2;
    /** The map node under the pointer, resolved via the data-node-id hook. */
    nodeId: number | null;
    hitConnectHandle: boolean;
    hitDragHandle: boolean;
    modifiers: { shift: boolean; ctrlOrMeta: boolean };
};

export type Gesture = {
    /** What the store's activeGesture reads while this gesture is committed. */
    kind: Exclude<GestureKind, 'none'>;
    /** Claim the pointer? Must be side-effect free apart from snapshotting state. */
    onStart(ctx: GestureContext): boolean;
    onMove(ctx: GestureContext): void;
    onEnd(ctx: GestureContext): void;
    onCancel(): void;
    /**
     * Movement (screen px) below which the pointer release counts as a tap.
     * 0 (the default) commits immediately on the claiming pointerdown.
     */
    hysteresis?: number;
    /** A press-and-release without crossing the hysteresis threshold. */
    onTap?(ctx: GestureContext): void;
};

/** Whether the pointer has moved far enough (screen px) to commit the gesture. */
export function exceedsHysteresis(start: Vec2, current: Vec2, threshold: number): boolean {
    return Math.hypot(current.x - start.x, current.y - start.y) >= threshold;
}

/** The first gesture (in order) that claims the context, or null. */
export function claimGesture(gestures: readonly Gesture[], ctx: GestureContext): Gesture | null {
    for (const gesture of gestures) {
        if (gesture.onStart(ctx)) {
            return gesture;
        }
    }
    return null;
}

/** The map node id an event target sits inside, via the data-node-id hook. */
export function resolveNodeId(target: EventTarget | null): number | null {
    if (!(target instanceof Element)) {
        return null;
    }
    const node = target.closest('[data-node-id]');
    if (!(node instanceof HTMLElement) || node.dataset.nodeId === undefined) {
        return null;
    }
    const id = Number(node.dataset.nodeId);
    return Number.isFinite(id) ? id : null;
}

type ActivePointer = {
    gesture: Gesture;
    pointerId: number;
    startClient: Vec2;
    /**
     * The viewport rect and scale are cached once at gesture start; the scroll
     * offsets are re-read from the surface on every event, so panning-by-script
     * (or a committed pan gesture itself) never desynchronizes the mapping.
     */
    rectLeft: number;
    rectTop: number;
    scale: number;
    committed: boolean;
};

export function usePointerGestures(surface: Ref<HTMLElement | null>, gestures: Gesture[], store: MapStore): void {
    let active: ActivePointer | null = null;

    function currentFrame(pointer: ActivePointer): ViewportFrame {
        const element = surface.value;
        return {
            rectLeft: pointer.rectLeft,
            rectTop: pointer.rectTop,
            scrollLeft: element?.scrollLeft ?? 0,
            scrollTop: element?.scrollTop ?? 0,
            scale: pointer.scale,
        };
    }

    function buildContext(event: PointerEvent, frame: ViewportFrame): GestureContext {
        const target = event.target instanceof Element ? event.target : null;
        return {
            event,
            basePoint: clientToBase(event.clientX, event.clientY, frame),
            nodeId: resolveNodeId(target),
            hitConnectHandle: Boolean(target?.closest('[data-connect-handle]')),
            hitDragHandle: Boolean(target?.closest('[data-drag-handle]')),
            modifiers: { shift: event.shiftKey, ctrlOrMeta: event.ctrlKey || event.metaKey },
        };
    }

    function commit(event: PointerEvent): void {
        if (!active || active.committed) {
            return;
        }
        active.committed = true;
        store.activeGesture.value = active.gesture.kind;
        try {
            surface.value?.setPointerCapture(active.pointerId);
        } catch {
            // The pointer may already be gone (e.g. released between events); the
            // window-level listeners still finish the gesture without capture.
        }
        event.preventDefault();
        document.body.style.userSelect = 'none';
    }

    function reset(): void {
        if (active?.committed) {
            try {
                surface.value?.releasePointerCapture(active.pointerId);
            } catch {
                // Capture may have been lost implicitly; nothing to release.
            }
        }
        active = null;
        store.activeGesture.value = 'none';
        document.body.style.userSelect = '';
    }

    function handlePointerDown(event: PointerEvent): void {
        if (active) {
            return;
        }
        const element = surface.value;
        if (!element) {
            return;
        }
        const rect = element.getBoundingClientRect();
        const frame: ViewportFrame = {
            rectLeft: rect.left,
            rectTop: rect.top,
            scrollLeft: element.scrollLeft,
            scrollTop: element.scrollTop,
            scale: store.scale.value,
        };
        const ctx = buildContext(event, frame);
        const gesture = claimGesture(gestures, ctx);
        if (!gesture) {
            return;
        }
        active = {
            gesture,
            pointerId: event.pointerId,
            startClient: { x: event.clientX, y: event.clientY },
            rectLeft: rect.left,
            rectTop: rect.top,
            scale: frame.scale,
            committed: false,
        };
        if ((gesture.hysteresis ?? 0) <= 0) {
            commit(event);
        }
    }

    function handlePointerMove(event: PointerEvent): void {
        if (!active || event.pointerId !== active.pointerId) {
            return;
        }
        const ctx = buildContext(event, currentFrame(active));
        if (!active.committed) {
            if (!exceedsHysteresis(active.startClient, { x: event.clientX, y: event.clientY }, active.gesture.hysteresis ?? 0)) {
                return;
            }
            commit(event);
        } else {
            event.preventDefault();
        }
        active.gesture.onMove(ctx);
    }

    function handlePointerUp(event: PointerEvent): void {
        if (!active || event.pointerId !== active.pointerId) {
            return;
        }
        const pointer = active;
        const ctx = buildContext(event, currentFrame(pointer));
        reset();
        if (pointer.committed) {
            pointer.gesture.onEnd(ctx);
        } else {
            pointer.gesture.onTap?.(ctx);
        }
    }

    function handlePointerCancel(event: PointerEvent): void {
        if (!active || event.pointerId !== active.pointerId) {
            return;
        }
        const pointer = active;
        reset();
        pointer.gesture.onCancel();
    }

    useEventListener(surface, 'pointerdown', handlePointerDown);
    useEventListener(window, 'pointermove', handlePointerMove);
    useEventListener(window, 'pointerup', handlePointerUp);
    useEventListener(window, 'pointercancel', handlePointerCancel);
}
