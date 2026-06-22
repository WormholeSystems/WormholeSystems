<script setup lang="ts">
import { classMeta } from '@/const/solarsystemClasses';
import type { TSignatureType } from '@/types/models';
import { computed } from 'vue';

const props = defineProps<{
    wormhole: TSignatureType;
}>();

const meta = computed(() => (props.wormhole.target_class ? classMeta(props.wormhole.target_class) : null));
</script>

<template>
    <span class="flex gap-2">
        <span class="inline-block w-[4ch]">
            {{ wormhole.signature }}
        </span>
        <span v-if="meta" :class="`text-${meta.color_token}`">
            {{ meta.short_label }}
        </span>
        <span v-if="wormhole.extra" class="text-muted-foreground"> ({{ wormhole.extra }}) </span>
    </span>
</template>

<style scoped></style>
