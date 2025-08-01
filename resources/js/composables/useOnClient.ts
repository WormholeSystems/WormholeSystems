const is_ssr = import.meta.env.SSR;

export function useOnClient(callback: () => any) {
    if (is_ssr) {
        return;
    }

    callback();
}
