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
import { useMapAction, useMapSolarsystems } from '@/composables/map';
import { useHasWritePermission } from '@/composables/useHasPermission';
import useUser from '@/composables/useUser';
import { useWaypoint } from '@/composables/useWaypoint';
import { computed } from 'vue';

const { solarsystem_id } = defineProps<{
    solarsystem_id: number;
}>();

const user = useUser();

const setWaypoint = useWaypoint();

const { map_solarsystems } = useMapSolarsystems();

const { addMapSolarsystem } = useMapAction();

const already_on_map = computed(() => {
    return map_solarsystems.value.some((map_solarsystem) => map_solarsystem.solarsystem_id === solarsystem_id);
});

const can_write = useHasWritePermission();
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
                <ContextMenuItem @select="addMapSolarsystem(solarsystem_id)">Add to map</ContextMenuItem>
            </template>
        </ContextMenuContent>
    </ContextMenu>
</template>

<style scoped></style>
