<script setup lang="ts">
import DestinationContextMenu from '@/components/autopilot/DestinationContextMenu.vue';
import RoutePopover from '@/components/autopilot/RoutePopover.vue';
import SolarsystemSovereignty from '@/components/map/SolarsystemSovereignty.vue';
import { Tooltip, TooltipContent, TooltipTrigger } from '@/components/ui/tooltip';
import { useMapSolarsystems } from '@/composables/map';
import { useStaticSolarsystems } from '@/composables/useStaticSolarsystems';
import type { TRaidableSkyhook, TResolvedSolarsystem } from '@/pages/maps';
import { UTCDate } from '@date-fns/utc';
import { vElementHover } from '@vueuse/components';
import { useNow } from '@vueuse/core';
import { differenceInMinutes, format } from 'date-fns';
import { computed } from 'vue';

const { skyhook, jumps, route } = defineProps<{
    skyhook: TRaidableSkyhook;
    jumps: number | null;
    route: TResolvedSolarsystem[] | null;
}>();

const { map_solarsystems, setHoveredMapSolarsystem } = useMapSolarsystems();
const { resolveSolarsystem } = useStaticSolarsystems();

const map_solarsystem = computed(() => map_solarsystems.value.find((s) => s.solarsystem_id === skyhook.solarsystem_id));
const staticSolarsystem = computed(() => resolveSolarsystem(skyhook.solarsystem_id));

function onHover(hovered: boolean) {
    if (map_solarsystem.value) {
        setHoveredMapSolarsystem(map_solarsystem.value.id, hovered);
    }
}

const now = useNow({ interval: 30_000 });

const start = computed(() => new UTCDate(skyhook.theft_vulnerability_start));
const end = computed(() => new UTCDate(skyhook.theft_vulnerability_end));

const isVulnerable = computed(() => now.value >= start.value && now.value < end.value);
const hasEnded = computed(() => now.value >= end.value);

function formatRelative(target: Date): string {
    const minutes = Math.abs(differenceInMinutes(target, now.value));
    if (minutes < 1) return '<1m';
    if (minutes < 60) return `${minutes}m`;
    const hours = Math.floor(minutes / 60);
    const remainingMinutes = minutes % 60;
    if (hours < 24) return remainingMinutes > 0 ? `${hours}h ${remainingMinutes}m` : `${hours}h`;
    const days = Math.floor(hours / 24);
    return `${days}d ${hours % 24}h`;
}

const fifteen_minutes_in_ms = 15 * 60 * 1000;

const isAboutToEnd = computed(() => isVulnerable.value && end.value.getTime() - now.value.getTime() < fifteen_minutes_in_ms);

const statusTime = computed(() => formatRelative(hasEnded.value || isVulnerable.value ? end.value : start.value));

const statusColor = computed(() => {
    if (hasEnded.value) return 'bg-muted-foreground/40';
    if (!isVulnerable.value) return 'bg-amber-400';
    if (isAboutToEnd.value) return 'bg-red-400 animate-pulse';
    return 'bg-emerald-400 animate-pulse';
});

const statusTimeClass = computed(() => {
    if (hasEnded.value) return 'text-muted-foreground/60';
    if (!isVulnerable.value) return 'text-amber-400';
    if (isAboutToEnd.value) return 'text-red-400 animate-pulse';
    return 'text-emerald-400';
});

const tooltipHeadline = computed(() => {
    if (hasEnded.value) return `Closed ${statusTime.value} ago`;
    if (isVulnerable.value) return `Raidable for ${statusTime.value}`;
    return `Raidable in ${statusTime.value}`;
});

const tooltipWindow = computed(() => `${format(start.value, 'HH:mm')} – ${format(end.value, 'HH:mm')} UTC`);

const planetLabel = computed(() => skyhook.planet_name ?? `Planet ${skyhook.planet_id}`);

const jumpsLabel = computed(() => {
    if (jumps === null) return '—';
    if (jumps === 0) return 'here';
    return `${jumps}j`;
});

const jumpsClass = computed(() => {
    if (jumps === null) return 'text-muted-foreground/60';
    if (jumps < 8) return 'text-green-400';
    if (jumps < 15) return 'text-amber-400';
    return 'text-red-400';
});

const hasRoute = computed(() => route !== null && route.length > 1);
</script>

<template>
    <DestinationContextMenu :solarsystem_id="skyhook.solarsystem_id">
        <div v-element-hover="onHover" class="col-span-full grid grid-cols-subgrid items-center px-3 py-1.5 hover:bg-muted/20">
            <span class="size-2 rounded-full" :class="statusColor" />
            <span class="truncate text-xs">{{ planetLabel }}</span>
            <span class="truncate font-mono text-[10px] text-muted-foreground">{{ staticSolarsystem.region?.name ?? '' }}</span>
            <SolarsystemSovereignty :solarsystem-id="skyhook.solarsystem_id" class="size-4" />
            <RoutePopover v-if="hasRoute" :route="route ?? undefined">
                <span class="cursor-pointer text-right font-mono text-[10px] tracking-wider uppercase hover:text-foreground" :class="jumpsClass">{{
                    jumpsLabel
                }}</span>
            </RoutePopover>
            <span v-else class="text-right font-mono text-[10px] tracking-wider uppercase" :class="jumpsClass">{{ jumpsLabel }}</span>
            <Tooltip>
                <TooltipTrigger as-child>
                    <span
                        class="cursor-help justify-self-end font-mono text-[10px] font-semibold tracking-wider uppercase"
                        :class="statusTimeClass"
                        >{{ statusTime }}</span
                    >
                </TooltipTrigger>
                <TooltipContent class="flex flex-col gap-0.5">
                    <span>{{ tooltipHeadline }}</span>
                    <span class="font-mono text-[10px] text-muted-foreground">{{ tooltipWindow }}</span>
                </TooltipContent>
            </Tooltip>
        </div>
    </DestinationContextMenu>
</template>
