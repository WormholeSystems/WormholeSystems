<script setup lang="ts">
import { SHIP_SIZE_LETTERS, shipSizeFromJumpMass } from '@/lib/shipSize';
import { formatKilotons } from '@/lib/utils';
import { computed } from 'vue';

const { wormhole } = defineProps<{
    wormhole: {
        maximum_lifetime: number;
        maximum_jump_mass: number;
        total_mass: number;
        signature_strength?: number | null;
    };
}>();

const maximumLifetime = computed(() => wormhole.maximum_lifetime / 3600);

const ship_size = computed(() => {
    const size = shipSizeFromJumpMass(wormhole.maximum_jump_mass);
    return size ? SHIP_SIZE_LETTERS[size] : '—';
});
</script>

<template>
    <div class="space-y-1">
        <div class="border-b pb-1 text-xs font-medium text-foreground">Properties</div>
        <div class="grid grid-cols-2 divide-y truncate text-xs text-muted-foreground *:py-1">
            <div class="col-span-full grid grid-cols-subgrid">
                <span>Total Mass</span>
                <span class="text-right">{{ formatKilotons(wormhole.total_mass) }} kt</span>
            </div>
            <div class="col-span-full grid grid-cols-subgrid">
                <span>Maximum Jump Mass</span>
                <span class="text-right">{{ formatKilotons(wormhole.maximum_jump_mass) }} kt</span>
            </div>
            <div class="col-span-full grid grid-cols-subgrid">
                <span>Maximum Lifetime</span>
                <span class="text-right">{{ maximumLifetime }}h</span>
            </div>
            <div class="col-span-full grid grid-cols-subgrid">
                <span>Ship Size</span>
                <span class="text-right">{{ ship_size }}</span>
            </div>
            <div v-if="wormhole.signature_strength != null" class="col-span-full grid grid-cols-subgrid">
                <span>Sig Strength</span>
                <span class="text-right">{{ wormhole.signature_strength }}%</span>
            </div>
        </div>
    </div>
</template>
