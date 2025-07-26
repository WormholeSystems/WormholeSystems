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
import useUser from '@/composables/useUser';
import { useWaypoint } from '@/composables/useWaypoint';

defineProps<{
    solarsystem_id: number;
}>();

const user = useUser();

const setWaypoint = useWaypoint();
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
            <ContextMenuSeparator />
        </ContextMenuContent>
    </ContextMenu>
</template>

<style scoped></style>
