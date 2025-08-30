<script setup lang="ts">
import { PopoverContent } from '@/components/ui/popover';
import { Tooltip, TooltipContent, TooltipTrigger } from '@/components/ui/tooltip';
import { useNowUTC } from '@/composables/useNowUTC';
import { TMapConnection, TMapSolarSystem } from '@/types/models';
import { UTCDate } from '@date-fns/utc';
import { differenceInDays, differenceInHours, differenceInMinutes, format, max, min } from 'date-fns';
import { computed } from 'vue';

const { connection } = defineProps<{
    connection: TMapConnection & {
        source: TMapSolarSystem;
        target: TMapSolarSystem;
    };
}>();

const now = useNowUTC();

const outSignature = computed(() => {
    if (!connection.signatures?.length) return null;
    return connection.signatures.find((sig) => !sig.wormhole?.name.startsWith('K162')) || null;
});

const inSignature = computed(() => {
    if (!connection.signatures?.length) return null;
    return connection.signatures.find((sig) => sig.wormhole?.name.startsWith('K162')) || null;
});

const wormhole = computed(() => {
    return outSignature.value?.wormhole || inSignature.value?.wormhole || null;
});

const maximumLifetime = computed(() => {
    if (!wormhole.value?.maximum_lifetime) return null;
    return wormhole.value.maximum_lifetime / 3600;
});

const maximumJumpMass = computed(() => {
    if (!wormhole.value?.maximum_jump_mass) return null;
    return wormhole.value.maximum_jump_mass / 1_000_000;
});

const totalMass = computed(() => {
    if (!wormhole.value?.total_mass) return null;
    return wormhole.value.total_mass / 1_000_000;
});

const ship_size = computed(() => {
    if (!wormhole.value?.maximum_jump_mass) return null;

    const jump_mass = maximumJumpMass.value!;

    if (jump_mass >= 1_000) return 'XL';
    if (jump_mass >= 62) return 'L';
    if (jump_mass >= 5) return 'M';

    return 'S';
});

const hasWormholeData = computed(() => {
    return wormhole.value && (wormhole.value.maximum_lifetime || wormhole.value.maximum_jump_mass || wormhole.value.total_mass);
});

const massStatusDisplay = computed(() => {
    const status = connection.mass_status;
    switch (status) {
        case 'fresh':
            return 'Fresh';
        case 'reduced':
            return 'Reduced';
        case 'critical':
            return 'Critical';
        default:
            return 'Unknown';
    }
});

const createdDate = computed(() => {
    const dates = [connection.created_at];

    // Add signature created_at dates if they exist
    if (connection.signatures?.length) {
        connection.signatures.forEach((sig) => {
            if (sig.created_at) {
                dates.push(sig.created_at);
            }
        });
    }

    // Find the earliest date
    return min(dates.map((date) => new UTCDate(date)));
});

const updatedDate = computed(() => {
    const dates = [connection.updated_at];

    // Add signature updated_at dates if they exist
    if (connection.signatures?.length) {
        connection.signatures.forEach((sig) => {
            if (sig.updated_at) {
                dates.push(sig.updated_at);
            }
        });
    }

    // Find the latest date
    return max(dates.map((date) => new UTCDate(date)));
});

const createdAt = computed(() => {
    return format(createdDate.value, 'MMM dd, HH:mm');
});

const updatedAt = computed(() => {
    return format(updatedDate.value, 'MMM dd, HH:mm');
});

function getTimeAgo(date: Date): string {
    const diff_in_days = differenceInDays(now.value, date);
    if (diff_in_days > 0) {
        return `${diff_in_days}d ago`;
    }
    const diff_in_hours = differenceInHours(now.value, date);
    if (diff_in_hours > 0) {
        return `${diff_in_hours}h ago`;
    }
    const diff_in_minutes = differenceInMinutes(now.value, date);
    if (diff_in_minutes > 0) {
        return `${diff_in_minutes}m ago`;
    }
    return 'just now';
}

const createdAgo = computed(() => getTimeAgo(createdDate.value));
const updatedAgo = computed(() => getTimeAgo(updatedDate.value));

function formatMass(mass: number): string {
    return mass.toLocaleString('en-US');
}
</script>

