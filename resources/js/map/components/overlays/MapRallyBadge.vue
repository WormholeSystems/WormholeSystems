<script setup lang="ts">
import DestinationContextMenu from '@/components/autopilot/DestinationContextMenu.vue';
import RoutePopover from '@/components/autopilot/RoutePopover.vue';
import SolarsystemClass from '@/components/solarsystem/SolarsystemClass.vue';
import { useMap } from '@/composables/useMap';
import { useRallyRoute } from '@/composables/useRallyRoute';
import { useStaticSolarsystems } from '@/composables/useStaticSolarsystems';
import { Flag } from 'lucide-vue-next';
import { computed } from 'vue';

const map = useMap();
const { rallyRoutes } = useRallyRoute();
const { getSolarsystemById, resolveSolarsystem } = useStaticSolarsystems();

const rallyPoints = computed(() =>
    map.value.rally_solarsystem_ids.flatMap((solarsystemId) => {
        const solarsystem = getSolarsystemById(solarsystemId);
        if (!solarsystem) return [];

        const route = rallyRoutes.value[solarsystemId] ?? [];

        return [
            {
                solarsystem,
                jumpCount: route.length < 2 ? null : route.length - 1,
                resolvedRoute: route.map((step) => resolveSolarsystem(step.id)),
            },
        ];
    }),
);
</script>

<template>
    <div v-if="rallyPoints.length" class="absolute top-3 right-3 z-30 flex flex-col items-end gap-2">
        <div
            v-for="rallyPoint in rallyPoints"
            :key="rallyPoint.solarsystem.id"
            class="flex items-center gap-3 rounded-xl border border-pink-500/40 bg-gradient-to-r from-pink-500/10 to-pink-500/5 px-4 py-2.5 shadow-lg shadow-pink-500/10 backdrop-blur-md dark:from-pink-500/15 dark:to-pink-950/20"
        >
            <DestinationContextMenu :solarsystem_id="rallyPoint.solarsystem.id">
                <button class="group flex cursor-pointer items-center gap-3 transition-all hover:opacity-80">
                    <div class="flex flex-col items-start gap-0.5">
                        <span class="text-[10px] font-medium tracking-wider text-pink-500/70 uppercase">Rally Point</span>
                        <div class="flex items-center gap-1.5 text-sm font-semibold">
                            <SolarsystemClass :solarsystem_class="rallyPoint.solarsystem.class" class="font-bold" />
                            <span>{{ rallyPoint.solarsystem.name }}</span>
                            <span v-if="rallyPoint.solarsystem.region" class="text-xs text-muted-foreground">{{
                                rallyPoint.solarsystem.region.name
                            }}</span>
                        </div>
                    </div>
                </button>
            </DestinationContextMenu>
            <RoutePopover v-if="rallyPoint.jumpCount !== null" :route="rallyPoint.resolvedRoute">
                <button
                    class="flex h-8 cursor-pointer items-center gap-1.5 rounded-lg bg-pink-500/15 px-2.5 font-mono text-sm font-bold text-pink-500 transition-colors hover:bg-pink-500/25"
                >
                    <Flag class="size-3" />
                    {{ rallyPoint.jumpCount }}j
                </button>
            </RoutePopover>
        </div>
    </div>
</template>
