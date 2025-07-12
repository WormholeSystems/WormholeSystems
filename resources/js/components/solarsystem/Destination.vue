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
import { usePath } from '@/composables/usePath';
import useUser from '@/composables/useUser';
import { useWaypoint } from '@/composables/useWaypoint';
import { TDestination } from '@/types/models';
import { useElementHover } from '@vueuse/core';
import { computed, useTemplateRef, watch } from 'vue';

const { destination } = defineProps<{
    destination: TDestination;
}>();

const { setPath } = usePath();

const element = useTemplateRef('element');

const hovered = useElementHover(element);

const user = useUser();

const setWaypoint = useWaypoint();

watch(hovered, (isHovered) => {
    setPath(isHovered ? destination.route : null);
});

const distance = computed(() => {
    const jumps = destination.route.length - 1;
    if (jumps < 0) return 'none';
    if (jumps > 10) return 'far';
    if (jumps > 5) return 'near';
    return 'short';
});
</script>

<template>
    <ContextMenu>
        <ContextMenuTrigger>
            <div class="flex items-center gap-1" ref="element">
                <span class="text-xs text-muted-foreground">
                    {{ destination.destination.name }}
                </span>
                <span
                    v-if="destination.route.length"
                    :data-distance="distance"
                    class="text-xs text-muted-foreground data-[distance=far]:text-red-500 data-[distance=near]:text-yellow-500 data-[distance=none]:text-neutral-500 data-[distance=short]:text-green-500"
                    >{{ destination.route.length - 1 }}</span
                >
                <span v-else class="text-xs text-muted-foreground">N/A</span>
            </div>
        </ContextMenuTrigger>
        <ContextMenuContent>
            <ContextMenuSub>
                <ContextMenuSubTrigger>Set destination</ContextMenuSubTrigger>
                <ContextMenuSubContent>
                    <ContextMenuItem
                        v-for="character in user.characters"
                        :key="character.id"
                        @select="setWaypoint(character.id, destination.destination.id)"
                    >
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
                        @select="setWaypoint(character.id, destination.destination.id, false)"
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
