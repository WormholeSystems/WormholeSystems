<script setup lang="ts">
import VirtualizedSolarsystemList from '@/components/solarsystem/VirtualizedSolarsystemList.vue';
import { type TComboboxSection } from '@/lib/comboboxSections';
import type { TStaticSolarsystem } from '@/types/static-data';
import { computed } from 'vue';

const { solarsystems, aliases } = defineProps<{
    solarsystems: TStaticSolarsystem[];
    aliases?: Map<number, string>;
}>();

const emit = defineEmits<{
    select: [system: TStaticSolarsystem];
}>();

const sections = computed<TComboboxSection<TStaticSolarsystem>[]>(() => [{ key: 'systems', heading: '', items: solarsystems }]);

function handleSelect(system: TStaticSolarsystem) {
    emit('select', system);
}

function getAlias(solarsystemId: number): string | null {
    return aliases?.get(solarsystemId) ?? null;
}
</script>

<template>
    <VirtualizedSolarsystemList class="w-[600px]" :sections="sections" :get-alias="getAlias" @select="handleSelect" />
</template>
