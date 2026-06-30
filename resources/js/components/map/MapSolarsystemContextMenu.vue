<script setup lang="ts">
import { CharacterImage } from '@/components/images';
import SolarsystemExternalLinks from '@/components/solarsystem/SolarsystemExternalLinks.vue';
import {
    ContextMenu,
    ContextMenuContent,
    ContextMenuItem,
    ContextMenuRadioGroup,
    ContextMenuRadioItem,
    ContextMenuSeparator,
    ContextMenuSub,
    ContextMenuSubContent,
    ContextMenuSubTrigger,
    ContextMenuTrigger,
} from '@/components/ui/context-menu';
import { deleteMapSolarsystem, updateMapSolarsystem, useAddConnection } from '@/composables/map';
import { useHomeSystem } from '@/composables/useHomeSystem';
import { useNavigationSystems } from '@/composables/useNavigationSystems';
import usePermission from '@/composables/usePermission';
import { useRallyPoint } from '@/composables/useRallyPoint';
import useUser from '@/composables/useUser';
import { useWaypoint } from '@/composables/useWaypoint';
import { isWormholeClass } from '@/const/solarsystemClasses';
import { TMapSolarsystem } from '@/pages/maps';
import { TMapSolarsystemStatus } from '@/types/models';
import { Compass, Flag, Home, Map, MapPin, Navigation, Pin, Route, Trash2, Users, Waypoints } from 'lucide-vue-next';
import type { AcceptableValue } from 'reka-ui';
import { computed } from 'vue';

const { map_solarsystem } = defineProps<{
    map_solarsystem: TMapSolarsystem;
}>();

const user = useUser();

// Waypoints can only be set for characters that are online in-game.
const onlineCharacters = computed(() => user.value?.characters.filter((character) => character.status?.is_online) ?? []);

const { canEdit: can_write } = usePermission();

const { isHome, toggleHomeSystem } = useHomeSystem(() => map_solarsystem.solarsystem_id);
const { isRally, toggleRallyPoint } = useRallyPoint(() => map_solarsystem.solarsystem_id);

const { setWaypoint, setWaypointAll } = useWaypoint();

const { setFromSystem, setToSystem } = useNavigationSystems();

const { openAddConnection } = useAddConnection();

function handleAddConnection() {
    openAddConnection(map_solarsystem);
}

function handleTogglePin() {
    updateMapSolarsystem(map_solarsystem, { pinned: !map_solarsystem.pinned });
}

function handleSetDestinationAll() {
    setWaypointAll(map_solarsystem.solarsystem_id);
}

function handleRemoveFromMap() {
    deleteMapSolarsystem(map_solarsystem);
}

function handleStatusChange(status: AcceptableValue) {
    updateMapSolarsystem(map_solarsystem, { status: status as string });
}

const options: TMapSolarsystemStatus[] = ['unknown', 'friendly', 'hostile', 'active', 'unscanned', 'empty'];
</script>

<template>
    <ContextMenu>
        <ContextMenuTrigger>
            <slot />
        </ContextMenuTrigger>
        <ContextMenuContent>
            <ContextMenuItem @select="handleTogglePin" v-if="can_write">
                <Pin class="size-4" />
                {{ map_solarsystem.pinned ? 'Unpin' : 'Pin' }}
            </ContextMenuItem>
            <ContextMenuItem @select="handleAddConnection" v-if="can_write">
                <Waypoints class="size-4" />
                Add connection
            </ContextMenuItem>
            <ContextMenuSub v-if="can_write">
                <ContextMenuSubTrigger>
                    <Map class="size-4" />
                    Status
                </ContextMenuSubTrigger>
                <ContextMenuSubContent>
                    <ContextMenuRadioGroup :model-value="map_solarsystem.status ?? undefined" @update:model-value="handleStatusChange">
                        <ContextMenuRadioItem v-for="option in options" :key="option" :value="option">
                            {{ option.charAt(0).toUpperCase() + option.slice(1) }}
                            <span
                                :data-status="option"
                                class="ml-auto inline-block size-2 rounded-full data-[status=active]:bg-active data-[status=empty]:bg-empty data-[status=friendly]:bg-friendly data-[status=hostile]:bg-hostile data-[status=unknown]:bg-unknown data-[status=unscanned]:bg-unscanned"
                            />
                        </ContextMenuRadioItem>
                    </ContextMenuRadioGroup>
                </ContextMenuSubContent>
            </ContextMenuSub>

            <ContextMenuSeparator />

            <SolarsystemExternalLinks :solarsystem="map_solarsystem.solarsystem" />
            <ContextMenuSub v-if="user && !isWormholeClass(map_solarsystem.solarsystem.class)">
                <ContextMenuSubTrigger>
                    <Navigation class="size-4" />
                    Set destination
                </ContextMenuSubTrigger>
                <ContextMenuSubContent>
                    <template v-if="onlineCharacters.length">
                        <ContextMenuItem
                            v-for="character in onlineCharacters"
                            :key="character.id"
                            @select="setWaypoint(character.id, map_solarsystem.solarsystem_id)"
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

            <ContextMenuSub v-if="user && !isWormholeClass(map_solarsystem.solarsystem.class)">
                <ContextMenuSubTrigger>
                    <MapPin class="size-4" />
                    Add waypoint
                </ContextMenuSubTrigger>
                <ContextMenuSubContent>
                    <template v-if="onlineCharacters.length">
                        <ContextMenuItem
                            v-for="character in onlineCharacters"
                            :key="character.id"
                            @select="setWaypoint(character.id, map_solarsystem.solarsystem_id, false)"
                        >
                            <CharacterImage :character_id="character.id" :character_name="character.name" class="size-5 rounded-lg" />
                            {{ character.name }}
                        </ContextMenuItem>
                        <ContextMenuSeparator v-if="onlineCharacters.length > 1" />
                        <ContextMenuItem v-if="onlineCharacters.length > 1" @select="setWaypointAll(map_solarsystem.solarsystem_id, false)">
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
                    <ContextMenuItem @select="setFromSystem(map_solarsystem.solarsystem_id)">
                        <Compass class="size-4" />
                        Set as origin
                    </ContextMenuItem>
                    <ContextMenuItem @select="setToSystem(map_solarsystem.solarsystem_id)">
                        <Navigation class="size-4" />
                        Set as destination
                    </ContextMenuItem>
                </ContextMenuSubContent>
            </ContextMenuSub>

            <ContextMenuSeparator />

            <ContextMenuItem @select="toggleHomeSystem" v-if="can_write">
                <Home class="size-4" />
                {{ isHome ? 'Unset Home System' : 'Set as Home System' }}
            </ContextMenuItem>
            <ContextMenuItem @select="toggleRallyPoint" v-if="can_write">
                <Flag class="size-4" />
                {{ isRally ? 'Clear Rally Point' : 'Set as Rally Point' }}
            </ContextMenuItem>

            <template v-if="!map_solarsystem.pinned && !isHome && can_write">
                <ContextMenuSeparator />
                <ContextMenuItem @select="handleRemoveFromMap" class="text-destructive focus:text-destructive">
                    <Trash2 class="size-4" />
                    Remove
                </ContextMenuItem>
            </template>
        </ContextMenuContent>
    </ContextMenu>
</template>

<style scoped></style>
