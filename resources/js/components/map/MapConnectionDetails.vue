<script setup lang="ts">
import { PopoverContent } from '@/components/ui/popover';
import { TMapConnection, TMapSolarSystem } from '@/types/models';
import { computed } from 'vue';

const { connection } = defineProps<{
    connection: TMapConnection & {
        source: TMapSolarSystem;
        target: TMapSolarSystem;
    };
}>();

const wormhole = computed(() => {
    if (!connection.signatures?.length) return null;

    return connection.signatures.find((sig) => !sig.wormhole?.name.startsWith('K162'))?.wormhole || null;
});

const maximumLifetime = computed(() => {
    if (!wormhole.value) return 0;
    return wormhole.value.maximum_lifetime / 3600;
});

const maximumJumpMass = computed(() => {
    if (!wormhole.value) return 0;
    return wormhole.value.maximum_jump_mass / 1_000_000;
});

const totalMass = computed(() => {
    if (!wormhole.value) return 0;
    return wormhole.value.total_mass / 1_000_000;
});

const ship_size = computed(() => {
    if (!wormhole.value) return '';

    const jump_mass = maximumJumpMass.value!;

    if (jump_mass >= 1_000) return 'XL';
    if (jump_mass >= 62) return 'L';
    if (jump_mass >= 5) return 'M';

    return 'S';
});

function formatMass(mass: number): string {
    return mass.toLocaleString('en-US');
}
</script>

<template>
    <PopoverContent class="w-60">
        <div class="grid grid-cols-2 divide-y truncate text-xs text-muted-foreground *:py-1" v-if="wormhole">
            <div class="col-span-full grid grid-cols-subgrid">
                <span>Type</span>
                <span class="text-right">{{ wormhole?.name }}</span>
            </div>
            <div class="col-span-full grid grid-cols-subgrid">
                <span>Total Mass</span>
                <span class="text-right">{{ formatMass(totalMass) }}</span>
            </div>
            <div class="col-span-full grid grid-cols-subgrid">
                <span>Maximum Jump Mass</span>
                <span class="text-right">{{ formatMass(maximumJumpMass) }}</span>
            </div>
            <div class="col-span-full grid grid-cols-subgrid">
                <span>Maximum Lifetime</span>
                <span class="text-right">{{ maximumLifetime }}h</span>
            </div>
            <div class="col-span-full grid grid-cols-subgrid">
                <span>Ship Size</span>
                <span class="text-right">{{ ship_size }}</span>
            </div>
            <div class="col-span-full grid grid-cols-subgrid">
                <span>Leads To</span>
                <span
                    :data-leads-to="wormhole?.leads_to"
                    class="text-right uppercase data-[leads-to=c1]:text-c1 data-[leads-to=c2]:text-c2 data-[leads-to=c3]:text-c3 data-[leads-to=c4]:text-c4 data-[leads-to=c5]:text-c5 data-[leads-to=c6]:text-c6 data-[leads-to=hs]:text-hs data-[leads-to=ls]:text-ls data-[leads-to=ns]:text-ns"
                    >{{ wormhole?.leads_to }}</span
                >
            </div>
        </div>
        <div v-else class="text-xs text-muted-foreground">No signature assigned</div>
    </PopoverContent>
</template>

<style scoped></style>
