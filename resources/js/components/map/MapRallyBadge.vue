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
                class="flex cursor-pointer items-center gap-2 rounded-lg border border-pink-500/30 bg-white/80 px-3 py-1.5 text-xs shadow-sm backdrop-blur-sm transition-colors hover:bg-pink-50 dark:bg-neutral-900/80 dark:hover:bg-pink-950/30"
            >
                <Flag class="size-3.5 text-pink-500" />
                <SolarsystemClass :wormhole_class="rallySolarsystem.class" :security="rallySolarsystem.security" class="font-bold" />
                <span class="font-medium">{{ rallySolarsystem.name }}</span>
                <span v-if="rallySolarsystem.region" class="text-muted-foreground">{{ rallySolarsystem.region.name }}</span>
                <span v-if="jumpCount !== null" class="font-mono text-pink-500">{{ jumpCount }}j</span>
            </button>
        </DestinationContextMenu>
    </div>
</template>
