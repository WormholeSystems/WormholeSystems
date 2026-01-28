<script setup lang="ts">
import Character from '@/components/characters/Character.vue';
import MapPanel from '@/components/ui/map-panel/MapPanel.vue';
import MapPanelContent from '@/components/ui/map-panel/MapPanelContent.vue';
import MapPanelHeader from '@/components/ui/map-panel/MapPanelHeader.vue';
import { useActiveMapCharacter } from '@/composables/useActiveMapCharacter';
import { useJumpCounts } from '@/composables/useJumpCounts';
import { useStaticSolarsystems } from '@/composables/useStaticSolarsystems';
import type { TMap, TResolvedSelectedMapSolarsystem, TResolvedSolarsystem } from '@/pages/maps';
import { TCharacter } from '@/types/models';
import { computed } from 'vue';

const { map_characters, map, selected_map_solarsystem, ignored_systems } = defineProps<{
    map_characters: TCharacter[];
    map: TMap;
    selected_map_solarsystem: TResolvedSelectedMapSolarsystem | null;
    ignored_systems: number[];
}>();

const { resolveSolarsystem } = useStaticSolarsystems();
const activeCharacter = useActiveMapCharacter();

const originSolarsystemId = computed(() => {
    return activeCharacter.value?.status?.solarsystem_id ?? selected_map_solarsystem?.solarsystem_id ?? null;
});

const targetIds = computed(() => {
    const ids = new Set<number>();

    for (const character of map_characters) {
        const targetId = character.status?.solarsystem_id;
        if (targetId) {
            ids.add(targetId);
        }
    }

    return [...ids];
});

const { routesByTarget } = useJumpCounts({
    fromId: originSolarsystemId,
    targets: targetIds,
    mapConnections: computed(() => map.map_connections ?? []),
    mapSolarsystems: computed(() => map.map_solarsystems ?? []),
    ignoredSystems: computed(() => ignored_systems ?? []),
});

const resolved_characters = computed(() =>
    map_characters.map((character) => ({
        ...character,
        route: resolveRoute(character.status?.solarsystem_id ?? null),
    })),
);

const sorted_characters = computed(() => resolved_characters.value.toSorted(sortCharacters));

function sortCharacters(a: TCharacter, b: TCharacter) {
    if (a.status?.ship_type?.name === 'Capsule' && b.status?.ship_type?.name !== 'Capsule') {
        return 1;
    }
    if (b.status?.ship_type?.name === 'Capsule' && a.status?.ship_type?.name !== 'Capsule') {
        return -1;
    }

    if ((a.status?.station_id || a.status?.structure_id) && !(b.status?.station_id || b.status?.structure_id)) {
        return 1;
    }

    if (!(a.status?.station_id || a.status?.structure_id) && (b.status?.station_id || b.status?.structure_id)) {
        return -1;
    }

    if (isCovertOps(a) && !isCovertOps(b)) {
        return 1;
    }

    if (!isCovertOps(a) && isCovertOps(b)) {
        return -1;
    }

    return a.name.localeCompare(b.name);
}

function isCovertOps(character: TCharacter) {
    return character.status?.ship_type?.group_id === 830;
}

function resolveRoute(targetId: number | null): TResolvedSolarsystem[] {
    if (!targetId) {
        return [];
    }

    const routeResult = routesByTarget.value.get(targetId);

    if (!routeResult) {
        return [];
    }

    return routeResult.route
        .map<TResolvedSolarsystem | null>((step, index) => {
            const solarsystem = resolveSolarsystem(step.id);

            if (!solarsystem) {
                return null;
            }

            return {
                ...solarsystem,
                connection_type: routeResult.route[index + 1]?.via ?? null,
            };
        })
        .filter((entry): entry is TResolvedSolarsystem => entry !== null);
}
</script>

<template>
    <MapPanel>
        <MapPanelHeader>
            <span class="inline-flex items-center gap-2">
                <span class="size-2 animate-pulse rounded-full bg-green-500" />
                Pilots
            </span>
            <span class="ml-1 font-mono text-amber-400">{{ sorted_characters.length }}</span>
        </MapPanelHeader>
        <MapPanelContent>
            <template v-if="sorted_characters?.length">
                <TransitionGroup name="list">
                    <div v-for="character in sorted_characters" :key="character.id">
                        <Character :character />
                    </div>
                </TransitionGroup>
            </template>
            <div v-else class="flex h-full flex-col items-center justify-center gap-2 p-4">
                <p class="font-mono text-[10px] tracking-wider text-muted-foreground/60 uppercase">No pilots online</p>
            </div>
        </MapPanelContent>
    </MapPanel>
</template>

<style scoped>
.list-move, /* apply transition to moving elements */
.list-enter-active,
.list-leave-active {
    transition: all 0.5s ease;
}

.list-enter-from,
.list-leave-to {
    opacity: 0;
    transform: translateX(30px);
}

/* ensure leaving items are taken out of layout flow so that moving
   animations can be calculated correctly. */
.list-leave-active {
    position: absolute;
    opacity: 0;
    transition-duration: 0ms;
}
</style>
