<script setup lang="ts">
import { CharacterImage } from '@/components/images';
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
import { Circle, Compass, ExternalLink, Flag, Globe, Map, MapPin, Navigation, Plus, Route } from 'lucide-vue-next';
import { computed } from 'vue';

const { solarsystem_id } = defineProps<{
    solarsystem_id: number;
}>();

const user = useUser();

const setWaypoint = useWaypoint();

const { map_solarsystems } = useMapSolarsystems();

const { setFromSystem, setToSystem } = useNavigationSystems();

const already_on_map = computed(() => {
    return map_solarsystems.value.some((map_solarsystem) => map_solarsystem.solarsystem_id === solarsystem_id);
});

const { canEdit: can_write } = usePermission();

const { isRally, toggleRallyPoint } = useRallyPoint(() => solarsystem_id);

const staticSolarsystem = useStaticSolarsystem(() => solarsystem_id);
</script>

<template>
    <ContextMenu>
        <ContextMenuTrigger as-child>
            <slot />
        </ContextMenuTrigger>
        <ContextMenuContent>
            <ContextMenuSub>
                <ContextMenuSubTrigger>
                    <Navigation class="size-4" />
                    Set destination
                </ContextMenuSubTrigger>
                <ContextMenuSubContent>
                    <ContextMenuItem v-for="character in user.characters" :key="character.id" @select="setWaypoint(character.id, solarsystem_id)">
                        <CharacterImage :character_id="character.id" :character_name="character.name" class="size-5 rounded-lg" />
                        {{ character.name }}
                    </ContextMenuItem>
                </ContextMenuSubContent>
            </ContextMenuSub>

            <ContextMenuSub>
                <ContextMenuSubTrigger>
                    <MapPin class="size-4" />
                    Add waypoint
                </ContextMenuSubTrigger>
                <ContextMenuSubContent>
                    <ContextMenuItem
                        v-for="character in user.characters"
                        :key="character.id"
                        @select="setWaypoint(character.id, solarsystem_id, false)"
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

            <template v-if="!already_on_map && can_write">
                <ContextMenuSeparator />
                <ContextMenuItem @select="createMapSolarsystem(solarsystem_id)">
                    <Plus class="size-4" />
                    Add to map
                </ContextMenuItem>
            </template>

            <ContextMenuSeparator />

            <ContextMenuSub v-if="staticSolarsystem">
                <ContextMenuSubTrigger>
                    <ExternalLink class="size-4" />
                    External
                </ContextMenuSubTrigger>
                <ContextMenuSubContent>
                    <ContextMenuSub>
                        <ContextMenuSubTrigger>
                            <img src="https://evemaps.dotlan.net/favicon.ico" alt="" class="size-4 rounded-sm" />
                            Dotlan
                        </ContextMenuSubTrigger>
                        <ContextMenuSubContent>
                            <ContextMenuItem as-child>
                                <a
                                    :href="`https://evemaps.dotlan.net/system/${staticSolarsystem.name.replaceAll(' ', '_')}`"
                                    target="_blank"
                                    rel="noopener"
                                >
                                    <Globe class="size-4" />
                                    System
                                </a>
                            </ContextMenuItem>
                            <ContextMenuItem as-child>
                                <a
                                    :href="`https://evemaps.dotlan.net/map/${staticSolarsystem.region?.name.replaceAll(' ', '_')}/${staticSolarsystem.name.replaceAll(' ', '_')}`"
                                    target="_blank"
                                    rel="noopener"
                                >
                                    <Map class="size-4" />
                                    Region Map
                                </a>
                            </ContextMenuItem>
                            <ContextMenuItem as-child v-if="!staticSolarsystem.class">
                                <a
                                    :href="`https://evemaps.dotlan.net/range/Revelation,5/${staticSolarsystem.name.replaceAll(' ', '_')}`"
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
                            <img src="https://zkillboard.com/favicon.ico" alt="" class="size-4 rounded-sm" />
                            zKillboard
                        </ContextMenuSubTrigger>
                        <ContextMenuSubContent>
                            <ContextMenuItem as-child>
                                <a :href="`https://zkillboard.com/system/${staticSolarsystem.id}/`" target="_blank" rel="noopener">
                                    <Globe class="size-4" />
                                    System
                                </a>
                            </ContextMenuItem>
                            <ContextMenuItem as-child>
                                <a :href="`https://zkillboard.com/region/${staticSolarsystem.region_id}/`" target="_blank" rel="noopener">
                                    <Map class="size-4" />
                                    Region
                                </a>
                            </ContextMenuItem>
                        </ContextMenuSubContent>
                    </ContextMenuSub>
                </ContextMenuSubContent>
            </ContextMenuSub>

            <ContextMenuItem @select="toggleRallyPoint" v-if="can_write">
                <Flag class="size-4" />
                {{ isRally ? 'Clear Rally Point' : 'Set as Rally Point' }}
            </ContextMenuItem>
        </ContextMenuContent>
    </ContextMenu>
</template>

<style scoped></style>
