<script setup lang="ts">
import {
    ContextMenu,
    ContextMenuContent,
    ContextMenuItem,
    ContextMenuSeparator,
    ContextMenuShortcut,
    ContextMenuTrigger,
} from '@/components/ui/context-menu';
import { useMapAction } from '@/composables/map';
import { TMapSolarSystem } from '@/types/models';

const { map_solarsystem } = defineProps<{
    map_solarsystem: TMapSolarSystem;
}>();

const { removeMapSolarsystem, updateMapSolarsystem } = useMapAction();

function handleTogglePin() {
    updateMapSolarsystem(map_solarsystem, { pinned: !map_solarsystem.pinned });
}

function handleRemoveFromMap() {
    removeMapSolarsystem(map_solarsystem);
}
</script>

<template>
    <ContextMenu>
        <ContextMenuTrigger as-child>
            <slot />
        </ContextMenuTrigger>
        <ContextMenuContent>
            <ContextMenuItem @select="handleTogglePin">
                {{ map_solarsystem.pinned ? 'Unpin' : 'Pin' }}
                <ContextMenuShortcut>Ctrl+P</ContextMenuShortcut>
            </ContextMenuItem>
            <template v-if="!map_solarsystem.pinned">
                <ContextMenuSeparator />
                <ContextMenuItem @select="handleRemoveFromMap">
                    Remove
                    <ContextMenuShortcut>DEL</ContextMenuShortcut>
                </ContextMenuItem>
            </template>
        </ContextMenuContent>
    </ContextMenu>
</template>

<style scoped></style>
