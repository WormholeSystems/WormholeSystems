<script setup lang="ts">
import Character from '@/components/characters/Character.vue';
import { CardDescription, CardHeader, CardTitle } from '@/components/ui/card';
import MapPanel from '@/components/ui/map-panel/MapPanel.vue';
import MapPanelContent from '@/components/ui/map-panel/MapPanelContent.vue';
import { TCharacter } from '@/types/models';
import { computed } from 'vue';

const { map_characters } = defineProps<{
    map_characters: TCharacter[];
}>();

const sorted_characters = computed(() => map_characters.toSorted(sortCharacters));

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
</script>

<template>
    <MapPanel>
        <CardHeader>
            <CardTitle>Characters</CardTitle>
            <CardDescription> See what characters are flying and where they are located in the map solarsystems. </CardDescription>
        </CardHeader>
        <MapPanelContent inner-class="border-0 bg-transparent">
            <div class="relative max-h-200 overflow-x-hidden overflow-y-scroll mask-b-from-90% mask-alpha pr-1">
                <div class="@container rounded-lg border bg-white dark:bg-neutral-900/40">
                    <div class="grid grid-cols-[auto_auto_auto] gap-x-2 text-xs">
                        <div class="col-span-full grid grid-cols-subgrid border-b bg-muted/50 px-2 py-1.5 text-xs font-medium text-muted-foreground">
                            <span>Pilot</span>
                            <span>Ship</span>
                            <span>Location</span>
                        </div>
                        <TransitionGroup name="list">
                            <div class="contents" v-for="character in sorted_characters" :key="character.id">
                                <Character :character />
                            </div>
                        </TransitionGroup>
                        <div v-if="!sorted_characters?.length" class="col-span-full p-2 text-center text-muted-foreground">No characters found</div>
                    </div>
                </div>
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
