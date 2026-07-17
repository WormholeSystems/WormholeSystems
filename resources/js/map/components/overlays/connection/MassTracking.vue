<script setup lang="ts">
import { CharacterImage, TypeImage } from '@/components/images';
import { Button } from '@/components/ui/button';
import { Combobox, ComboboxAnchor, ComboboxInput, ComboboxItem, ComboboxVirtualList } from '@/components/ui/combobox';
import { DropdownMenu, DropdownMenuContent, DropdownMenuItem, DropdownMenuTrigger } from '@/components/ui/dropdown-menu';
import { Input } from '@/components/ui/input';
import { Popover, PopoverContent, PopoverTrigger } from '@/components/ui/popover';
import { Tooltip, TooltipContent, TooltipTrigger } from '@/components/ui/tooltip';
import { useEveSearch } from '@/composables/useEveSearch';
import { useNowUTC } from '@/composables/useNowUTC';
import { formatKilotons } from '@/lib/utils';
import { createMapConnectionJump, TMapConnectionJumpDirection } from '@/map/actions/createMapConnectionJump';
import { deleteMapConnectionJump } from '@/map/actions/deleteMapConnectionJump';
import { updateMapConnectionJump } from '@/map/actions/updateMapConnectionJump';
import { TConnectionJump, TMapConnection, TMapSolarsystem } from '@/pages/maps';
import { TEveSearchResult } from '@/types/models';
import { UTCDate } from '@date-fns/utc';
import { differenceInDays, differenceInHours, differenceInMinutes, format } from 'date-fns';
import { ChevronRight, EllipsisVertical, MoveLeft, MoveRight, Pencil, PencilLine, Plus, Trash2, X } from 'lucide-vue-next';
import { computed, ref, watch } from 'vue';

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

const isFormOpen = ref(false);
const editingJump = ref<TConnectionJump | null>(null);
const direction = ref<TMapConnectionJumpDirection>('outbound');
const shipTypeId = ref<number | null>(null);
const shipLabel = ref('');
const massKt = ref('');

/** EVE's "Ship" category: the manual jump log only ever records ships. */
const EVE_SHIP_CATEGORY_ID = 6;

const { results, search } = useEveSearch();
const shipTerm = ref('');

watch(shipTerm, (value) => void search('type', value, { category_id: EVE_SHIP_CATEGORY_ID }));

function systemLabel(system: TMapSolarsystem): string {
    return system.alias || system.solarsystem.name;
}

const directionLabel = computed(() => {
    const source = systemLabel(props.connection.source);
    const target = systemLabel(props.connection.target);
    return direction.value === 'outbound' ? `${source} → ${target}` : `${target} → ${source}`;
});

function flipDirection() {
    direction.value = direction.value === 'outbound' ? 'inbound' : 'outbound';
}

/** Resets the form for a fresh entry; the popover trigger itself toggles it open. */
function prepareAddForm() {
    editingJump.value = null;
    direction.value = 'outbound';
    shipTypeId.value = null;
    shipLabel.value = '';
    massKt.value = '';
}

function openEditForm(jump: TConnectionJump) {
    editingJump.value = jump;
    direction.value = isOutbound(jump) ? 'outbound' : 'inbound';
    shipTypeId.value = jump.ship_type_id;
    shipLabel.value = jump.ship_type_name ?? '';
    massKt.value = String(Math.round((jump.mass / 1_000_000) * 10) / 10);

    /* Deferred: the row menu is still closing when the item is selected, and its
     * dismiss/focus-restore would register as an outside interaction on a popover
     * opened in the same tick, closing it immediately.
     */
    window.setTimeout(() => {
        isFormOpen.value = true;
    });
}

function closeForm() {
    isFormOpen.value = false;
    editingJump.value = null;
}

function pickShip(result: TEveSearchResult) {
    shipTypeId.value = result.id;
    shipLabel.value = result.name;
    if (result.mass) {
        massKt.value = String(Math.round((result.mass / 1_000_000) * 10) / 10);
    }
    shipTerm.value = '';
    results.value = [];
}

function clearShip() {
    shipTypeId.value = null;
    shipLabel.value = '';
}

