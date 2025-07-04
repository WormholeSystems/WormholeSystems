<script setup lang="ts">
import { ContextMenu, ContextMenuContent, ContextMenuItem, ContextMenuShortcut, ContextMenuTrigger } from '@/components/ui/context-menu';
import { TMapSolarSystem } from '@/types/models';
import { router } from '@inertiajs/vue3';

const { map_solarsystem } = defineProps<{
    map_solarsystem: TMapSolarSystem;
}>();

function handleRemoveFromMap() {
    router.delete(route('map-solarsystems.destroy', map_solarsystem.id));
}
</script>

<template>
    <ContextMenu>
        <ContextMenuTrigger as-child>
            <slot />
        </ContextMenuTrigger>
        <ContextMenuContent>
            <ContextMenuItem @select="handleRemoveFromMap">
                Remove
                <ContextMenuShortcut>DEL</ContextMenuShortcut>
            </ContextMenuItem>
        </ContextMenuContent>
    </ContextMenu>
</template>

<style scoped></style>
