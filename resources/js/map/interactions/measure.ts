import type { Size } from '@/map/core/types';
import { useMapStore } from '@/map/store/mapStore';

/**
 * Shared node measurement: every node registers its card element with one
 * module-level ResizeObserver, and all resize entries of a frame are flushed
 * into the store's nodeSizes Map in a single requestAnimationFrame, so N nodes
 * resizing together cost one batch of reactive writes instead of N observers.
 *
 * Sizes are read as offsetWidth/offsetHeight — the border-box layout size,
 * independent of the CSS scale transform on an ancestor — so they land in base
 * units, matching the old useElementSize(root, ..., { box: 'border-box' }).
 */
export type NodeSizeSink = {
    nodeSizes: Map<number, Size>;
};

type Registration = { id: number; sink: NodeSizeSink };

const registrations = new Map<HTMLElement, Registration>();
const pending = new Set<HTMLElement>();
let observer: ResizeObserver | null = null;
let frameHandle: number | null = null;

function ensureObserver(): ResizeObserver {
    if (!observer) {
        observer = new ResizeObserver((entries) => {
            for (const entry of entries) {
                if (registrations.has(entry.target as HTMLElement)) {
                    pending.add(entry.target as HTMLElement);
                }
            }
            scheduleFlush();
        });
    }
    return observer;
}

function scheduleFlush(): void {
    if (frameHandle !== null || pending.size === 0) {
        return;
    }
    frameHandle = requestAnimationFrame(flush);
}

function flush(): void {
    frameHandle = null;
    for (const element of pending) {
        const registration = registrations.get(element);
        if (!registration) {
            continue;
        }
        const width = element.offsetWidth;
        const height = element.offsetHeight;
        if (width <= 0 || height <= 0) {
            continue;
        }
        const existing = registration.sink.nodeSizes.get(registration.id);
        if (existing && existing.width === width && existing.height === height) {
            continue;
        }
        registration.sink.nodeSizes.set(registration.id, { width, height });
    }
    pending.clear();
}

export function useNodeMeasurement(sink: NodeSizeSink = useMapStore()) {
    function observeNode(el: HTMLElement, id: number): void {
        registrations.set(el, { id, sink });
        ensureObserver().observe(el, { box: 'border-box' });
    }

    /** Stops watching the element and forgets the node's size (the old forgetNodeSize). */
    function unobserveNode(el: HTMLElement, id: number): void {
        observer?.unobserve(el);
        registrations.delete(el);
        pending.delete(el);
        sink.nodeSizes.delete(id);
    }

    return { observeNode, unobserveNode };
}

/** Test-only: drops the shared observer and any queued measurements. */
export function resetNodeMeasurement(): void {
    observer?.disconnect();
    observer = null;
    registrations.clear();
    pending.clear();
    if (frameHandle !== null && typeof cancelAnimationFrame === 'function') {
        cancelAnimationFrame(frameHandle);
    }
    frameHandle = null;
}
