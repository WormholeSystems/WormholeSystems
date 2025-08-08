import { UTCDate } from '@date-fns/utc';
import { useNow } from '@vueuse/core';
import { computed } from 'vue';

export function useNowUTC() {
    const now = useNow();
    return computed(() => new UTCDate(now.value));
}
