<script setup lang="ts">
import UnknownTypeOption from '@/components/signatures/UnknownTypeOption.vue';
import { Combobox, ComboboxAnchor, ComboboxInput, ComboboxItem, ComboboxTrigger, ComboboxVirtualList } from '@/components/ui/combobox';
import { useMapUserSettings } from '@/composables/useMapUserSettings';
import { getTypeById } from '@/const/signatures';
import { comboboxRowText, flattenComboboxSections, type TComboboxRow } from '@/lib/comboboxSections';
import { TSignatureType } from '@/types/models';
import { computed, ref, watch } from 'vue';

const { options, rawTypeName, category } = defineProps<{
    can_write: boolean;
    options: TSignatureType[];
    rawTypeName?: string | null;
    category?: string;
}>();

const model = defineModel<number | null>({
    required: true,
});

const hasRawTypeName = computed(() => !model.value && rawTypeName);

const map_user_settings = useMapUserSettings();

const open = ref(false);
const search = ref('');

watch(open, (isOpen) => {
    if (isOpen) {
        search.value = '';
    }
});

// A pinned null row lets a signature be reset to an unknown type.
const rows = computed<TComboboxRow<TSignatureType | null>[]>(() => {
    const needle = search.value.trim().toLowerCase();
    const matched = needle === '' ? options : options.filter((option) => option.name.toLowerCase().includes(needle));

    return flattenComboboxSections<TSignatureType | null>([
        { key: 'unknown', heading: '', items: needle === '' || 'unknown'.includes(needle) ? [null] : [] },
        { key: 'types', heading: '', items: matched },
    ]);
});

const selected_option = computed(() => {
    if (!model.value) return null;
    return options.find((option) => option.id === model.value) ?? getTypeById(model.value) ?? null;
});

function handleSelect(row: TComboboxRow<TSignatureType | null>) {
    if (row.kind !== 'option') {
        return;
    }
    model.value = row.value?.id ?? null;
    open.value = false;
}

function rowText(row: TComboboxRow<TSignatureType | null>): string {
    return comboboxRowText(row, (option) => option?.name ?? 'Unknown');
}
</script>

<template>
    <Combobox v-model:open="open" :ignore-filter="true" :disabled="!can_write || !category">
        <ComboboxAnchor as-child>
            <ComboboxTrigger class="text-xs" :size="map_user_settings.compact_signature_list ? 'sm' : 'default'" :disabled="!can_write || !category">
                <span v-if="hasRawTypeName" class="truncate text-foreground">{{ rawTypeName }}</span>
                <span v-else-if="selected_option" class="truncate">{{ selected_option.name }}</span>
                <span v-else class="truncate text-muted-foreground">Type</span>
            </ComboboxTrigger>
        </ComboboxAnchor>
        <ComboboxVirtualList :options="rows" :text-content="rowText" empty-text="No types found" class="min-w-44">
            <template #header>
                <ComboboxInput v-model="search" placeholder="Search types" class="h-8 border-b text-xs" auto-focus />
            </template>
            <template #default="{ option }">
                <ComboboxItem v-if="option.kind === 'option'" :value="option" class="text-xs" @select.prevent="() => handleSelect(option)">
                    <UnknownTypeOption v-if="option.value === null" />
                    <span v-else class="truncate">{{ option.value.name }}</span>
                </ComboboxItem>
            </template>
        </ComboboxVirtualList>
    </Combobox>
</template>

<style scoped></style>
