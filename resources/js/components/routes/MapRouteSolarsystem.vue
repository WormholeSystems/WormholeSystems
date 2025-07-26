<script setup lang="ts">
import DestinationContextMenu from '@/components/DestinationContextMenu.vue';
import LockIcon from '@/components/icons/LockIcon.vue';
import TrashIcon from '@/components/icons/TrashIcon.vue';
import SovereigntyIcon from '@/components/map/SovereigntyIcon.vue';
import SolarsystemClass from '@/components/SolarsystemClass.vue';
import { Button } from '@/components/ui/button';
import { Popover, PopoverContent, PopoverTrigger } from '@/components/ui/popover';
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
                <Popover>
                    <PopoverTrigger as-child>
                        <Button variant="ghost">
                            {{ map_route.route.length ? map_route.route.length - 1 : 'N/A' }}
                        </Button>
                    </PopoverTrigger>
                    <PopoverContent>
                        <div class="flex flex-col gap-2">
                            <span class="text-sm text-muted-foreground">Route</span>
                            <ul class="grid divide-y text-xs">
                                <DestinationContextMenu v-for="(solarsystem, index) in map_route.route" :key="index" :solarsystem_id="solarsystem.id">
                                    <li class="col-span-4 grid grid-cols-subgrid gap-x-2 py-1 hover:bg-accent">
                                        <div class="flex justify-center">
                                            <SolarsystemClass :wormhole_class="solarsystem.class" :security="solarsystem.security" />
                                        </div>
                                        <span class="truncate">
                                            {{ solarsystem.name }}
                                        </span>
                                        <span class="truncate text-muted-foreground">
                                            {{ solarsystem.region?.name }}
                                        </span>
                                        <SovereigntyIcon v-if="solarsystem.sovereignty" :sovereignty="solarsystem.sovereignty" />
                                    </li>
                                </DestinationContextMenu>
                            </ul>
                        </div>
                    </PopoverContent>
                </Popover>
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
