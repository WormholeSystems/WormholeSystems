<script setup lang="ts">
import type { TSignatureType } from '@/const/signatures';
import { computed } from 'vue';

const props = defineProps<{
    wormhole: TSignatureType;
}>();

const displayClass = computed(() => {
    const targetClass = props.wormhole.target_class;
    if (!targetClass) return '';
    
    // If it's a numeric class (1-18), add C prefix
    if (/^\d+$/.test(targetClass)) {
        return `C${targetClass}`;
    }
    
    // Convert hs/ls/ns to H/L/N
    if (targetClass === 'hs') return 'H';
    if (targetClass === 'ls') return 'L';
    if (targetClass === 'ns') return 'N';
    
    // Otherwise return as-is (for pv, unknown, etc.)
    return targetClass.toUpperCase();
});
</script>

<template>
    <span class="flex gap-2">
        <span class="inline-block w-[4ch]">
            {{ wormhole.signature }}
        </span>
        <span
            :data-class="wormhole.target_class"
            class="text-muted-foreground data-[class='1']:text-c1 data-[class='2']:text-c2 data-[class='3']:text-c3 data-[class='4']:text-c4 data-[class='5']:text-c5 data-[class='6']:text-c6 data-[class=hs]:text-hs data-[class=ls]:text-ls data-[class=ns]:text-ns"
        >
            {{ displayClass }}
        </span>
        <span v-if="wormhole.extra" class="text-muted-foreground"> ({{ wormhole.extra }}) </span>
    </span>
</template>

<style scoped></style>
