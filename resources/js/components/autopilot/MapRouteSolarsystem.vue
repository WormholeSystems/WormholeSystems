<script setup lang="ts">
import DestinationContextMenu from '@/components/autopilot/DestinationContextMenu.vue';
import RoutePopover from '@/components/autopilot/RoutePopover.vue';
import PinIcon from '@/components/icons/PinIcon.vue';
import TrashIcon from '@/components/icons/TrashIcon.vue';
import SolarsystemSovereignty from '@/components/map/SolarsystemSovereignty.vue';
import SolarsystemClass from '@/components/solarsystem/SolarsystemClass.vue';
import { Button } from '@/components/ui/button';
import { Tooltip, TooltipContent, TooltipTrigger } from '@/components/ui/tooltip';
import useHasWritePermission from '@/composables/useHasWritePermission';
import { usePath } from '@/composables/usePath';
import type { TResolvedMapRouteSolarsystem } from '@/pages/maps';
import MapRouteSolarsystems from '@/routes/map-route-solarsystems';
import { router } from '@inertiajs/vue3';
import { vElementHover } from '@vueuse/components';

const { map_route } = defineProps<{
    map_route: TResolvedMapRouteSolarsystem;
}>();

const { setPath } = usePath();

const can_write = useHasWritePermission();

function onHover(hovered: boolean) {
    if (hovered) {
        const routeToShow = map_route.route ?? [];
        setPath(routeToShow);
    } else {
        setPath(null);
    }
}

function togglePinned() {
    router.put(
        MapRouteSolarsystems.update(map_route.id).url,
        {
            is_pinned: !map_route.is_pinned,
        },
        {
            preserveScroll: true,
            preserveState: true,
            only: ['map_navigation'],
        },
    );
}

function removeRoute() {
    router.delete(MapRouteSolarsystems.destroy(map_route.id).url, {
        preserveScroll: true,
        preserveState: true,
        only: ['map_navigation'],
    });
}
</script>

<template>
    <DestinationContextMenu :solarsystem_id="map_route.solarsystem.id">
        <div
            class="group col-span-full grid grid-cols-subgrid items-center border-b py-1 *:first:pl-2 last:border-b-0 *:last:pr-2 hover:bg-neutral-50 dark:hover:bg-neutral-800/30"
            v-element-hover="onHover"
        >
            <div class="flex items-center justify-center">
                <SolarsystemClass :wormhole_class="map_route.solarsystem.class" :security="map_route.solarsystem.security" />
            </div>

            <span class="truncate font-medium">
                {{ map_route.solarsystem.name }}
            </span>

            <RoutePopover :route="map_route.route">
                <Button variant="secondary" size="sm" class="font-mono">
                    <span v-if="map_route.route && map_route.route.length > 0">
                        {{ map_route.route.length - 1 }}
                    </span>
                    <span v-else>∞</span>
                </Button>
            </RoutePopover>

            <span class="truncate text-muted-foreground">
                {{ map_route.solarsystem.region?.name || '' }}
            </span>

            <SolarsystemSovereignty :sovereignty="map_route.solarsystem.sovereignty" :solarsystem-id="map_route.solarsystem.id" class="size-6" />

            <div v-if="can_write" class="flex justify-center gap-2">
                <Tooltip>
                    <TooltipTrigger as-child>
                        <Button
                            :data-pinned="map_route.is_pinned"
                            class="data-[pinned=false]:text-muted-foreground"
                            :variant="map_route.is_pinned ? 'secondary' : 'ghost'"
                            size="sm"
                            @click="togglePinned"
                        >
                            <PinIcon />
                        </Button>
                    </TooltipTrigger>
                    <TooltipContent>{{ map_route.is_pinned ? 'Unpin' : 'Pin' }}</TooltipContent>
                </Tooltip>

                <Tooltip>
                    <TooltipTrigger as-child>
                        <Button variant="secondary" size="sm" @click="removeRoute">
                            <TrashIcon />
                        </Button>
                    </TooltipTrigger>
                    <TooltipContent>Remove</TooltipContent>
                </Tooltip>
            </div>
        </div>
    </DestinationContextMenu>
</template>

<style scoped></style>