<template>
    <PopoverContent class="w-60">
        <div v-if="outSignature || inSignature" class="space-y-3">
            <!-- Out Signature (Non-K162) -->
            <div v-if="outSignature" class="space-y-1">
                <div class="border-b pb-1 text-xs font-medium text-foreground">Out Sig</div>
                <div class="grid grid-cols-2 divide-y truncate text-xs text-muted-foreground *:py-1">
                    <div class="col-span-full grid grid-cols-subgrid">
                        <span>Type</span>
                        <span class="text-right">{{ outSignature.wormhole?.name || outSignature.type || 'Unknown' }}</span>
                    </div>
                    <div class="col-span-full grid grid-cols-subgrid">
                        <span>Signature ID</span>
                        <span class="text-right">{{ outSignature.signature_id || 'Unknown' }}</span>
                    </div>
                    <div class="col-span-full grid grid-cols-subgrid" v-if="outSignature.wormhole?.leads_to">
                        <span>Leads To</span>
                        <span
                            :data-leads-to="outSignature.wormhole?.leads_to"
                            class="text-right uppercase data-[leads-to=c1]:text-c1 data-[leads-to=c2]:text-c2 data-[leads-to=c3]:text-c3 data-[leads-to=c4]:text-c4 data-[leads-to=c5]:text-c5 data-[leads-to=c6]:text-c6 data-[leads-to=hs]:text-hs data-[leads-to=ls]:text-ls data-[leads-to=ns]:text-ns"
                            >{{ outSignature.wormhole?.leads_to }}</span
                        >
                    </div>
                </div>
            </div>

            <!-- In Signature (K162) -->
            <div v-if="inSignature" class="space-y-1">
                <div class="border-b pb-1 text-xs font-medium text-foreground">In Sig</div>
                <div class="grid grid-cols-2 divide-y truncate text-xs text-muted-foreground *:py-1">
                    <div class="col-span-full grid grid-cols-subgrid">
                        <span>Type</span>
                        <span class="text-right">{{ inSignature.wormhole?.name || inSignature.type || 'Unknown' }}</span>
                    </div>
                    <div class="col-span-full grid grid-cols-subgrid">
                        <span>Signature ID</span>
                        <span class="text-right">{{ inSignature.signature_id || 'Unknown' }}</span>
                    </div>
                </div>
            </div>

            <!-- Connection Status -->
            <div class="space-y-1">
                <div class="border-b pb-1 text-xs font-medium text-foreground">Status</div>
                <div class="grid grid-cols-2 divide-y truncate text-xs text-muted-foreground *:py-1">
                    <div class="col-span-full grid grid-cols-subgrid">
                        <span>EOL</span>
                        <span class="text-right" :class="{ 'text-purple-500': connection.is_eol }">
                            {{ connection.is_eol ? 'Yes' : 'No' }}
                        </span>
                    </div>
                    <div class="col-span-full grid grid-cols-subgrid">
                        <span>Mass Status</span>
                        <span
                            class="text-right"
                            :class="{
                                'text-green-500': connection.mass_status === 'fresh',
                                'text-yellow-500': connection.mass_status === 'reduced',
                                'text-red-500': connection.mass_status === 'critical',
                            }"
                        >
                            {{ massStatusDisplay }}
                        </span>
                    </div>
                    <div class="col-span-full grid grid-cols-subgrid">
                        <span>Created</span>
                        <Tooltip>
                            <TooltipTrigger as-child>
                                <span class="cursor-help text-right">{{ createdAt }}</span>
                            </TooltipTrigger>
                            <TooltipContent>{{ createdAgo }}</TooltipContent>
                        </Tooltip>
                    </div>
                    <div class="col-span-full grid grid-cols-subgrid">
                        <span>Updated</span>
                        <Tooltip>
                            <TooltipTrigger as-child>
                                <span class="cursor-help text-right">{{ updatedAt }}</span>
                            </TooltipTrigger>
                            <TooltipContent>{{ updatedAgo }}</TooltipContent>
                        </Tooltip>
                    </div>
                </div>
            </div>

            <!-- Wormhole Properties (shared between both signatures) -->
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
        </div>
        <div v-else class="text-xs text-muted-foreground">No signature assigned</div>
    </PopoverContent>
</template>

<style scoped></style>
