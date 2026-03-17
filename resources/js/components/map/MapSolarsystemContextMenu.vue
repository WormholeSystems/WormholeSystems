<script setup lang="ts">
import { CharacterImage } from '@/components/images';
import CopyNameMenu from '@/components/map/CopyNameMenu.vue';
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
import { deleteMapSolarsystem, updateMapSolarsystem } from '@/composables/map';
import { useNavigationSystems } from '@/composables/useNavigationSystems';
import usePermission from '@/composables/usePermission';
import useUser from '@/composables/useUser';
import { useWaypoint } from '@/composables/useWaypoint';
import { TMapSolarsystem } from '@/pages/maps';
import { TMapSolarsystemStatus } from '@/types/models';
import { Circle, Compass, ExternalLink, Globe, Map, MapPin, Navigation, Pin, Route, Trash2 } from 'lucide-vue-next';
import type { AcceptableValue } from 'reka-ui';

const { map_solarsystem } = defineProps<{
    map_solarsystem: TMapSolarsystem;
}>();

const user = useUser();

const { canEdit: can_write } = usePermission();

const setWaypoint = useWaypoint();

const { setFromSystem, setToSystem } = useNavigationSystems();

function handleTogglePin() {
    updateMapSolarsystem(map_solarsystem, { pinned: !map_solarsystem.pinned });
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

            <ContextMenuSub>
                <ContextMenuSubTrigger>
                    <ExternalLink class="size-4" />
                    External
                </ContextMenuSubTrigger>
                <ContextMenuSubContent>
                    <ContextMenuSub>
                        <ContextMenuSubTrigger>
                            <ExternalLink class="size-4" />
                            Dotlan
                        </ContextMenuSubTrigger>
                        <ContextMenuSubContent>
                            <ContextMenuItem as-child>
                                <a
                                    :href="`https://evemaps.dotlan.net/system/${map_solarsystem.solarsystem?.name.replaceAll(' ', '_')}`"
                                    target="_blank"
                                    rel="noopener"
                                >
                                    <Globe class="size-4" />
                                    System
                                </a>
                            </ContextMenuItem>
                            <ContextMenuItem as-child>
                                <a
                                    :href="`https://evemaps.dotlan.net/map/${map_solarsystem.solarsystem?.region?.name.replaceAll(' ', '_')}/${map_solarsystem.solarsystem?.name.replaceAll(' ', '_')}`"
                                    target="_blank"
                                    rel="noopener"
                                >
                                    <Map class="size-4" />
                                    Region Map
                                </a>
                            </ContextMenuItem>
                            <ContextMenuItem as-child v-if="!map_solarsystem.solarsystem.class">
                                <a
                                    :href="`https://evemaps.dotlan.net/range/Revelation,5/${map_solarsystem.solarsystem?.name.replaceAll(' ', '_')}`"
                                    target="_blank"
                                    rel="noopener"
                                >
                                    <Circle class="size-4" />
                                    Jump Range
                                </a>
                            </ContextMenuItem>
                        </ContextMenuSubContent>
                    </ContextMenuSub>
                    <ContextMenuSub>
                        <ContextMenuSubTrigger>
                            <ExternalLink class="size-4" />
                            zKillboard
                        </ContextMenuSubTrigger>
                        <ContextMenuSubContent>
                            <ContextMenuItem as-child>
                                <a :href="`https://zkillboard.com/system/${map_solarsystem.solarsystem?.id}/`" target="_blank" rel="noopener">
                                    <Globe class="size-4" />
                                    System
                                </a>
                            </ContextMenuItem>
                            <ContextMenuItem as-child>
                                <a :href="`https://zkillboard.com/region/${map_solarsystem.solarsystem?.region_id}/`" target="_blank" rel="noopener">
                                    <Map class="size-4" />
                                    Region
                                </a>
                            </ContextMenuItem>
                        </ContextMenuSubContent>
                    </ContextMenuSub>
                </ContextMenuSubContent>
            </ContextMenuSub>
            <ContextMenuSub v-if="user && !map_solarsystem.solarsystem.class">
                <ContextMenuSubTrigger>
                    <Navigation class="size-4" />
                    Set destination
                </ContextMenuSubTrigger>
                <ContextMenuSubContent>
                    <ContextMenuItem
                        v-for="character in user.characters"
                        :key="character.id"
                        @select="setWaypoint(character.id, map_solarsystem.solarsystem_id)"
                    >
                        <CharacterImage :character_id="character.id" :character_name="character.name" class="size-5 rounded-lg" />
                        {{ character.name }}
                    </ContextMenuItem>
                </ContextMenuSubContent>
            </ContextMenuSub>

            <ContextMenuSub v-if="user && !map_solarsystem.solarsystem.class">
                <ContextMenuSubTrigger>
                    <MapPin class="size-4" />
                    Add waypoint
                </ContextMenuSubTrigger>
                <ContextMenuSubContent>
                    <ContextMenuItem
                        v-for="character in user.characters"
                        :key="character.id"
                        @select="setWaypoint(character.id, map_solarsystem.solarsystem_id, false)"
                    >
                        <CharacterImage :character_id="character.id" :character_name="character.name" class="size-5 rounded-lg" />
                        {{ character.name }}
                    </ContextMenuItem>
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

            <CopyNameMenu :map_solarsystem="map_solarsystem" />

            <template v-if="!map_solarsystem.pinned && can_write">
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
