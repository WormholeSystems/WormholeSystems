<script setup lang="ts">
import type { TSignatureType, TStringedSolarsystemClass } from '@/types/models';
import { computed } from 'vue';

const props = defineProps<{
    wormhole: TSignatureType;
}>();

const display_class = computed(() => {
    const target_class = props.wormhole.target_class;
    if (!target_class) return '';

    const stringed_target_class: TStringedSolarsystemClass = String(target_class) as TStringedSolarsystemClass;

    // If it's a numeric class (1-18), add C prefix
    if (/^\d+$/.test(stringed_target_class)) {
        return `C${stringed_target_class}`;
    }

    if (stringed_target_class === 'unknown') return 'Unknown';

    return stringed_target_class.toUpperCase();
});
</script>

<template>
    <span class="flex gap-2">
        <span class="inline-block w-[4ch]">
            {{ wormhole.signature }}
        </span>
        <span
            :data-class="wormhole.target_class"
            class="text-muted-foreground data-[class='1']:text-c1 data-[class='2']:text-c2 data-[class='3']:text-c3 data-[class='4']:text-c4 data-[class='5']:text-c5 data-[class='6']:text-c6 data-[class=h]:text-hs data-[class=l]:text-ls data-[class=n]:text-ns"
        >
            {{ display_class }}
        </span>
        <span v-if="wormhole.extra" class="text-muted-foreground"> ({{ wormhole.extra }}) </span>
    </span>
</template>

<style scoped></style>
