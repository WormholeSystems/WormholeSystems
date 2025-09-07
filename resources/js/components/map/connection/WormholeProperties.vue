<script setup lang="ts">
import { TWormhole } from '@/types/models';
import { computed } from 'vue';

const props = defineProps<{
    wormhole: TWormhole;
}>();

const maximumLifetime = computed(() => {
    if (!props.wormhole?.maximum_lifetime) return null;
    return props.wormhole.maximum_lifetime / 3600;
});

const maximumJumpMass = computed(() => {
    if (!props.wormhole?.maximum_jump_mass) return null;
    return props.wormhole.maximum_jump_mass / 1_000_000;
});

const totalMass = computed(() => {
    if (!props.wormhole?.total_mass) return null;
    return props.wormhole.total_mass / 1_000_000;
});

const ship_size = computed(() => {
    if (!props.wormhole?.maximum_jump_mass) return null;

    const jump_mass = maximumJumpMass.value!;

    if (jump_mass >= 1_000) return 'XL';
    if (jump_mass >= 62) return 'L';
    if (jump_mass >= 5) return 'M';

    return 'S';
});

const hasWormholeData = computed(() => {
    return props.wormhole && (props.wormhole.maximum_lifetime || props.wormhole.maximum_jump_mass || props.wormhole.total_mass);
});

function formatMass(mass: number): string {
    return mass.toLocaleString('en-US');
}
</script>

<template>
    <div v-if="hasWormholeData" class="space-y-1">
        <div class="border-b pb-1 text-xs font-medium text-foreground">Properties</div>
        <div class="grid grid-cols-2 divide-y truncate text-xs text-muted-foreground *:py-1">
            <div v-if="totalMass !== null" class="col-span-full grid grid-cols-subgrid">
                <span>Total Mass</span>
                <span class="text-right">{{ formatMass(totalMass) }}</span>
            </div>
            <div v-if="maximumJumpMass !== null" class="col-span-full grid grid-cols-subgrid">
                <span>Maximum Jump Mass</span>
                <span class="text-right">{{ formatMass(maximumJumpMass) }}</span>
            </div>
            <div v-if="maximumLifetime !== null" class="col-span-full grid grid-cols-subgrid">
                <span>Maximum Lifetime</span>
                <span class="text-right">{{ maximumLifetime }}h</span>
            </div>
            <div v-if="ship_size !== null" class="col-span-full grid grid-cols-subgrid">
                <span>Ship Size</span>
                <span class="text-right">{{ ship_size }}</span>
            </div>
        </div>
    </div>
</template>
