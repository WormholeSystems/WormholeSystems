<script setup lang="ts">
import { computed } from 'vue';

const { wormhole } = defineProps<{
    wormhole: {
        maximum_lifetime: number;
        maximum_jump_mass: number;
        total_mass: number;
    };
}>();

const maximumLifetime = computed(() => wormhole.maximum_lifetime / 3600);

const maximumJumpMass = computed(() => wormhole.maximum_jump_mass / 1_000_000);
const totalMass = computed(() => wormhole.total_mass / 1_000_000);

const ship_size = computed(() => {
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
    <div class="space-y-1">
        <div class="border-b pb-1 text-xs font-medium text-foreground">Properties</div>
        <div class="grid grid-cols-2 divide-y truncate text-xs text-muted-foreground *:py-1">
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
        </div>
    </div>
</template>
