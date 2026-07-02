import { router } from '@inertiajs/vue3';

export type ReloadCoalescer = {
    schedule(props: string[]): void;
    /** Test hook: flush synchronously without waiting for the debounce. */
    flushNow(): void;
};

type CoalescerOptions = {
    delayMs?: number;
    reload?: (props: string[]) => void;
};

/**
 * The only place sync-driven Inertia reloads happen: prop names are unioned and
 * flushed once per debounce window, so a burst of broadcast events costs one
 * HTTP request with the combined `only` list instead of one request each.
 */
export function createReloadCoalescer(options: CoalescerOptions = {}): ReloadCoalescer {
    const delayMs = options.delayMs ?? 150;
    const reload = options.reload ?? ((props: string[]) => router.reload({ only: props }));

    const pending = new Set<string>();
    let timer: ReturnType<typeof setTimeout> | null = null;

    function flush(): void {
        timer = null;
        if (pending.size === 0) return;
        const props = [...pending];
        pending.clear();
        reload(props);
    }

    function schedule(props: string[]): void {
        if (props.length === 0) return;
        props.forEach((prop) => pending.add(prop));
        if (timer !== null) {
            clearTimeout(timer);
        }
        timer = setTimeout(flush, delayMs);
    }

    return {
        schedule,
        flushNow: flush,
    };
}
