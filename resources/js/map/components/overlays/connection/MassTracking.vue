<script setup lang="ts">
import { CharacterImage, TypeImage } from '@/components/images';
import { Popover, PopoverContent, PopoverTrigger } from '@/components/ui/popover';
import { Tooltip, TooltipContent, TooltipTrigger } from '@/components/ui/tooltip';
import { useNowUTC } from '@/composables/useNowUTC';
import { formatKilotons } from '@/lib/utils';
import { TConnectionJump, TMapConnection, TMapSolarsystem } from '@/pages/maps';
import { UTCDate } from '@date-fns/utc';
import { differenceInDays, differenceInHours, differenceInMinutes, format } from 'date-fns';
import { ChevronRight, MoveLeft, MoveRight } from 'lucide-vue-next';
import { computed } from 'vue';

const props = defineProps<{
    connection: TMapConnection & {
        source: TMapSolarsystem;
        target: TMapSolarsystem;
    };
    wormhole: { total_mass: number } | null;
}>();

const now = useNowUTC();

const jumps = computed<TConnectionJump[]>(() => props.connection.jumps ?? []);

const remainingPercent = computed(() => {
    if (!props.wormhole || props.wormhole.total_mass <= 0) return null;
    return Math.max(0, 100 - (props.connection.jumps_mass_sum / props.wormhole.total_mass) * 100);
});

const remainingMassKg = computed(() => {
    if (!props.wormhole) return null;
    return Math.max(0, props.wormhole.total_mass - props.connection.jumps_mass_sum);
});

/** The in-game mass stages: a hole shrinks below 50% and verges below 10%. */
const barColor = computed(() => {
    if (remainingPercent.value === null) return 'bg-neutral-500';
    if (remainingPercent.value <= 10) return 'bg-red-500';
    if (remainingPercent.value <= 50) return 'bg-amber-500';
    return 'bg-green-500';
});

function isOutbound(jump: TConnectionJump): boolean {
    return jump.from_solarsystem_id === props.connection.source.solarsystem_id;
}

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

function getJumpedAgo(jump: TConnectionJump): string {
    return getTimeAgo(new UTCDate(jump.created_at));
}

function getJumpedAt(jump: TConnectionJump): string {
    return format(new UTCDate(jump.created_at), 'MMM dd, HH:mm');
}
</script>

