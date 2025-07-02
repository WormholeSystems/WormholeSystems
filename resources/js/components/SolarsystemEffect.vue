<script setup lang="ts">
import { TWormholeEffectName } from '@/types/models';

const { effect } = defineProps<{
    effect: TWormholeEffectName;
}>();
</script>

<template>
    <span v-if="effect === 'Pulsar'" class="inline-block size-1 animate-pulse rounded-full bg-blue-500" />
    <span v-if="effect === 'Magnetar'" class="inline-block size-1 animate-pulse rounded-full bg-pink-500" />
    <span v-if="effect === 'Wolf-Rayet Star'" class="inline-block size-1 animate-pulse rounded-full bg-amber-500" />
    <span v-if="effect === 'Black Hole'" class="inline-block size-1 animate-pulse rounded-full bg-red-500" />
    <span v-if="effect === 'Red Giant'" class="inline-block size-1 animate-pulse rounded-full bg-red-500" />
    <span v-if="effect === 'Cataclysmic Variable'" class="inline-block size-1 animate-pulse rounded-full bg-orange-500" />
</template>

<style scoped></style>
