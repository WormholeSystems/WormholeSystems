<script setup lang="ts">
import SolarsystemClass from '@/components/solarsystem/SolarsystemClass.vue';
import VirtualizedSolarsystemList from '@/components/solarsystem/VirtualizedSolarsystemList.vue';
import { Combobox, ComboboxAnchor, ComboboxInput } from '@/components/ui/combobox';
import { useStaticData } from '@/composables/useStaticData';
import { type TComboboxSection } from '@/lib/comboboxSections';
import { MAX_SEARCH_RESULTS, takeRanked } from '@/lib/searchRank';
import type { TStaticSolarsystem } from '@/types/static-data';
import { computed, ref, watch } from 'vue';

/**
 * Search picker that keeps the picked system in the input instead of clearing it,
 * for forms where the selection has to survive until the form is submitted.
 */
const { inputId, placeholder = 'Search for a system…' } = defineProps<{
    inputId?: string;
    placeholder?: string;
}>();

/** 0 means nothing is selected. */
const solarsystemId = defineModel<number>({ required: true });

const { staticData, loadStaticData } = useStaticData();

void loadStaticData();

const solarsystems = computed(() => staticData.value?.solarsystems ?? []);
const selected = computed(() => solarsystems.value.find((solarsystem) => solarsystem.id === solarsystemId.value));

const search = ref(selected.value?.name ?? '');

const results = computed(() => {
    const query = search.value.trim().toLowerCase();
    if (!query) return [] as TStaticSolarsystem[];
    return takeRanked(
        solarsystems.value,
        query,
        MAX_SEARCH_RESULTS,
        (solarsystem) => [solarsystem.name],
        (solarsystem) => solarsystem.name,
    );
});

const sections = computed<TComboboxSection<TStaticSolarsystem>[]>(() => [{ key: 'results', heading: 'Search Results', items: results.value }]);

/**
 * Guards the two watchers below against each other: only changes coming from
 * outside the component should overwrite the query, not the ones we make here.
 * The watchers flush synchronously so the flag is still set when they run.
 */
let syncingFromSearch = false;

function withoutQuerySync(update: () => void): void {
    syncingFromSearch = true;
    update();
    syncingFromSearch = false;
}

// Editing the query after a pick drops the selection.
watch(
    search,
    (value) => {
        if (selected.value && value.trim() !== selected.value.name) {
            withoutQuerySync(() => (solarsystemId.value = 0));
        }
    },
    { flush: 'sync' },
);

// Follow selections set from the outside, e.g. when a form is loaded for editing.
// Keyed off the resolved system so a preselected id still fills the query once
// the static data has loaded.
watch(
    selected,
    (solarsystem) => {
        if (syncingFromSearch) return;
        search.value = solarsystem?.name ?? '';
    },
    { flush: 'sync' },
);

function handleSelect(solarsystem: TStaticSolarsystem): void {
    withoutQuerySync(() => (solarsystemId.value = solarsystem.id));
    search.value = solarsystem.name;
}
</script>

<template>
    <div class="space-y-1.5">
        <div v-if="selected" class="flex items-center gap-2 text-sm">
            <SolarsystemClass :solarsystem_class="selected.class" :name="selected.name" />
            <span class="font-medium">{{ selected.name }}</span>
        </div>
        <!--
            Both resets have to be off: the combobox itself holds no value, so reka would
            clear the query when the list closes and again when the input mounts, and this
            component reads a cleared query as "the selection was edited away".
        -->
        <Combobox
            class="rounded-lg border bg-neutral-900"
            :ignore-filter="true"
            :reset-search-term-on-blur="false"
            :reset-search-term-on-select="false"
        >
            <ComboboxAnchor>
                <ComboboxInput :id="inputId" v-model="search" :placeholder="placeholder" />
            </ComboboxAnchor>
            <VirtualizedSolarsystemList align="start" :sections="sections" @select="handleSelect" />
        </Combobox>
    </div>
</template>
