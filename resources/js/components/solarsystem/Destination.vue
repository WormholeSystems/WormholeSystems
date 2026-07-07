<script setup lang="ts">
import DestinationContextMenu from '@/components/autopilot/DestinationContextMenu.vue';
import RoutePopover from '@/components/autopilot/RoutePopover.vue';
import SolarsystemSovereignty from '@/components/map/SolarsystemSovereignty.vue';
import SolarsystemClass from '@/components/solarsystem/SolarsystemClass.vue';
import { Button } from '@/components/ui/button';
import { usePath } from '@/composables/usePath';
import { useSolarsystemAliases } from '@/composables/useSolarsystemAliases';
import { useMapSolarsystems } from '@/map/api';
import type { TResolvedMapRouteSolarsystem } from '@/pages/maps';
import { vElementHover } from '@vueuse/components';
import { computed } from 'vue';

const { destination } = defineProps<{
    destination: TResolvedMapRouteSolarsystem;
}>();

const { map_solarsystems } = useMapSolarsystems();
const { getAlias } = useSolarsystemAliases(map_solarsystems);
const alias = computed(() => getAlias(destination.solarsystem.id));

const { setPath } = usePath();

function onHover(hovered: boolean) {
    if (hovered) {
        const routeToShow = destination.route ?? [];
        setPath(routeToShow);
    } else {
        setPath(null);
    }
}
</script>

<template>
    <DestinationContextMenu :solarsystem_id="destination.solarsystem.id">
        <div
            class="flex items-center gap-1.5 rounded border bg-white px-2 py-1 hover:bg-neutral-50 dark:bg-neutral-900/40 dark:hover:bg-neutral-800/30"
            v-element-hover="onHover"
        >
            <SolarsystemClass :solarsystem_class="destination.solarsystem.class" />

            <span class="truncate text-xs font-medium">
                <span v-if="alias" class="mr-1">{{ alias }}</span>
                <span :class="{ 'text-muted-foreground': alias }">{{ destination.solarsystem.name }}</span>
            </span>

            <SolarsystemSovereignty :sovereignty="destination.solarsystem.sovereignty" :solarsystem-id="destination.solarsystem.id" class="size-3" />

            <RoutePopover :route="destination.route">
                <Button variant="secondary" size="sm" class="h-5 px-1.5 font-mono text-xs">
                    <span v-if="destination.route && destination.route.length > 0">
                        {{ destination.route.length - 1 }}
                    </span>
                    <span v-else>∞</span>
                </Button>
            </RoutePopover>
        </div>
    </DestinationContextMenu>
</template>

<style scoped></style>
