<script setup lang="ts">
import { ContextMenuContent, ContextMenuItem } from '@/components/ui/context-menu';
import { useMapAction } from '@/composables/map';

const { removeAllMapSolarsystems, sortMapSolarsystemsByRegion, organizeMapSolarsystems } = useMapAction();

function handleSystemDelete() {
    if (!confirm('Are you sure you want to clear the map? This will remove all solarsystems.')) {
        return;
    }
    removeAllMapSolarsystems();
}
</script>

<template>
    <ContextMenuContent>
        <ContextMenuItem @select="handleSystemDelete"> Clear map</ContextMenuItem>
        <ContextMenuItem @select="sortMapSolarsystemsByRegion"> Sort by region</ContextMenuItem>
        <ContextMenuItem @select="organizeMapSolarsystems"> Organize solarsystems</ContextMenuItem>
    </ContextMenuContent>
</template>

<style scoped></style>