function resultName(result: TEveSearchResult): string {
    return result.name;
}

const massKg = computed<number | undefined>(() => {
    const parsed = Number(massKt.value);
    if (massKt.value === '' || !Number.isFinite(parsed) || parsed < 0) return undefined;
    return Math.round(parsed * 1_000_000);
});

const canSubmit = computed(() => massKg.value !== undefined || shipTypeId.value !== null);

function submitForm() {
    if (!canSubmit.value) return;

    const payload = {
        direction: direction.value,
        ship_type_id: shipTypeId.value,
        ...(massKg.value !== undefined ? { mass: massKg.value } : {}),
    };

    if (editingJump.value) {
        updateMapConnectionJump(editingJump.value, payload, { onSuccess: closeForm });
    } else {
        createMapConnectionJump(props.connection.id, payload, { onSuccess: closeForm });
    }
}
</script>

<template>
    <div class="space-y-1">
        <div class="flex items-center justify-between border-b pb-1 text-xs font-medium text-foreground">
            <Tooltip>
                <TooltipTrigger as-child>
                    <span class="cursor-help">Mass (estimate)</span>
                </TooltipTrigger>
                <TooltipContent> Only tracked pilots are counted automatically; the total mass varies by ±10% in game. </TooltipContent>
            </Tooltip>
            <Popover>
                <PopoverTrigger as-child>
                    <button
                        type="button"
                        class="flex items-center gap-0.5 rounded font-normal text-muted-foreground transition-colors hover:text-foreground"
                    >
                        {{ connection.jumps_count }} jumps
                        <ChevronRight class="size-3" />
                    </button>
                </PopoverTrigger>
                <PopoverContent class="w-96 p-0" side="right" align="start">
                    <div class="max-h-64 overflow-y-auto px-3">
                        <div class="grid grid-cols-[auto_auto_1fr_auto_auto_auto_auto] divide-y divide-border/40">
                            <div
                                class="sticky top-0 z-10 col-span-full grid grid-cols-subgrid gap-x-3 bg-popover py-1.5 text-[10px] font-medium tracking-wider text-muted-foreground uppercase"
                            >
                                <span class="col-span-3">Ship</span>
                                <span>Pilot</span>
                                <span class="text-right">kt</span>
                                <span class="text-right">Age</span>
                                <Popover v-model:open="isFormOpen">
                                    <PopoverTrigger as-child>
                                        <button
                                            type="button"
                                            class="flex items-center justify-end text-muted-foreground transition-colors hover:text-foreground"
                                            title="Log jump manually"
                                            @click="prepareAddForm"
                                        >
                                            <Plus class="size-3" />
                                        </button>
                                    </PopoverTrigger>
                                    <!-- focus-outside is prevented so the row menu's teardown can't dismiss the
                                         form; pointer interactions outside still close it as usual. -->
                                    <PopoverContent class="w-72 p-3" side="right" align="start" @focus-outside.prevent>
                                        <form class="space-y-2" @submit.prevent="submitForm">
                                            <div class="flex items-center justify-between gap-2">
                                                <span class="text-[10px] font-medium tracking-wider text-muted-foreground uppercase">
                                                    {{ editingJump ? 'Edit jump' : 'Log jump' }}
                                                </span>
                                                <Tooltip>
                                                    <TooltipTrigger as-child>
                                                        <button
                                                            type="button"
                                                            class="flex min-w-0 items-center gap-1 rounded text-xs text-muted-foreground transition-colors hover:text-foreground"
                                                            @click="flipDirection"
                                                        >
                                                            <component
                                                                :is="direction === 'outbound' ? MoveRight : MoveLeft"
                                                                class="size-3 shrink-0"
                                                                :class="direction === 'outbound' ? 'text-sky-500' : 'text-purple-500'"
                                                            />
                                                            <span class="truncate">{{ directionLabel }}</span>
                                                        </button>
                                                    </TooltipTrigger>
                                                    <TooltipContent> Click to flip the jump direction </TooltipContent>
                                                </Tooltip>
                                            </div>
                                            <div v-if="shipTypeId" class="flex items-center gap-1.5 rounded-md border px-2 py-1">
                                                <TypeImage :type_id="shipTypeId" :type_name="shipLabel" class="size-4 rounded" />
                                                <span class="min-w-0 flex-1 truncate text-xs">{{ shipLabel }}</span>
                                                <button type="button" class="text-muted-foreground hover:text-foreground" @click="clearShip">
                                                    <X class="size-3" />
                                                </button>
                                            </div>
                                            <Combobox v-else :ignore-filter="true" class="rounded-md border">
                                                <ComboboxAnchor>
                                                    <ComboboxInput v-model="shipTerm" placeholder="Search ship type…" />
                                                </ComboboxAnchor>
                                                <ComboboxVirtualList align="start" :options="results" :estimate-size="32" :text-content="resultName">
                                                    <template #empty>{{
                                                        shipTerm.trim().length > 0 ? 'No matches' : 'Start typing to search…'
                                                    }}</template>
                                                    <template #default="{ option }">
                                                        <ComboboxItem
                                                            :value="option.name"
                                                            class="flex w-full items-center justify-between gap-3"
                                                            @select.prevent="() => pickShip(option)"
                                                        >
                                                            <span class="truncate">{{ option.name }}</span>
                                                            <span v-if="option.group_name" class="shrink-0 text-xs text-muted-foreground">
                                                                {{ option.group_name }}
                                                            </span>
                                                        </ComboboxItem>
                                                    </template>
                                                </ComboboxVirtualList>
                                            </Combobox>
                                            <div class="flex items-center gap-2">
                                                <Input
                                                    v-model="massKt"
                                                    type="number"
                                                    min="0"
                                                    step="any"
                                                    placeholder="Mass"
                                                    class="h-7 flex-1 text-xs"
                                                />
                                                <span class="text-xs text-muted-foreground">kt</span>
                                                <Button type="submit" size="xs" :disabled="!canSubmit">Save</Button>
                                                <Button type="button" size="xs" variant="ghost" @click="closeForm">Cancel</Button>
                                            </div>
                                        </form>
                                    </PopoverContent>
                                </Popover>
                            </div>
                            <div
                                v-for="jump in jumps"
                                :key="jump.id"
                                class="group col-span-full grid grid-cols-subgrid items-center gap-x-3 py-1.5 transition-colors hover:bg-muted/30"
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
                                <div v-if="jump.character_id" class="flex items-center gap-1.5">
                                    <CharacterImage
                                        :character_id="jump.character_id"
                                        :character_name="jump.character_name || ''"
                                        class="size-4 shrink-0 rounded-full"
                                    />
                                    <span class="max-w-20 truncate text-[10px] text-muted-foreground">{{ jump.character_name }}</span>
                                </div>
                                <div v-else class="flex items-center gap-1 text-[10px] text-muted-foreground italic">
                                    <PencilLine class="size-3 shrink-0" />
                                    manual
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
                                <DropdownMenu>
                                    <DropdownMenuTrigger as-child>
                                        <button
                                            type="button"
                                            class="flex items-center justify-end rounded p-0.5 text-muted-foreground transition-colors hover:text-foreground"
                                            title="Jump actions"
                                        >
                                            <EllipsisVertical class="size-3" />
                                        </button>
                                    </DropdownMenuTrigger>
                                    <!-- close-auto-focus is prevented so the closing menu doesn't return focus
                                         to the row trigger, which would dismiss the edit popover it just opened. -->
                                    <DropdownMenuContent side="right" align="start" @close-auto-focus.prevent>
                                        <DropdownMenuItem @select="openEditForm(jump)">
                                            <Pencil class="size-3.5" />
                                            Edit
                                        </DropdownMenuItem>
                                        <DropdownMenuItem variant="destructive" @select="deleteMapConnectionJump(jump)">
                                            <Trash2 class="size-3.5" />
                                            Delete
                                        </DropdownMenuItem>
                                    </DropdownMenuContent>
                                </DropdownMenu>
                            </div>
                            <div v-if="!jumps.length" class="col-span-full py-3 text-center text-xs text-muted-foreground">No jumps logged yet</div>
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
