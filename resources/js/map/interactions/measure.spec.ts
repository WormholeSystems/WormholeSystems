import type { Size } from '@/map/core/types';
import { resetNodeMeasurement, useNodeMeasurement, type NodeSizeSink } from '@/map/interactions/measure';
import { afterEach, beforeEach, describe, expect, it } from 'vitest';

/**
 * The module runs against browser globals (ResizeObserver, rAF); vitest runs in
 * plain node, so both are stubbed explicitly and driven by hand.
 */
class FakeResizeObserver {
    static instances: FakeResizeObserver[] = [];

    observed: Element[] = [];

    constructor(private readonly callback: ResizeObserverCallback) {
        FakeResizeObserver.instances.push(this);
    }

    observe(target: Element): void {
        this.observed.push(target);
    }

    unobserve(target: Element): void {
        this.observed = this.observed.filter((element) => element !== target);
    }

    disconnect(): void {
        this.observed = [];
    }

    trigger(targets: Element[]): void {
        const entries = targets.map((target) => ({ target }) as ResizeObserverEntry);
        this.callback(entries, this as unknown as ResizeObserver);
    }
}

let frameQueue: FrameRequestCallback[] = [];

function runFrame(): void {
    const callbacks = frameQueue;
    frameQueue = [];
    for (const callback of callbacks) {
        callback(0);
    }
}

function fakeElement(width: number, height: number): HTMLElement {
    return { offsetWidth: width, offsetHeight: height } as HTMLElement;
}

/** A sink whose nodeSizes Map counts writes, so "no write" is observable. */
function fakeSink(): NodeSizeSink & { writes: number } {
    const sizes = new Map<number, Size>();
    const sink = {
        nodeSizes: sizes,
        writes: 0,
    };
    const originalSet = sizes.set.bind(sizes);
    sizes.set = (key: number, value: Size) => {
        sink.writes += 1;
        return originalSet(key, value);
    };
    return sink;
}

function currentObserver(): FakeResizeObserver {
    const observer = FakeResizeObserver.instances.at(-1);
    if (!observer) {
        throw new Error('no ResizeObserver was created');
    }
    return observer;
}

beforeEach(() => {
    FakeResizeObserver.instances = [];
    frameQueue = [];
    globalThis.ResizeObserver = FakeResizeObserver as unknown as typeof ResizeObserver;
    globalThis.requestAnimationFrame = (callback: FrameRequestCallback): number => {
        frameQueue.push(callback);
        return frameQueue.length;
    };
    globalThis.cancelAnimationFrame = () => {};
});

afterEach(() => {
    resetNodeMeasurement();
});

describe('useNodeMeasurement', () => {
    it('shares one ResizeObserver across nodes and flushes a batch in one frame', () => {
        const sink = fakeSink();
        const { observeNode } = useNodeMeasurement(sink);
        const elements = [fakeElement(100, 40), fakeElement(120, 60), fakeElement(80, 40)];

        observeNode(elements[0], 1);
        observeNode(elements[1], 2);
        observeNode(elements[2], 3);

        expect(FakeResizeObserver.instances).toHaveLength(1);
        expect(currentObserver().observed).toHaveLength(3);

        currentObserver().trigger([elements[0], elements[1]]);
        currentObserver().trigger([elements[2]]);
        expect(frameQueue).toHaveLength(1);
        expect(sink.writes).toBe(0);

        runFrame();

        expect(sink.writes).toBe(3);
        expect(sink.nodeSizes.get(1)).toEqual({ width: 100, height: 40 });
        expect(sink.nodeSizes.get(2)).toEqual({ width: 120, height: 60 });
        expect(sink.nodeSizes.get(3)).toEqual({ width: 80, height: 40 });
    });

    it('skips the write when the measured size is unchanged', () => {
        const sink = fakeSink();
        const { observeNode } = useNodeMeasurement(sink);
        const element = fakeElement(100, 40);
        sink.nodeSizes.set(1, { width: 100, height: 40 });
        sink.writes = 0;

        observeNode(element, 1);
        currentObserver().trigger([element]);
        runFrame();

        expect(sink.writes).toBe(0);
        expect(sink.nodeSizes.get(1)).toEqual({ width: 100, height: 40 });
    });

    it('writes when the size actually changed', () => {
        const sink = fakeSink();
        const { observeNode } = useNodeMeasurement(sink);
        const element = fakeElement(100, 40);
        sink.nodeSizes.set(1, { width: 100, height: 60 });
        sink.writes = 0;

        observeNode(element, 1);
        currentObserver().trigger([element]);
        runFrame();

        expect(sink.writes).toBe(1);
        expect(sink.nodeSizes.get(1)).toEqual({ width: 100, height: 40 });
    });

    it('ignores zero-sized measurements', () => {
        const sink = fakeSink();
        const { observeNode } = useNodeMeasurement(sink);
        const element = fakeElement(0, 0);

        observeNode(element, 1);
        currentObserver().trigger([element]);
        runFrame();

        expect(sink.writes).toBe(0);
        expect(sink.nodeSizes.has(1)).toBe(false);
    });

    it('unobserve stops observing, forgets the size, and drops pending entries', () => {
        const sink = fakeSink();
        const { observeNode, unobserveNode } = useNodeMeasurement(sink);
        const element = fakeElement(100, 40);

        observeNode(element, 1);
        currentObserver().trigger([element]);
        runFrame();
        expect(sink.nodeSizes.get(1)).toEqual({ width: 100, height: 40 });

        currentObserver().trigger([element]);
        unobserveNode(element, 1);

        expect(currentObserver().observed).toHaveLength(0);
        expect(sink.nodeSizes.has(1)).toBe(false);

        sink.writes = 0;
        runFrame();
        expect(sink.writes).toBe(0);
        expect(sink.nodeSizes.has(1)).toBe(false);
    });
});
