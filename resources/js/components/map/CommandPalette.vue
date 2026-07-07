<script setup lang="ts">
import { AllianceLogo, CorporationLogo } from '@/components/images';
import CommandPaletteAddBadge from '@/components/map/CommandPaletteAddBadge.vue';
import SolarsystemSearchResult from '@/components/solarsystem/SolarsystemSearchResult.vue';
import { useCommandPalette } from '@/composables/useCommandPalette';
import { useMapSearch } from '@/composables/useMapSearch';
import usePermission from '@/composables/usePermission';
import { useShowMap } from '@/composables/useShowMap';
import { useStaticSolarsystems } from '@/composables/useStaticSolarsystems';
import { takeRanked } from '@/lib/searchRank';
import { createMapSolarsystem, useMapSolarsystems, useSolarsystemSearch } from '@/map/api';
import type { TMapSolarsystem, TResolvedSolarsystem } from '@/pages/maps';
import { show } from '@/routes/maps';
import { router } from '@inertiajs/vue3';
import { useMagicKeys, whenever } from '@vueuse/core';
import { Search } from 'lucide-vue-next';
import {
    ComboboxContent,
    ComboboxGroup,
    ComboboxInput,
    ComboboxItem,
    ComboboxLabel,
    ComboboxRoot,
    DialogContent,
    DialogOverlay,
    DialogPortal,
    DialogRoot,
    DialogTitle,
} from 'reka-ui';
import { computed, nextTick, onUnmounted, ref, watch } from 'vue';

type TThreatSystemMatch = {
    kills: number;
    occupier_alias: string | null;
    solarsystem: TResolvedSolarsystem;
    map_solarsystem: TMapSolarsystem | null;
};

const item_classes =
    'col-span-full grid cursor-default grid-cols-subgrid items-center rounded-sm px-2 py-1.5 outline-none select-none data-[disabled]:opacity-50 data-[highlighted]:bg-accent';
const label_classes = 'col-span-full px-2 py-1.5 font-mono text-[10px] tracking-wider text-muted-foreground uppercase';

const { open } = useCommandPalette();
const search = ref('');

const page = useShowMap();
const { map_solarsystems } = useMapSolarsystems();
const { resolveSolarsystem } = useStaticSolarsystems();
const { canEdit: can_write } = usePermission();
const { threat_results, note_results, occupier_results, search: searchMap } = useMapSearch(() => page.props.map.slug);
const { new_solarsystems } = useSolarsystemSearch(search, () => map_solarsystems.value);

const keys = useMagicKeys({
    passive: false,
    onEventFired(event) {
        if ((event.metaKey || event.ctrlKey) && event.key?.toLowerCase() === 'k' && event.type === 'keydown') {
            event.preventDefault();
        }
    },
});

whenever(
    () => keys['meta+k'].value || keys['ctrl+k'].value,
    () => {
        open.value = !open.value;
    },
);

watch(open, (value) => {
    if (value) {
        search.value = '';
    }
});

// The shared `open` singleton is not reset by anything else on navigation.
watch(
    () => page.props.map.id,
    () => {
        open.value = false;
    },
);

onUnmounted(() => {
    open.value = false;
});

watch(search, (value) => {
    void searchMap(value);
});

const matched_systems = computed(() => {
    const needle = search.value.trim().toLowerCase();

    if (!needle) {
        return [];
    }

    return takeRanked(
        map_solarsystems.value,
        needle,
        8,
        (system) => [system.solarsystem.name, system.alias, system.occupier_alias],
        (system) => system.solarsystem.name,
    );
});

const systemsBySolarsystemId = computed(() => new Map(map_solarsystems.value.map((system) => [system.solarsystem_id, system])));

// Name matches from all of EVE plus systems this map only remembers by occupier alias.
const off_map_systems = computed(() => {
    const byName = new_solarsystems.value.slice(0, 6).map((solarsystem) => ({
        solarsystem,
        occupier_alias: null as string | null,
    }));

    const nameMatchedIds = new Set(byName.map((entry) => entry.solarsystem.id));

    const byOccupier = occupier_results.value
        .filter((occupier) => !systemsBySolarsystemId.value.has(occupier.solarsystem_id) && !nameMatchedIds.has(occupier.solarsystem_id))
        .map((occupier) => ({
            solarsystem: resolveSolarsystem(occupier.solarsystem_id),
            occupier_alias: occupier.occupier_alias,
        }));

    return [...byOccupier, ...byName];
});

