import PingController from '@/actions/App/Http/Controllers/PingController';
import { TMap } from '@/types/models';
import { useFetch, useIntervalFn } from '@vueuse/core';
import { MaybeRefOrGetter, toValue } from 'vue';

const PING_INTERVAL_SECONDS = 60 * 5 * 1000;

/**
 * Ping the server every 5 minutes to keep the server aware of the active map/user.
 */
export function usePing(map: MaybeRefOrGetter<TMap>) {
    const { execute } = useFetch(() => PingController.show(toValue(map).slug).url, {
        immediate: false,
    });
    useIntervalFn(execute, PING_INTERVAL_SECONDS, {
        immediateCallback: true,
    });
}
