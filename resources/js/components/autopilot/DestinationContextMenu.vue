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
import useUser from '@/composables/useUser';
import { useWaypoint } from '@/composables/useWaypoint';
import { Compass, MapPin, Navigation, Plus, Route } from 'lucide-vue-next';
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

        </ContextMenuContent>
    </ContextMenu>
</template>

<style scoped></style>
