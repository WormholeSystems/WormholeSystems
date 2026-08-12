<script setup lang="ts">
import { TMapSolarsystemStatus } from '@/types/models';
import { Activity, CircleDashed, CircleHelp, Radar, ShieldCheck, Skull, type LucideProps } from 'lucide-vue-next';
import { computed, type FunctionalComponent } from 'vue';

/**
 * Glyph for a system status. The shape carries the distinction for colorblind
 * users; the status color stays as the fast visual channel.
 */
const { status } = defineProps<{
    status: TMapSolarsystemStatus;
}>();

const meta: Record<TMapSolarsystemStatus, { icon: FunctionalComponent<LucideProps>; classes: string }> = {
    friendly: { icon: ShieldCheck, classes: 'text-friendly' },
    hostile: { icon: Skull, classes: 'text-hostile' },
    active: { icon: Activity, classes: 'text-active' },
    unscanned: { icon: Radar, classes: 'text-unscanned' },
    empty: { icon: CircleDashed, classes: 'text-empty' },
    unknown: { icon: CircleHelp, classes: 'text-muted-foreground' },
};

const entry = computed(() => meta[status]);
</script>

<template>
    <component :is="entry.icon" v-if="entry" class="size-[14px]" :class="entry.classes" :aria-label="status" />
</template>

<style scoped></style>
