<script setup lang="ts">
import SolarsystemSearchResult from '@/components/solarsystem/SolarsystemSearchResult.vue';
import { ComboboxItem, ComboboxVirtualList } from '@/components/ui/combobox';
import { comboboxRowText, flattenComboboxSections, type TComboboxRow, type TComboboxSection } from '@/lib/comboboxSections';
import type { TStaticSolarsystem } from '@/types/static-data';
import { computed } from 'vue';

/**
 * Virtualized body for solarsystem picker comboboxes. Every row carries its own
 * fixed grid template: virtual rows are absolutely positioned, so subgrid
 * alignment against a shared ancestor is not available.
 */
const { sections, getAlias } = defineProps<{
    sections: TComboboxSection<TStaticSolarsystem>[];
    getAlias?: (solarsystemId: number) => string | null;
}>();

const emit = defineEmits<{
    select: [solarsystem: TStaticSolarsystem, section: string];
}>();

const rows = computed(() => flattenComboboxSections(sections));

function handleSelect(row: TComboboxRow<TStaticSolarsystem>) {
    // The disabled binding alone does not stop the select event from firing.
    if (row.kind !== 'option' || !row.selectable) {
        return;
    }
    emit('select', row.value, row.section);
}

function rowText(row: TComboboxRow<TStaticSolarsystem>): string {
    return comboboxRowText(row, (solarsystem) => solarsystem.name);
}
</script>

<template>
    <ComboboxVirtualList :options="rows" :text-content="rowText" empty-text="No systems found">
        <template #default="{ option }">
            <div class="w-full px-1">
                <div v-if="option.kind === 'heading'" class="px-2 py-1.5 text-xs font-medium text-muted-foreground">{{ option.label }}</div>
                <ComboboxItem
                    v-else
                    :value="`${option.section}-${option.value.id}`"
                    :disabled="!option.selectable"
                    class="grid grid-cols-[2.5rem_minmax(0,1fr)_minmax(0,1fr)_2rem] items-center gap-x-2 data-[disabled]:opacity-60"
                    @select.prevent="() => handleSelect(option)"
                >
                    <SolarsystemSearchResult :solarsystem="option.value" :alias="getAlias?.(option.value.id) ?? null" />
                </ComboboxItem>
            </div>
        </template>
    </ComboboxVirtualList>
</template>

<style scoped></style>
