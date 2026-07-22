<script setup lang="ts">
import { Combobox, ComboboxAnchor, ComboboxInput, ComboboxItem, ComboboxTrigger, ComboboxVirtualList } from '@/components/ui/combobox';
import { useMapUserSettings } from '@/composables/useMapUserSettings';
import { getTypeById } from '@/const/signatures';
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

const filtered_options = computed<TSignatureType[]>(() => {
    const needle = search.value.trim().toLowerCase();
    if (needle === '') {
        return options;
    }
    return options.filter((option) => option.name.toLowerCase().includes(needle));
});

const selected_option = computed(() => {
    if (!model.value) return null;
    return options.find((option) => option.id === model.value) ?? getTypeById(model.value) ?? null;
});

function handleSelect(option: TSignatureType) {
    model.value = option.id;
    open.value = false;
}

function optionName(option: TSignatureType): string {
    return option.name;
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
        <ComboboxVirtualList :options="filtered_options" :text-content="optionName" empty-text="Unknown" class="min-w-44">
            <template #header>
                <ComboboxInput v-model="search" placeholder="Search types" class="h-8 border-b text-xs" auto-focus />
            </template>
            <template #default="{ option }">
                <ComboboxItem :value="option" class="text-xs" @select.prevent="() => handleSelect(option)">
                    <span class="truncate">{{ option.name }}</span>
                </ComboboxItem>
            </template>
        </ComboboxVirtualList>
    </Combobox>
</template>

<style scoped></style>