<template>
    <div class="space-y-1">
        <div class="flex items-center justify-between border-b pb-1 text-xs font-medium text-foreground">
            <Tooltip>
                <TooltipTrigger as-child>
                    <span class="cursor-help">Mass (estimate)</span>
                </TooltipTrigger>
                <TooltipContent> Only tracked pilots are counted; the total mass varies by ±10% in game. </TooltipContent>
            </Tooltip>
            <Popover v-if="jumps.length">
                <PopoverTrigger as-child>
                    <button
                        type="button"
                        class="flex items-center gap-0.5 rounded font-normal text-muted-foreground transition-colors hover:text-foreground"
                    >
                        {{ connection.jumps_count }} jumps
                        <ChevronRight class="size-3" />
                    </button>
                </PopoverTrigger>
                <PopoverContent class="w-88 p-0" side="right" align="start">
                    <div class="max-h-64 overflow-y-auto px-3">
                        <div class="grid grid-cols-[auto_auto_1fr_auto_auto_auto] divide-y divide-border/40">
                            <div
                                class="sticky top-0 z-10 col-span-full grid grid-cols-subgrid gap-x-3 bg-popover py-1.5 text-[10px] font-medium tracking-wider text-muted-foreground uppercase"
                            >
                                <span class="col-span-3">Ship</span>
                                <span>Pilot</span>
                                <span class="text-right">kt</span>
                                <span class="text-right">Age</span>
                            </div>
                            <div
                                v-for="jump in jumps"
                                :key="jump.id"
                                class="col-span-full grid grid-cols-subgrid items-center gap-x-3 py-1.5 transition-colors hover:bg-muted/30"
                            >
                                <component
                                    :is="isOutbound(jump) ? MoveRight : MoveLeft"
                                    class="size-3 shrink-0"
                                    :class="isOutbound(jump) ? 'text-sky-500' : 'text-purple-500'"
                                />
                                <div class="size-5 shrink-0">
                                    <TypeImage
                                        v-if="jump.ship_type_id"
                                        :type_id="jump.ship_type_id"
                                        :type_name="jump.ship_type_name || ''"
                                        class="size-5 rounded"
                                    />
                                </div>
                                <span class="min-w-0 truncate text-xs text-foreground">{{ jump.ship_type_name || 'Unknown' }}</span>
                                <div class="flex items-center gap-1.5">
                                    <CharacterImage
                                        :character_id="jump.character_id"
                                        :character_name="jump.character_name"
                                        class="size-4 shrink-0 rounded-full"
                                    />
                                    <span class="max-w-20 truncate text-[10px] text-muted-foreground">{{ jump.character_name }}</span>
                                </div>
                                <span class="text-right font-mono text-[10px] whitespace-nowrap text-foreground/80 tabular-nums">{{
                                    formatKilotons(jump.mass)
                                }}</span>
                                <Tooltip>
                                    <TooltipTrigger as-child>
                                        <span
                                            class="cursor-help text-right font-mono text-[10px] whitespace-nowrap text-muted-foreground tabular-nums"
                                            >{{ getJumpedAgo(jump) }}</span
                                        >
                                    </TooltipTrigger>
                                    <TooltipContent> {{ getJumpedAt(jump) }} </TooltipContent>
                                </Tooltip>
                            </div>
                        </div>
                    </div>
                    <div class="flex items-center justify-between border-t bg-muted/30 px-3 py-2 text-xs">
                        <span class="font-medium text-foreground">
                            Total
                            <span v-if="connection.jumps_count > jumps.length" class="font-normal text-muted-foreground">
                                · latest {{ jumps.length }} of {{ connection.jumps_count }} shown
                            </span>
                        </span>
                        <span class="font-mono text-foreground tabular-nums">{{ formatKilotons(connection.jumps_mass_sum) }} kt</span>
                    </div>
                </PopoverContent>
            </Popover>
            <span v-else class="font-normal text-muted-foreground">{{ connection.jumps_count }} jumps</span>
        </div>
        <div class="grid grid-cols-2 divide-y text-xs text-muted-foreground *:py-1">
            <div v-if="remainingPercent !== null" class="col-span-full">
                <div class="relative h-1.5 w-full overflow-hidden rounded-full bg-muted">
                    <div class="h-full rounded-full transition-all" :class="barColor" :style="{ width: `${remainingPercent}%` }" />
                    <Tooltip>
                        <TooltipTrigger as-child>
                            <span class="absolute inset-y-0 left-[10%] flex w-2 -translate-x-1/2 justify-center">
                                <span class="h-full w-px bg-popover" />
                            </span>
                        </TooltipTrigger>
                        <TooltipContent> Below 10% the hole verges to critical </TooltipContent>
                    </Tooltip>
                    <Tooltip>
                        <TooltipTrigger as-child>
                            <span class="absolute inset-y-0 left-1/2 flex w-2 -translate-x-1/2 justify-center">
                                <span class="h-full w-px bg-popover" />
                            </span>
                        </TooltipTrigger>
                        <TooltipContent> Below 50% the hole shrinks to reduced </TooltipContent>
                    </Tooltip>
                </div>
            </div>
            <div v-if="remainingPercent !== null && remainingMassKg !== null" class="col-span-full grid grid-cols-subgrid">
                <span>Remaining</span>
                <span class="text-right tabular-nums">≈ {{ formatKilotons(remainingMassKg) }} ({{ Math.round(remainingPercent) }}%)</span>
            </div>
            <div class="col-span-full grid grid-cols-subgrid">
                <span>Jumped</span>
                <span class="text-right tabular-nums">{{ formatKilotons(connection.jumps_mass_sum) }}</span>
            </div>
        </div>
    </div>
</template>
