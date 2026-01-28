<script setup lang="ts">
import DestinationContextMenu from '@/components/autopilot/DestinationContextMenu.vue';
import RoutePopover from '@/components/autopilot/RoutePopover.vue';
import SolarsystemSovereignty from '@/components/map/SolarsystemSovereignty.vue';
import SolarsystemClass from '@/components/solarsystem/SolarsystemClass.vue';
import SolarsystemEffect from '@/components/solarsystem/SolarsystemEffect.vue';
import type { TResolvedSolarsystem } from '@/pages/maps';
import type { TEveScoutConnection } from '@/types/eve-scout';
import type { TStaticSolarsystem } from '@/types/static-data';
import { computed } from 'vue';

type TEveScoutConnectionWithSystems = TEveScoutConnection & {
    in_system: TStaticSolarsystem;
    out_system: TStaticSolarsystem;
    route?: TResolvedSolarsystem[] | null;
};

const { connection, specialSystem } = defineProps<{
    connection: TEveScoutConnectionWithSystems;
    specialSystem: string;
}>();

// Get the "other" system (not the special system)
const otherSystem = computed(() => {
    return connection.in_system.name === specialSystem ? connection.out_system : connection.in_system;
});

// Get the signature for the special system side
const specialSignature = computed(() => {
    return connection.in_system.name === specialSystem ? connection.in_signature : connection.out_signature;
});

// Get the signature for the other system side
const otherSignature = computed(() => {
    return connection.in_system.name === specialSystem ? connection.out_signature : connection.in_signature;
});

const remainingHoursFormatted = computed(() => {
    if (!connection.remaining_hours) return null;
    const hours = Math.floor(connection.remaining_hours);
    const minutes = Math.round((connection.remaining_hours - hours) * 60);
    if (hours === 0) return `${minutes}m`;
    if (minutes === 0) return `${hours}h`;
    return `${hours}h ${minutes}m`;
});
</script>

<template>
    <DestinationContextMenu :solarsystem_id="otherSystem.id">
        <div class="col-span-full grid grid-cols-subgrid items-center border-b border-border/30 px-3 py-1.5 hover:bg-muted/30">
            <SolarsystemClass :wormhole_class="otherSystem.class" :security="otherSystem.security" class="justify-self-center" />

            <span class="truncate text-xs">{{ otherSystem.name }}</span>

            <span class="truncate text-[10px] text-muted-foreground">{{ otherSystem.region?.name || '' }}</span>

            <SolarsystemSovereignty :sovereignty="otherSystem.sovereignty" :solarsystem-id="otherSystem.id" class="size-4 justify-self-center">
                <template #fallback>
                    <SolarsystemEffect v-if="otherSystem.effect" :effect="otherSystem.effect.name" />
                </template>
            </SolarsystemSovereignty>

            <RoutePopover :route="connection.route ?? undefined">
                <span
                    v-if="connection.jumps_from_selected !== null"
                    class="cursor-pointer font-mono text-xs font-medium"
                    :class="{
                        'text-green-400': connection.jumps_from_selected < 8,
                        'text-amber-400': connection.jumps_from_selected >= 8 && connection.jumps_from_selected < 15,
                        'text-red-400': connection.jumps_from_selected >= 15,
                    }"
                >
                    {{ connection.jumps_from_selected }}j
                </span>
                <span v-else class="font-mono text-[10px] text-muted-foreground/60">--</span>
            </RoutePopover>

            <span class="font-mono text-[10px] text-muted-foreground">{{ specialSignature }}</span>

            <span class="font-mono text-[10px] text-muted-foreground">{{ otherSignature }}</span>

            <span class="font-mono text-[10px] text-muted-foreground">{{ connection.wormhole_type }}</span>

            <span v-if="remainingHoursFormatted" class="font-mono text-[10px] text-muted-foreground">{{ remainingHoursFormatted }}</span>
            <span v-else class="font-mono text-[10px] text-muted-foreground/60">--</span>
        </div>
    </DestinationContextMenu>
</template>
