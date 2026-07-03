<script setup lang="ts">
import { formatKilotons } from '@/lib/utils';
import type { TStatic } from '@/pages/maps';
import { computed } from 'vue';

const { wormhole } = defineProps<{
    wormhole: TStatic;
}>();

const lifetimeHours = computed(() => Math.round(wormhole.maximum_lifetime / 3600));

const shipSize = computed(() => {
    const jumpMassMillions = wormhole.maximum_jump_mass / 1_000_000;

    if (jumpMassMillions >= 1_000) return 'XL';
    if (jumpMassMillions >= 62) return 'L';
    if (jumpMassMillions >= 5) return 'M';

    return 'S';
});

function formatMass(mass: number): string {
    if (!mass) return '—';
    return `${formatKilotons(mass)} kt`;
}
</script>

<template>
    <div class="min-w-40">
        <div class="border-b border-border/50 px-3 py-2">
            <span class="font-mono text-xs font-medium">
                {{ wormhole.name }} →
                <span
                    :data-leads-to="wormhole.leads_to"
                    class="uppercase data-[leads-to=c1]:text-c1 data-[leads-to=c13]:text-c13 data-[leads-to=c2]:text-c2 data-[leads-to=c3]:text-c3 data-[leads-to=c4]:text-c4 data-[leads-to=c5]:text-c5 data-[leads-to=c6]:text-c6 data-[leads-to=hs]:text-hs data-[leads-to=ls]:text-ls data-[leads-to=ns]:text-ns"
                >
                    {{ wormhole.leads_to }}
                </span>
            </span>
        </div>
        <div class="space-y-1 px-3 py-2 text-[11px]">
            <div class="flex justify-between gap-4">
                <span class="text-muted-foreground">Total Mass</span>
                <span>{{ formatMass(wormhole.total_mass) }}</span>
            </div>
            <div class="flex justify-between gap-4">
                <span class="text-muted-foreground">Max Jump Mass</span>
                <span>{{ formatMass(wormhole.maximum_jump_mass) }}</span>
            </div>
            <div class="flex justify-between gap-4">
                <span class="text-muted-foreground">Ship Size</span>
                <span>{{ shipSize }}</span>
            </div>
            <div class="flex justify-between gap-4">
                <span class="text-muted-foreground">Lifetime</span>
                <span>{{ lifetimeHours ? `${lifetimeHours}h` : '—' }}</span>
            </div>
            <div class="flex justify-between gap-4">
                <span class="text-muted-foreground">Sig Strength</span>
                <span>{{ wormhole.signature_strength != null ? `${wormhole.signature_strength}%` : 'unknown' }}</span>
            </div>
        </div>
    </div>
</template>

<style scoped></style>
