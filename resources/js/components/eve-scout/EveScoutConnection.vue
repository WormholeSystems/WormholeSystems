<script setup lang="ts">
import DestinationContextMenu from '@/components/autopilot/DestinationContextMenu.vue';
import RoutePopover from '@/components/autopilot/RoutePopover.vue';
import SolarsystemSovereignty from '@/components/map/SolarsystemSovereignty.vue';
import SecurityStatus from '@/components/solarsystem/SecurityStatus.vue';
import SolarsystemClass from '@/components/solarsystem/SolarsystemClass.vue';
import { Badge } from '@/components/ui/badge';
import { Button } from '@/components/ui/button';
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
        <div class="col-span-full grid grid-cols-subgrid items-center border-t px-2 py-1.5 text-xs hover:bg-muted/30">
            <!-- System -->
            <div class="flex min-w-0 items-center gap-1.5">
                <SolarsystemClass v-if="otherSystem.class" :wormhole_class="otherSystem.class" class="shrink-0" />
                <SecurityStatus v-else-if="otherSystem.security !== undefined" :security="otherSystem.security" class="shrink-0" />
                <span class="truncate font-medium">{{ otherSystem.name }}</span>
            </div>

            <!-- Region -->
            <div class="truncate text-muted-foreground">{{ otherSystem.region?.name || '-' }}</div>

            <!-- Sovereignty -->
            <div class="flex items-center gap-1.5">
                <SolarsystemSovereignty :sovereignty="otherSystem.sovereignty" :solarsystem-id="otherSystem.id" class="size-3.5">
                    <template #fallback>
                        <span class="text-muted-foreground">-</span>
                    </template>
                </SolarsystemSovereignty>
            </div>

            <!-- Jumps -->
            <div class="flex justify-center">
                <RoutePopover :route="connection.route ?? undefined">
                    <Button variant="secondary" size="sm" class="h-5 px-1.5 font-mono text-xs">
                        <span v-if="connection.jumps_from_selected !== null">{{ connection.jumps_from_selected }}</span>
                        <span v-else>-</span>
                    </Button>
                </RoutePopover>
            </div>

            <!-- WH Type -->
            <div class="flex justify-center">
                <Badge variant="outline" class="h-5 px-1.5 font-mono text-xs">{{ connection.wormhole_type }}</Badge>
            </div>

            <!-- Sig In -->
            <div class="text-center font-mono text-muted-foreground">{{ specialSignature }}</div>

            <!-- Sig Out -->
            <div class="text-center font-mono text-muted-foreground">{{ otherSignature }}</div>

            <!-- Time -->
            <div class="flex justify-center">
                <Badge v-if="remainingHoursFormatted" variant="secondary" class="h-5 px-1.5 text-xs">
                    {{ remainingHoursFormatted }}
                </Badge>
                <span v-else class="text-muted-foreground">-</span>
            </div>
        </div>
    </DestinationContextMenu>
</template>
