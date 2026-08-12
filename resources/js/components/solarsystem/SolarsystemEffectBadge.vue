<script setup lang="ts">
import { TWormholeEffectName } from '@/types/models';
import { computed } from 'vue';

/**
 * Letter badge for a wormhole effect. The letter carries the distinction for
 * colorblind users; the effect color stays as the fast visual channel.
 */
const { name, size = 'md' } = defineProps<{
    name: TWormholeEffectName;
    size?: 'sm' | 'md';
}>();

const meta: Record<TWormholeEffectName, { letter: string; classes: string }> = {
    Pulsar: { letter: 'P', classes: 'bg-pulsar text-white' },
    Magnetar: { letter: 'M', classes: 'bg-magnetar text-white' },
    'Wolf-Rayet Star': { letter: 'W', classes: 'bg-wolf-rayet-star text-white' },
    'Black Hole': { letter: 'B', classes: 'bg-black-hole text-white' },
    'Red Giant': { letter: 'R', classes: 'bg-red-giant text-white' },
    'Cataclysmic Variable': { letter: 'C', classes: 'bg-catalysmic-variable text-neutral-950' },
};

const badge = computed(() => meta[name]);
</script>

<template>
    <span
        v-if="badge"
        :data-size="size"
        :aria-label="name"
        class="grid size-[14px] place-items-center rounded-full text-[9px] leading-none font-semibold data-[size=sm]:size-3 data-[size=sm]:text-[8px]"
        :class="badge.classes"
        >{{ badge.letter }}</span
    >
</template>

<style scoped></style>
