<script setup lang="ts">
import MapRouteSolarsystem from '@/components/autopilot/MapRouteSolarsystem.vue';
import useHasWritePermission from '@/composables/useHasWritePermission';
import { TMapRouteSolarsystem } from '@/types/models';
import { computed } from 'vue';

const { map_route_solarsystems } = defineProps<{
    map_route_solarsystems: TMapRouteSolarsystem[];
}>();

const sorted = computed(() => {
    return map_route_solarsystems?.toSorted((a, b) => {
        if (a.is_pinned && !b.is_pinned) return -1;
        if (!a.is_pinned && b.is_pinned) return 1;
        return a.solarsystem.name.localeCompare(b.solarsystem.name);
    });
});

const can_write = useHasWritePermission();
</script>

<template>
    <div
        :class="can_write ? 'grid-cols-[auto_1fr_auto_1fr_auto_auto]' : 'grid-cols-[auto_1fr_auto_1fr_auto]'"
        class="grid gap-x-4 overflow-hidden rounded border bg-white text-xs dark:bg-neutral-900/40"
    >
        <div class="col-span-full grid grid-cols-subgrid border-b bg-muted/50 px-2 py-1.5 text-xs font-medium text-muted-foreground">
            <div></div>
            <div>System</div>
            <div class="text-center">Jumps</div>
            <div>Region</div>
        </div>

        <MapRouteSolarsystem v-for="route in sorted" :key="route.solarsystem.id" :map_route="route" />

        <div v-if="!sorted?.length" class="col-span-full py-4 text-center text-muted-foreground">
            <div class="mb-1 text-sm">🎯</div>
            <div>No destinations</div>
        </div>
    </div>
</template>

<style scoped></style>
