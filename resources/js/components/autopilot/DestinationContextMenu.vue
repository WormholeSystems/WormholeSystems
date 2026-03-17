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
                <ContextMenuSubTrigger>Set destination</ContextMenuSubTrigger>
                <ContextMenuSubContent>
                    <ContextMenuItem v-for="character in user.characters" :key="character.id" @select="setWaypoint(character.id, solarsystem_id)">
                        <CharacterImage :character_id="character.id" :character_name="character.name" class="size-5 rounded-lg" />
                        {{ character.name }}
                    </ContextMenuItem>
                </ContextMenuSubContent>
            </ContextMenuSub>

            <ContextMenuSub>
                <ContextMenuSubTrigger>Add waypoint</ContextMenuSubTrigger>
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
            <template v-if="!already_on_map && can_write">
                <ContextMenuSeparator />
                <ContextMenuItem @select="createMapSolarsystem(solarsystem_id)">Add to map</ContextMenuItem>
            </template>

            <ContextMenuSeparator />

            <ContextMenuSub>
                <ContextMenuSubTrigger>Route planner</ContextMenuSubTrigger>
                <ContextMenuSubContent>
                    <ContextMenuItem @select="setFromSystem(solarsystem_id)">Set as origin</ContextMenuItem>
                    <ContextMenuItem @select="setToSystem(solarsystem_id)">Set as destination</ContextMenuItem>
                </ContextMenuSubContent>
            </ContextMenuSub>
        </ContextMenuContent>
    </ContextMenu>
</template>

<style scoped></style>
