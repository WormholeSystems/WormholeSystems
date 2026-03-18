<script setup lang="ts">
import DestinationContextMenu from '@/components/autopilot/DestinationContextMenu.vue';
import RoutePopover from '@/components/autopilot/RoutePopover.vue';
import SolarsystemClass from '@/components/solarsystem/SolarsystemClass.vue';
import { useMap } from '@/composables/useMap';
import { useRallyRoute } from '@/composables/useRallyRoute';
import { useStaticSolarsystem, useStaticSolarsystems } from '@/composables/useStaticSolarsystems';
import { Flag } from 'lucide-vue-next';
import { computed } from 'vue';

const map = useMap();
const { rallyRoute } = useRallyRoute();
const { resolveSolarsystem } = useStaticSolarsystems();

const rallySolarsystemId = computed(() => map.value.rally_solarsystem_id);
const rallySolarsystem = useStaticSolarsystem(rallySolarsystemId);

const jumpCount = computed(() => {
    if (rallyRoute.value.length < 2) return null;
    return rallyRoute.value.length - 1;
});

const resolvedRoute = computed(() => {
    return rallyRoute.value.map((step) => resolveSolarsystem(step.id));
});
</script>

<template>
    <div v-if="rallySolarsystem" class="absolute top-3 right-3 z-30">
        <div
            class="flex items-center gap-3 rounded-xl border border-pink-500/40 bg-gradient-to-r from-pink-500/10 to-pink-500/5 px-4 py-2.5 shadow-lg shadow-pink-500/10 backdrop-blur-md dark:from-pink-500/15 dark:to-pink-950/20"
        >
            <DestinationContextMenu :solarsystem_id="rallySolarsystem.id">
                <button class="group flex cursor-pointer items-center gap-3 transition-all hover:opacity-80">
                    <div class="flex flex-col items-start gap-0.5">
                        <span class="text-[10px] font-medium tracking-wider text-pink-500/70 uppercase">Rally Point</span>
                        <div class="flex items-center gap-1.5 text-sm font-semibold">
                            <SolarsystemClass :wormhole_class="rallySolarsystem.class" :security="rallySolarsystem.security" class="font-bold" />
                            <span>{{ rallySolarsystem.name }}</span>
                            <span v-if="rallySolarsystem.region" class="text-xs text-muted-foreground">{{ rallySolarsystem.region.name }}</span>
                        </div>
                    </div>
                </button>
            </DestinationContextMenu>
            <RoutePopover v-if="jumpCount !== null" :route="resolvedRoute">
                <button
                    class="flex h-8 cursor-pointer items-center gap-1.5 rounded-lg bg-pink-500/15 px-2.5 font-mono text-sm font-bold text-pink-500 transition-colors hover:bg-pink-500/25"
                >
                    <Flag class="size-3" />
                    {{ jumpCount }}j
                </button>
            </RoutePopover>
        </div>
    </div>
</template>
