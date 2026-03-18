<script setup lang="ts">
import DestinationContextMenu from '@/components/autopilot/DestinationContextMenu.vue';
import SolarsystemClass from '@/components/solarsystem/SolarsystemClass.vue';
import { useMap } from '@/composables/useMap';
import { useRallyRoute } from '@/composables/useRallyRoute';
import { useStaticSolarsystem } from '@/composables/useStaticSolarsystems';
import { Flag } from 'lucide-vue-next';
import { computed } from 'vue';

const map = useMap();
const { rallyRoute } = useRallyRoute();

const rallySolarsystemId = computed(() => map.value.rally_solarsystem_id);
const rallySolarsystem = useStaticSolarsystem(rallySolarsystemId);

const jumpCount = computed(() => {
    if (rallyRoute.value.length < 2) return null;
    return rallyRoute.value.length - 1;
});
</script>

<template>
    <div v-if="rallySolarsystem" class="absolute top-3 right-3 z-30">
        <DestinationContextMenu :solarsystem_id="rallySolarsystem.id">
            <button
                class="group flex cursor-pointer items-center gap-3 rounded-xl border border-pink-500/40 bg-gradient-to-r from-pink-500/10 to-pink-500/5 px-4 py-2.5 shadow-lg shadow-pink-500/10 backdrop-blur-md transition-all hover:border-pink-500/60 hover:shadow-pink-500/20 dark:from-pink-500/15 dark:to-pink-950/20"
            >
                <div class="flex size-8 items-center justify-center rounded-lg bg-pink-500/15 transition-colors group-hover:bg-pink-500/25">
                    <Flag class="size-4 text-pink-500" />
                </div>
                <div class="flex flex-col items-start gap-0.5">
                    <div class="flex items-center gap-1.5 text-sm font-semibold">
                        <SolarsystemClass :wormhole_class="rallySolarsystem.class" :security="rallySolarsystem.security" class="font-bold" />
                        <span>{{ rallySolarsystem.name }}</span>
                    </div>
                    <span v-if="rallySolarsystem.region" class="text-[11px] text-muted-foreground">{{ rallySolarsystem.region.name }}</span>
                </div>
                <div
                    v-if="jumpCount !== null"
                    class="flex h-8 items-center rounded-lg bg-pink-500/15 px-2.5 font-mono text-sm font-bold text-pink-500 transition-colors group-hover:bg-pink-500/25"
                >
                    {{ jumpCount }}j
                </div>
            </button>
        </DestinationContextMenu>
    </div>
</template>
