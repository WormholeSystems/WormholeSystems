const is_ssr = import.meta.env.SSR;

export function isSSR(): boolean {
    return is_ssr;
}

export function isPWA(): boolean {
    if (is_ssr) {
        return false;
    }

    return window.matchMedia('(display-mode: standalone)').matches;
}

export function useOnClient(callback: () => any) {
    if (is_ssr) {
        return;
    }

    callback();
}
