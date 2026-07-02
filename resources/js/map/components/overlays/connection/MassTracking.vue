<script setup lang="ts">
import { CharacterImage, TypeImage } from '@/components/images';
import { Popover, PopoverContent, PopoverTrigger } from '@/components/ui/popover';
import { Tooltip, TooltipContent, TooltipTrigger } from '@/components/ui/tooltip';
import { useNowUTC } from '@/composables/useNowUTC';
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

const jumpedMass = computed(() => props.connection.jumps_mass_sum / 1_000_000);

const remainingPercent = computed(() => {
    if (!props.wormhole || props.wormhole.total_mass <= 0) return null;
    return Math.max(0, 100 - (props.connection.jumps_mass_sum / props.wormhole.total_mass) * 100);
});

const remainingMass = computed(() => {
    if (!props.wormhole) return null;
    return Math.max(0, props.wormhole.total_mass - props.connection.jumps_mass_sum) / 1_000_000;
});

const barColor = computed(() => {
    if (remainingPercent.value === null) return 'bg-neutral-500';
    if (remainingPercent.value <= 10) return 'bg-red-500';
    if (remainingPercent.value <= 50) return 'bg-amber-500';
    return 'bg-green-500';
});

function formatMass(mass: number): string {
    return Math.round(mass).toLocaleString('en-US');
}

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
                <PopoverContent class="w-80 p-3" side="right" align="start">
                    <div class="space-y-1">
                        <div class="border-b pb-1 text-xs font-medium text-foreground">Jump history</div>
                        <div class="grid max-h-64 grid-cols-[auto_auto_1fr_auto_auto] divide-y divide-border/30 overflow-y-auto">
                            <div
                                v-for="jump in jumps"
                                :key="jump.id"
                                class="col-span-full grid grid-cols-subgrid items-center gap-2 py-1.5 hover:bg-muted/30"
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
                                <span class="min-w-0 truncate text-xs">{{ jump.ship_type_name || 'Unknown' }}</span>
                                <div class="flex items-center gap-1">
                                    <CharacterImage
                                        :character_id="jump.character_id"
                                        :character_name="jump.character_name"
                                        class="size-4 shrink-0 rounded"
                                    />
                                    <span class="max-w-20 truncate font-mono text-[10px] text-muted-foreground">{{ jump.character_name }}</span>
                                </div>
                                <Tooltip>
                                    <TooltipTrigger as-child>
                                        <span class="cursor-help text-right font-mono text-[10px] whitespace-nowrap text-muted-foreground">{{
                                            getJumpedAgo(jump)
                                        }}</span>
                                    </TooltipTrigger>
                                    <TooltipContent> {{ formatMass(jump.mass / 1_000_000) }} · {{ getJumpedAt(jump) }} </TooltipContent>
                                </Tooltip>
                            </div>
                        </div>
                        <div v-if="connection.jumps_count > jumps.length" class="pt-1 text-center text-xs text-muted-foreground">
                            Showing the latest {{ jumps.length }} of {{ connection.jumps_count }} jumps
                        </div>
                    </div>
                </PopoverContent>
            </Popover>
            <span v-else class="font-normal text-muted-foreground">{{ connection.jumps_count }} jumps</span>
        </div>
        <div class="space-y-1 text-xs text-muted-foreground">
            <template v-if="remainingPercent !== null && remainingMass !== null">
                <div class="h-1.5 w-full overflow-hidden rounded-full bg-muted">
                    <div class="h-full rounded-full transition-all" :class="barColor" :style="{ width: `${remainingPercent}%` }" />
                </div>
                <div class="flex justify-between py-1">
                    <span>≈ {{ formatMass(remainingMass) }} remaining</span>
                    <span>{{ Math.round(remainingPercent) }}%</span>
                </div>
            </template>
            <div v-else class="flex justify-between py-1">
                <span>Tracked jumped mass</span>
                <span class="text-right">{{ formatMass(jumpedMass) }}</span>
            </div>
        </div>
    </div>
</template>
