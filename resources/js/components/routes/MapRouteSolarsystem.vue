<script setup lang="ts">
import PinIcon from '@/components/icons/PinIcon.vue';
import TrashIcon from '@/components/icons/TrashIcon.vue';
import SovereigntyIcon from '@/components/map/SovereigntyIcon.vue';
import RoutePopover from '@/components/routes/RoutePopover.vue';
import SolarsystemClass from '@/components/SolarsystemClass.vue';
import { Button } from '@/components/ui/button';
import { Tooltip, TooltipContent, TooltipTrigger } from '@/components/ui/tooltip';
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
    <div
        class="group col-span-full grid grid-cols-subgrid items-center border-b px-3 py-1 last:border-b-0 hover:bg-neutral-50 dark:hover:bg-neutral-800/30"
        v-element-hover="onHover"
    >
        <!-- Class Column -->
        <div class="flex justify-center">
            <SolarsystemClass :wormhole_class="map_route.solarsystem.class" :security="map_route.solarsystem.security" />
        </div>

        <!-- Name Column -->
        <div class="min-w-0 truncate font-medium">
            {{ map_route.solarsystem.name }}
        </div>

        <!-- Jumps Column -->
        <div class="flex justify-center">
            <RoutePopover :route="map_route.route">
                <Button variant="outline" size="sm" class="h-5 w-8 px-0 font-mono text-[10px] font-medium">
                    <span v-if="map_route.route && map_route.route.length > 0">
                        {{ map_route.route.length - 1 }}
                    </span>
                    <span v-else>∞</span>
                </Button>
            </RoutePopover>
        </div>

        <!-- Region Column -->
        <div class="min-w-0 truncate text-[10px] text-muted-foreground">
            {{ map_route.solarsystem.region?.name || '' }}
        </div>

        <!-- Sovereignty Column -->
        <div class="flex justify-center">
            <SovereigntyIcon v-if="map_route.solarsystem.sovereignty" :sovereignty="map_route.solarsystem.sovereignty" />
            <div v-else class="size-4"></div>
        </div>

        <!-- Actions Column -->
        <div v-if="can_write" class="flex justify-center">
            <Tooltip>
                <TooltipTrigger>
                    <Button
                        variant="ghost"
                        size="sm"
                        @click="togglePinned"
                        :class="map_route.is_pinned ? 'text-yellow-600' : 'text-muted-foreground'"
                        class="h-5 w-5 p-0"
                    >
                        <PinIcon class="size-3" />
                    </Button>
                </TooltipTrigger>
                <TooltipContent>{{ map_route.is_pinned ? 'Unpin' : 'Pin' }}</TooltipContent>
            </Tooltip>

            <Tooltip>
                <TooltipTrigger>
                    <Button variant="ghost" size="sm" @click="removeRoute" class="h-5 w-5 p-0 text-muted-foreground hover:text-red-600">
                        <TrashIcon class="size-3" />
                    </Button>
                </TooltipTrigger>
                <TooltipContent>Remove</TooltipContent>
            </Tooltip>
        </div>
    </div>
</template>

<style scoped></style>
