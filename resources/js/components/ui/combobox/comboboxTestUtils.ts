import { vi } from 'vitest';

/**
 * happy-dom performs no layout, so every element measures 0x0 and the
 * virtualizer would render an empty range. tanstack/virtual seeds its
 * measurement from getBoundingClientRect, so stubbing it with a fixed size is
 * enough to make virtual rows render in tests.
 */
export function stubElementMeasurements(width = 300, height = 300): void {
    const rect = {
        width,
        height,
        top: 0,
        left: 0,
        right: width,
        bottom: height,
        x: 0,
        y: 0,
        toJSON: () => ({}),
    } as DOMRect;

    // happy-dom overrides getBoundingClientRect at the HTMLElement level, so
    // stubbing Element.prototype alone would not be picked up.
    vi.spyOn(HTMLElement.prototype, 'getBoundingClientRect').mockImplementation(() => rect);
    vi.spyOn(Element.prototype, 'getBoundingClientRect').mockImplementation(() => rect);

    // tanstack/virtual measures the scroll element via offsetWidth/offsetHeight.
    vi.spyOn(HTMLElement.prototype, 'offsetWidth', 'get').mockReturnValue(width);
    vi.spyOn(HTMLElement.prototype, 'offsetHeight', 'get').mockReturnValue(height);
}

/** Flush the mount + virtualizer measurement microtasks. */
export async function flushVirtualizer(): Promise<void> {
    await new Promise((resolve) => setTimeout(resolve, 0));
    await new Promise((resolve) => setTimeout(resolve, 0));
}
