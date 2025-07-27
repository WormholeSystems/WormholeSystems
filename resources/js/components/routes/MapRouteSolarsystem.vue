<script setup lang="ts">
import DestinationContextMenu from '@/components/DestinationContextMenu.vue';
import LockIcon from '@/components/icons/LockIcon.vue';
import TrashIcon from '@/components/icons/TrashIcon.vue';
import RoutePopover from '@/components/routes/RoutePopover.vue';
import SolarsystemClass from '@/components/SolarsystemClass.vue';
import { Button } from '@/components/ui/button';
import { TableCell, TableRow } from '@/components/ui/table';
import { useHasWritePermission } from '@/composables/useHasPermission';
import { usePath } from '@/composables/usePath';
import { TMapRouteSolarsystem } from '@/types/models';
import { router } from '@inertiajs/vue3';
import { vElementHover } from '@vueuse/components';

const { map_route } = defineProps<{
    map_route: TMapRouteSolarsystem;
}>();

const { setPath } = usePath();

const can_write = useHasWritePermission();

function onHover(route: TMapRouteSolarsystem, hovered: boolean) {
    if (hovered) {
        setPath(route.route);
    } else {
        setPath(null);
    }
}

function togglePinned() {
    router.put(
        route('map-route-solarsystems.update', map_route.id),
        {
            is_pinned: !map_route.is_pinned,
        },
        {
            preserveScroll: true,
            preserveState: true,
            only: ['map_route_solarsystems'],
        },
    );
}

function removeRoute() {
    router.delete(route('map-route-solarsystems.destroy', map_route.id), {
        preserveScroll: true,
        preserveState: true,
        only: ['map_route_solarsystems'],
    });
}
</script>

<template>
    <DestinationContextMenu :solarsystem_id="map_route.solarsystem.id">
        <TableRow v-element-hover="(hovered) => onHover(map_route, hovered)" class="group">
            <TableCell>
                <SolarsystemClass :wormhole_class="map_route.solarsystem.class" :security="map_route.solarsystem.security" />
                {{ map_route.solarsystem.name }}
            </TableCell>
            <TableCell>
                <RoutePopover :route="map_route.route">
                    <Button variant="ghost">
                        {{ map_route.route.length ? map_route.route.length - 1 : 'N/A' }}
                    </Button>
                </RoutePopover>
            </TableCell>
            <TableCell v-if="can_write">
                <Button
                    variant="ghost"
                    size="icon"
                    @click="togglePinned"
                    :data-pinned="map_route.is_pinned"
                    class="data-[pinned=false]:opacity-0 group-hover:data-[pinned=false]:opacity-100"
                >
                    <span class="sr-only">Toggle Pin</span>
                    <LockIcon />
                </Button>
                <Button variant="ghost" size="icon" @click="removeRoute" class="opacity-0 group-hover:opacity-100">
                    <span class="sr-only">Remove Route</span>
                    <TrashIcon />
                </Button>
            </TableCell>
        </TableRow>
    </DestinationContextMenu>
</template>

<style scoped></style>
