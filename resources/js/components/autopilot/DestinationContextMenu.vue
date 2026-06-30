<script setup lang="ts">
import { CharacterImage } from '@/components/images';
import SolarsystemExternalLinks from '@/components/solarsystem/SolarsystemExternalLinks.vue';
import {
    ContextMenu,
    ContextMenuContent,
    ContextMenuItem,
    ContextMenuSeparator,
    ContextMenuSub,
    ContextMenuSubContent,
    ContextMenuSubTrigger,
    ContextMenuTrigger,
} from '@/components/ui/context-menu';
import { createMapSolarsystem, useMapSolarsystems } from '@/composables/map';
import { useNavigationSystems } from '@/composables/useNavigationSystems';
import usePermission from '@/composables/usePermission';
import { useRallyPoint } from '@/composables/useRallyPoint';
import { useStaticSolarsystem } from '@/composables/useStaticSolarsystems';
import useUser from '@/composables/useUser';
import { useWaypoint } from '@/composables/useWaypoint';
import { Compass, Flag, MapPin, Navigation, Plus, Route, Users } from 'lucide-vue-next';
import { computed } from 'vue';

const { solarsystem_id } = defineProps<{
    solarsystem_id: number;
}>();

const user = useUser();

// Waypoints can only be set for characters that are online in-game, so the
// destination/waypoint menus only offer online characters.
const onlineCharacters = computed(() => user.value?.characters.filter((character) => character.status?.is_online) ?? []);

const { setWaypoint, setWaypointAll } = useWaypoint();

const { map_solarsystems } = useMapSolarsystems();

const { setFromSystem, setToSystem } = useNavigationSystems();

const already_on_map = computed(() => {
    return map_solarsystems.value.some((map_solarsystem) => map_solarsystem.solarsystem_id === solarsystem_id);
});

const { canEdit: can_write } = usePermission();

const { isRally, toggleRallyPoint } = useRallyPoint(() => solarsystem_id);

function handleSetDestinationAll() {
    setWaypointAll(solarsystem_id);
}

const staticSolarsystem = useStaticSolarsystem(() => solarsystem_id);
</script>

<template>
    <ContextMenu>
        <ContextMenuTrigger as-child>
            <slot />
        </ContextMenuTrigger>
        <ContextMenuContent>
            <template v-if="!already_on_map && can_write">
                <ContextMenuItem @select="createMapSolarsystem(solarsystem_id)">
                    <Plus class="size-4" />
                    Add to map
                </ContextMenuItem>
                <ContextMenuSeparator />
            </template>

            <SolarsystemExternalLinks v-if="staticSolarsystem" :solarsystem="staticSolarsystem" />

            <ContextMenuSub>
                <ContextMenuSubTrigger>
                    <Navigation class="size-4" />
                    Set destination
                </ContextMenuSubTrigger>
                <ContextMenuSubContent>
                    <template v-if="onlineCharacters.length">
                        <ContextMenuItem
                            v-for="character in onlineCharacters"
                            :key="character.id"
                            @select="setWaypoint(character.id, solarsystem_id)"
                        >
                            <CharacterImage :character_id="character.id" :character_name="character.name" class="size-5 rounded-lg" />
                            {{ character.name }}
                        </ContextMenuItem>
                        <ContextMenuSeparator v-if="onlineCharacters.length > 1" />
                        <ContextMenuItem v-if="onlineCharacters.length > 1" @select="handleSetDestinationAll">
                            <Users class="size-4" />
                            All Characters
                        </ContextMenuItem>
                    </template>
                    <ContextMenuItem v-else disabled>No characters online</ContextMenuItem>
                </ContextMenuSubContent>
            </ContextMenuSub>

            <ContextMenuSub>
                <ContextMenuSubTrigger>
                    <MapPin class="size-4" />
                    Add waypoint
                </ContextMenuSubTrigger>
                <ContextMenuSubContent>
                    <template v-if="onlineCharacters.length">
                        <ContextMenuItem
                            v-for="character in onlineCharacters"
                            :key="character.id"
                            @select="setWaypoint(character.id, solarsystem_id, false)"
                        >
                            <CharacterImage :character_id="character.id" :character_name="character.name" class="size-5 rounded-lg" />
                            {{ character.name }}
                        </ContextMenuItem>
                        <ContextMenuSeparator v-if="onlineCharacters.length > 1" />
                        <ContextMenuItem v-if="onlineCharacters.length > 1" @select="setWaypointAll(solarsystem_id, false)">
                            <Users class="size-4" />
                            All Characters
                        </ContextMenuItem>
                    </template>
                    <ContextMenuItem v-else disabled>No characters online</ContextMenuItem>
                </ContextMenuSubContent>
            </ContextMenuSub>

            <ContextMenuSub>
                <ContextMenuSubTrigger>
                    <Route class="size-4" />
                    Route planner
                </ContextMenuSubTrigger>
                <ContextMenuSubContent>
                    <ContextMenuItem @select="setFromSystem(solarsystem_id)">
                        <Compass class="size-4" />
                        Set as origin
                    </ContextMenuItem>
                    <ContextMenuItem @select="setToSystem(solarsystem_id)">
                        <Navigation class="size-4" />
                        Set as destination
                    </ContextMenuItem>
                </ContextMenuSubContent>
            </ContextMenuSub>

            <ContextMenuSeparator />

            <ContextMenuItem @select="toggleRallyPoint" v-if="can_write">
                <Flag class="size-4" />
                {{ isRally ? 'Clear Rally Point' : 'Set as Rally Point' }}
            </ContextMenuItem>
        </ContextMenuContent>
    </ContextMenu>
</template>

<style scoped></style>