const threat_entities = computed(() =>
    threat_results.value.map((entity) => ({
        ...entity,
        systems: entity.systems.map(
            (system): TThreatSystemMatch => ({
                kills: system.kills,
                occupier_alias: system.occupier_alias,
                solarsystem: resolveSolarsystem(system.solarsystem_id),
                map_solarsystem: systemsBySolarsystemId.value.get(system.solarsystem_id) ?? null,
            }),
        ),
    })),
);

const note_matches = computed(() =>
    note_results.value
        .map((note) => ({ ...note, map_solarsystem: systemsBySolarsystemId.value.get(note.solarsystem_id) ?? null }))
        .filter((note): note is typeof note & { map_solarsystem: TMapSolarsystem } => note.map_solarsystem !== null),
);

const hasResults = computed(
    () => matched_systems.value.length > 0 || off_map_systems.value.length > 0 || threat_entities.value.length > 0 || note_matches.value.length > 0,
);

function handleSelect(map_solarsystem: TMapSolarsystem) {
    open.value = false;

    router.visit(show(page.props.map.slug, { mergeQuery: { solarsystem_id: map_solarsystem.solarsystem_id } }));

    void nextTick(() => {
        document.querySelector(`[data-node-id="${map_solarsystem.id}"]`)?.scrollIntoView({ behavior: 'smooth', block: 'center', inline: 'center' });
    });
}

function handleAddSystem(solarsystem_id: number) {
    if (!can_write.value) {
        return;
    }

    open.value = false;
    createMapSolarsystem(solarsystem_id);
}

function handleThreatSystemSelect(system: TThreatSystemMatch) {
    if (system.map_solarsystem) {
        handleSelect(system.map_solarsystem);
        return;
    }

    handleAddSystem(system.solarsystem.id);
}
</script>

