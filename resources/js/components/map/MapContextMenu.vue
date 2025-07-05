<script setup lang="ts">
import { ContextMenuContent, ContextMenuItem } from '@/components/ui/context-menu';
import { useMapAction } from '@/composables/map';

const { removeAllMapSolarsystems, sortMapSolarsystemsByRegion } = useMapAction();

function handleSystemDelete() {
    removeAllMapSolarsystems();
}
</script>

<template>
    <ContextMenuContent>
        <ContextMenuItem @select="handleSystemDelete"> Clear map</ContextMenuItem>
        <ContextMenuItem @select="sortMapSolarsystemsByRegion"> Sort by region</ContextMenuItem>
    </ContextMenuContent>
</template>

<style scoped></style>
