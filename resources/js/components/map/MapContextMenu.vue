<script setup lang="ts">
import { ContextMenuContent, ContextMenuItem } from '@/components/ui/context-menu';
import { useMapSolarsystems } from '@/composables/map';
import { router } from '@inertiajs/vue3';

const { map_solarsystems } = useMapSolarsystems();

function handleSystemDelete() {
    router.delete(route('map-selection.destroy'), {
        data: {
            map_solarsystem_ids: map_solarsystems.value.map((map_solarsystem) => map_solarsystem.id),
        },
    });
}
</script>

<template>
    <ContextMenuContent>
        <ContextMenuItem @select="handleSystemDelete"> Clear map</ContextMenuItem>
    </ContextMenuContent>
</template>

<style scoped></style>