<template>
    <DialogRoot v-model:open="open">
        <DialogPortal>
            <DialogOverlay class="fixed inset-0 z-50 bg-black/60 data-[state=open]:animate-in data-[state=open]:fade-in-0" />
            <DialogContent
                class="fixed top-24 left-1/2 z-50 w-[560px] max-w-[calc(100%-2rem)] -translate-x-1/2 overflow-hidden rounded-lg border border-border bg-background shadow-lg data-[state=open]:animate-in data-[state=open]:fade-in-0 data-[state=open]:zoom-in-95"
            >
                <DialogTitle class="sr-only">Search the map</DialogTitle>
                <ComboboxRoot :open="true" ignore-filter>
                    <div class="flex items-center gap-2 border-b border-border/50 px-3">
                        <Search class="size-4 shrink-0 text-muted-foreground" />
                        <ComboboxInput
                            v-model="search"
                            auto-focus
                            placeholder="Search systems, aliases, occupiers, threats..."
                            class="h-11 w-full bg-transparent text-sm outline-none placeholder:text-muted-foreground"
                            @keydown.esc.prevent="open = false"
                        />
                        <kbd class="shrink-0 rounded border border-border/50 px-1.5 py-0.5 font-mono text-[10px] text-muted-foreground">esc</kbd>
                    </div>
                    <!-- reka forces inline flex styles on the content element, so the grid lives on an inner div. -->
                    <ComboboxContent position="inline" class="max-h-80 overflow-y-auto p-1">
                        <div class="grid grid-cols-[auto_minmax(0,1.3fr)_minmax(0,1fr)_minmax(0,1.2fr)_auto] content-start gap-x-2">
                            <!-- Systems on the map, then matching systems anywhere in EVE -->
                            <ComboboxGroup v-if="matched_systems.length || off_map_systems.length" class="col-span-full grid grid-cols-subgrid">
                                <ComboboxLabel :class="label_classes"> Systems </ComboboxLabel>
                                <ComboboxItem
                                    v-for="system in matched_systems"
                                    :key="`system-${system.id}`"
                                    :value="`system-${system.id}`"
                                    :class="item_classes"
                                    @select.prevent="handleSelect(system)"
                                >
                                    <SolarsystemSearchResult
                                        :solarsystem="system.solarsystem"
                                        :alias="system.alias"
                                        :occupier_alias="system.occupier_alias"
                                    >
                                        <span></span>
                                    </SolarsystemSearchResult>
                                </ComboboxItem>
                                <ComboboxItem
                                    v-for="entry in off_map_systems"
                                    :key="`all-${entry.solarsystem.id}`"
                                    :value="`all-${entry.solarsystem.id}`"
                                    :disabled="!can_write"
                                    :class="item_classes"
                                    @select.prevent="handleAddSystem(entry.solarsystem.id)"
                                >
                                    <SolarsystemSearchResult :solarsystem="entry.solarsystem" :occupier_alias="entry.occupier_alias">
                                        <span class="flex items-center justify-end">
                                            <CommandPaletteAddBadge />
                                        </span>
                                    </SolarsystemSearchResult>
                                </ComboboxItem>
                            </ComboboxGroup>

                            <!-- Notes -->
                            <ComboboxGroup v-if="note_matches.length" class="col-span-full grid grid-cols-subgrid">
                                <ComboboxLabel :class="label_classes"> Notes </ComboboxLabel>
                                <ComboboxItem
                                    v-for="note in note_matches"
                                    :key="`note-${note.map_solarsystem_id}`"
                                    :value="`note-${note.map_solarsystem_id}`"
                                    :class="item_classes"
                                    @select.prevent="handleSelect(note.map_solarsystem)"
                                >
                                    <SolarsystemSearchResult
                                        :solarsystem="note.map_solarsystem.solarsystem"
                                        :alias="note.alias"
                                        :occupier_alias="note.occupier_alias"
                                    >
                                        <span class="min-w-0 truncate text-[10px] text-muted-foreground italic">
                                            {{ note.note_excerpt }}
                                        </span>
                                    </SolarsystemSearchResult>
                                </ComboboxItem>
                            </ComboboxGroup>

                            <!-- Threat activity -->
                            <ComboboxGroup
                                v-for="entity in threat_entities"
                                :key="`${entity.type}:${entity.id}`"
                                class="col-span-full grid grid-cols-subgrid"
                            >
                                <ComboboxLabel class="col-span-full flex items-center gap-2 px-2 py-1.5">
                                    <AllianceLogo
                                        v-if="entity.type === 'alliance'"
                                        :alliance_id="entity.id"
                                        :alliance_name="entity.name"
                                        class="size-4 rounded"
                                    />
                                    <CorporationLogo
                                        v-else-if="entity.type === 'corporation'"
                                        :corporation_id="entity.id"
                                        :corporation_name="entity.name"
                                        class="size-4 rounded"
                                    />
                                    <span class="font-mono text-[10px] tracking-wider text-muted-foreground uppercase">{{ entity.name }}</span>
                                    <span class="ml-auto font-mono text-[10px] text-muted-foreground/60">
                                        {{ entity.total_kills }} kills · {{ entity.systems_count }} systems
                                    </span>
                                </ComboboxLabel>
                                <ComboboxItem
                                    v-for="system in entity.systems"
                                    :key="`threat-${entity.type}-${entity.id}-${system.solarsystem.id}`"
                                    :value="`threat-${entity.type}-${entity.id}-${system.solarsystem.id}`"
                                    :disabled="!system.map_solarsystem && !can_write"
                                    :class="item_classes"
                                    @select.prevent="handleThreatSystemSelect(system)"
                                >
                                    <SolarsystemSearchResult
                                        :solarsystem="system.solarsystem"
                                        :alias="system.map_solarsystem?.alias ?? null"
                                        :occupier_alias="system.map_solarsystem?.occupier_alias ?? system.occupier_alias"
                                    >
                                        <span
                                            class="flex items-center justify-end gap-1 font-mono text-[10px] whitespace-nowrap text-muted-foreground"
                                        >
                                            {{ system.kills }} kills
                                            <CommandPaletteAddBadge v-if="!system.map_solarsystem" />
                                        </span>
                                    </SolarsystemSearchResult>
                                </ComboboxItem>
                            </ComboboxGroup>

                            <!-- Empty / hint -->
                            <div v-if="!hasResults" class="col-span-full p-6 text-center">
                                <p class="font-mono text-[10px] tracking-wider text-muted-foreground/60 uppercase">
                                    {{ search.trim() ? 'No results' : 'Search systems, aliases, occupiers, notes and threat activity' }}
                                </p>
                            </div>
                        </div>
                    </ComboboxContent>
                </ComboboxRoot>
            </DialogContent>
        </DialogPortal>
    </DialogRoot>
</template>

<style scoped></style>
