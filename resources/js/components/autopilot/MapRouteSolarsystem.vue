<script setup lang="ts">
import DestinationContextMenu from '@/components/autopilot/DestinationContextMenu.vue';
import RoutePopover from '@/components/autopilot/RoutePopover.vue';
import PinIcon from '@/components/icons/PinIcon.vue';
import TrashIcon from '@/components/icons/TrashIcon.vue';
import SolarsystemSovereignty from '@/components/map/SolarsystemSovereignty.vue';
import SolarsystemClass from '@/components/solarsystem/SolarsystemClass.vue';
import SolarsystemEffect from '@/components/solarsystem/SolarsystemEffect.vue';
import { usePath } from '@/composables/usePath';
import usePermission from '@/composables/usePermission';
import type { TResolvedMapRouteSolarsystem } from '@/pages/maps';
import MapRouteSolarsystems from '@/routes/map-route-solarsystems';
import { router } from '@inertiajs/vue3';
import { vElementHover } from '@vueuse/components';

const { map_route } = defineProps<{
    map_route: TResolvedMapRouteSolarsystem;
}>();

const { setPath } = usePath();

const { canEdit: can_write } = usePermission();

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
            class="col-span-full grid grid-cols-subgrid items-center border-b border-border/30 px-3 py-1.5 hover:bg-muted/30"
            v-element-hover="onHover"
        >
            <SolarsystemClass :wormhole_class="map_route.solarsystem.class" :security="map_route.solarsystem.security" class="justify-self-center" />

            <span class="truncate text-xs">{{ map_route.solarsystem.name }}</span>

            <span class="truncate text-[10px] text-muted-foreground">{{ map_route.solarsystem.region?.name || '' }}</span>

            <SolarsystemSovereignty
                :sovereignty="map_route.solarsystem.sovereignty"
                :solarsystem-id="map_route.solarsystem.id"
                class="size-4 justify-self-center"
            >
                <template #fallback>
                    <SolarsystemEffect v-if="map_route.solarsystem.effect" :effect="map_route.solarsystem.effect.name" />
                </template>
            </SolarsystemSovereignty>

            <RoutePopover :route="map_route.route">
                <span
                    v-if="map_route.route && map_route.route.length > 0"
                    class="cursor-pointer font-mono text-xs font-medium"
                    :class="{
                        'text-green-400': map_route.route.length - 1 < 8,
                        'text-amber-400': map_route.route.length - 1 >= 8 && map_route.route.length - 1 < 15,
                        'text-red-400': map_route.route.length - 1 >= 15,
                    }"
                >
                    {{ map_route.route.length - 1 }}j
                </span>
                <span v-else class="font-mono text-[10px] text-muted-foreground/60">--</span>
            </RoutePopover>

            <div v-if="can_write" class="flex justify-end gap-1">
                <button
                    :data-pinned="map_route.is_pinned"
                    class="text-muted-foreground/60 hover:text-foreground data-[pinned=true]:text-amber-400"
                    @click.stop="togglePinned"
                >
                    <PinIcon class="size-3" />
                </button>
                <button class="text-muted-foreground/60 hover:text-destructive" @click.stop="removeRoute">
                    <TrashIcon class="size-3" />
                </button>
            </div>
        </div>
    </DestinationContextMenu>
</template>

<style scoped></style>
