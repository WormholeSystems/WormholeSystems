<script setup lang="ts">
import { TWormholeDefinition } from '@/const/signatures';

defineProps<{
    wormhole: TWormholeDefinition;
}>();
</script>

<template>
    <span>
        {{ wormhole.signature }}
        <span
            :data-class="wormhole.target_class"
            class="w-[4ch] text-muted-foreground data-[class=C1]:text-c1 data-[class=C2]:text-c2 data-[class=C2/C3]:text-c2 data-[class=C3]:text-c3 data-[class=C4]:text-c4 data-[class=C4/C5]:text-c4 data-[class=C5]:text-c5 data-[class=C6]:text-c6 data-[class=H]:text-hs data-[class=L]:text-ls data-[class=N]:text-ns"
        >
            {{ wormhole.target_class }}
        </span>
        <span v-if="wormhole.extra" class="text-muted-foreground"> ({{ wormhole.extra }}) </span>
    </span>
</template>

<style scoped></style>
