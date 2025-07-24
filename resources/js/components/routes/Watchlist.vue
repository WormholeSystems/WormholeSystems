<script setup lang="ts">
import Spinner from '@/components/icons/Spinner.vue';
import MapRouteSolarsystem from '@/components/routes/MapRouteSolarsystem.vue';
import MapRouteSolarsystemAdd from '@/components/routes/MapRouteSolarsystemAdd.vue';
import { Card, CardAction, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card';
import { Table, TableBody, TableHead, TableHeader, TableRow } from '@/components/ui/table';
import { useHasWritePermission } from '@/composables/useHasPermission';
import { TMap, TMapRouteSolarsystem, TMapSolarSystem, TSolarsystem } from '@/types/models';
import { Deferred } from '@inertiajs/vue3';
import { computed } from 'vue';

const { map_route_solarsystems, map, solarsystems, selected_map_solarsystem } = defineProps<{
    map: TMap;
    solarsystems: TSolarsystem[];
    map_route_solarsystems?: TMapRouteSolarsystem[];
    selected_map_solarsystem?: TMapSolarSystem;
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
    <Card class="pb-0">
        <CardHeader>
            <CardTitle>Watchlist</CardTitle>
            <CardDescription> Lists the number of jumps from {{ selected_map_solarsystem?.name }}</CardDescription>
            <CardAction v-if="can_write">
                <MapRouteSolarsystemAdd :map :solarsystems :map_route_solarsystems />
            </CardAction>
        </CardHeader>
        <CardContent class="p-1 py-1">
            <Deferred data="map_route_solarsystems">
                <template #fallback>
                    <span class="flex items-center gap-2 text-xs text-muted-foreground">
                        <Spinner class="animate-spin" />
                        <span class="animate-pulse"> Loading route distances </span>
                    </span>
                </template>
                <div class="rounded-lg border bg-neutral-900/40">
                    <Table>
                        <TableHeader>
                            <TableRow>
                                <TableHead> Name</TableHead>
                                <TableHead> Jumps</TableHead>
                            </TableRow>
                        </TableHeader>
                        <TableBody>
                            <MapRouteSolarsystem v-for="route in sorted" :key="route.solarsystem.id" :map_route="route" />
                        </TableBody>
                    </Table>
                </div>
            </Deferred>
        </CardContent>
    </Card>
</template>

<style scoped></style>
