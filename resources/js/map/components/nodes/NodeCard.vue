<script setup lang="ts">
import LockIcon from '@/components/icons/LockIcon.vue';
import SatelliteDish from '@/components/icons/SatelliteDish.vue';
import HasExtraConnections from '@/components/map/HasExtraConnections.vue';
import SolarsystemEffect from '@/components/map/SolarsystemEffect.vue';
import SolarsystemSovereignty from '@/components/map/SolarsystemSovereignty.vue';
import SolarsystemClass from '@/components/solarsystem/SolarsystemClass.vue';
import { Button } from '@/components/ui/button';
import { Input } from '@/components/ui/input';
import { Popover, PopoverAnchor, PopoverContent } from '@/components/ui/popover';
import { Tooltip, TooltipContent, TooltipTrigger } from '@/components/ui/tooltip';
import { isWormholeClass } from '@/const/solarsystemClasses';
import SolarsystemName from '@/map/components/solarsystem/SolarsystemName.vue';
import SolarsystemPilots from '@/map/components/solarsystem/SolarsystemPilots.vue';
import SolarsystemRegion from '@/map/components/solarsystem/SolarsystemRegion.vue';
import SolarsystemStatics from '@/map/components/solarsystem/SolarsystemStatics.vue';
import { TMapSolarsystem } from '@/pages/maps';
import MapSolarsystems from '@/routes/map-solarsystems';
import { TCharacter, TThreatLevel } from '@/types/models';
import { useForm } from '@inertiajs/vue3';
import { Flag as FlagIcon, Home as HomeIcon } from 'lucide-vue-next';
import { computed, ref, useTemplateRef } from 'vue';

/**
 * The visual node card. Fully presentational: everything it renders arrives via
 * props, measurement/selection/hover live in MapNode. Only the alias-edit
 * popover (a self-contained form against the card's own record) stays inside.
 */
const {
    system,
    pilots,
    threatLevel = null,
} = defineProps<{
    system: TMapSolarsystem;
    pilots: TCharacter[];
    isSelected: boolean;
    isHovered: boolean;
    isActive: boolean;
    isHome: boolean;
    isRally: boolean;
    fixedWidth: boolean;
    threatLevel?: TThreatLevel | null;
}>();

const form = useForm<{
    alias: string;
    occupier_alias: string;
}>({
    alias: system.alias ?? '',
    occupier_alias: system.occupier_alias ?? '',
});

const resolvedSolarsystem = computed(() => ({
    id: system.solarsystem?.id ?? system.solarsystem_id,
    name: system.solarsystem?.name ?? system.alias ?? '',
    security: system.solarsystem?.security ?? 0,
    class: system.solarsystem?.class ?? 'unknown',
    sovereignty: system.solarsystem?.sovereignty ?? null,
    region: system.solarsystem?.region ?? null,
    statics: system.solarsystem?.statics ?? null,
    effect: system.solarsystem?.effect ?? null,
}));

const open = ref(false);

/**
 * The form snapshots the record at mount, but the alias can change afterwards
 * (tracker dialog, another user's edit) — reseed it whenever the editor opens.
 */
function openEditor() {
    form.defaults({ alias: system.alias ?? '', occupier_alias: system.occupier_alias ?? '' });
    form.reset();
    open.value = true;
}

/** Exposed so MapNode can observe the card's border-box size for the store. */
const root = useTemplateRef<HTMLElement>('root');
defineExpose({ root });

const hasUncategorizedSignatures = computed(() => {
    return (system.uncategorized_signatures_count ?? 0) > 0;
});

const signatureTooltipText = computed(() => {
    const total = system.signatures_count;
    const uncategorized = system.uncategorized_signatures_count ?? 0;
    const signatureLabel = total === 1 ? 'signature' : 'signatures';

    return `${total} ${signatureLabel}${uncategorized ? ` ${uncategorized} uncategorized` : ''}`;
});

const signatureIconClass = computed(() => {
    return hasUncategorizedSignatures.value ? 'size-[14px] text-rose-500' : 'size-[14px] text-amber-500';
});

const extra_connections_count = computed(() => {
    const connections_count = system.wormhole_signatures_count ?? 0;
    const mapped_connections_count = system.map_connections_count ?? 0;
    return Math.max(0, connections_count - mapped_connections_count);
});

function handleSubmit() {
    form.put(MapSolarsystems.update(system.id).url, {
        onSuccess: () => {
            open.value = false;
        },
        preserveScroll: true,
        preserveState: true,
        only: ['map', 'selected_map_solarsystem'],
    });
}
</script>

<template>
    <div
        ref="root"
        :data-solarsystem-id="system.solarsystem_id"
        :data-selected="isSelected"
        :data-hovered="isHovered"
        :data-status="system.status"
        :data-has-pilots="pilots.length > 0"
        :data-is-active="isActive"
        :data-threat-level="threatLevel"
        class="grid h-[40px] rounded border border-neutral-300 bg-white text-left text-xs ring-offset-2 ring-offset-neutral-50 transition-colors duration-200 ease-in-out select-none hover:bg-white focus:bg-white data-[has-pilots=true]:h-[60px] data-[hovered=true]:outline-2 data-[hovered=true]:outline-yellow-500 data-[is-active=true]:ring-2 data-[is-active=true]:ring-amber-500 data-[selected=true]:bg-amber-100 data-[status=active]:border-active data-[status=empty]:border-empty data-[status=friendly]:border-friendly data-[status=hostile]:border-hostile data-[status=unknown]:border-unknown data-[is-active=false]:data-[threat-level=critical]:ring-2 data-[is-active=false]:data-[threat-level=critical]:ring-threat-critical data-[is-active=false]:data-[threat-level=high]:ring-2 data-[is-active=false]:data-[threat-level=high]:ring-threat-high dark:border-neutral-700 dark:bg-neutral-900 dark:ring-offset-neutral-900 dark:hover:bg-neutral-800 dark:focus:bg-neutral-800 dark:data-[is-active=true]:ring-2 dark:data-[is-active=true]:ring-amber-500 dark:data-[selected=true]:bg-amber-900 dark:data-[status=active]:border-active dark:data-[status=empty]:border-empty dark:data-[status=friendly]:border-friendly dark:data-[status=hostile]:border-hostile dark:data-[status=unscanned]:border-unscanned"
        :class="{ 'w-[180px]': fixedWidth }"
        @dblclick="openEditor()"
        @drag.prevent
    >
        <div class="row-start-1 grid grid-cols-[auto_1fr_auto] items-center justify-center gap-x-1 px-2">
            <SolarsystemClass :solarsystem_class="resolvedSolarsystem.class" />
            <Popover :open="open" @update:open="(value) => open && (open = value)">
                <PopoverAnchor class="col-start-2 row-start-1 min-w-0">
                    <SolarsystemName :map_solarsystem="system" :truncate="fixedWidth" />
                </PopoverAnchor>
                <PopoverContent>
                    <form @submit.prevent="handleSubmit" class="grid gap-2">
                        <Input v-model="form.alias" type="text" placeholder="Alias" class="w-full" />
                        <Input v-model="form.occupier_alias" type="text" placeholder="Occupier Alias" class="w-full" />
                        <Button type="submit"> Save</Button>
                    </form>
                </PopoverContent>
            </Popover>
            <div class="col-start-3 row-start-1 flex items-center gap-1">
                <Tooltip v-if="isHome" :delay-duration="500">
                    <TooltipTrigger>
                        <HomeIcon class="size-[14px] text-amber-400" />
                    </TooltipTrigger>
                    <TooltipContent> Home system </TooltipContent>
                </Tooltip>
                <Tooltip v-if="isRally" :delay-duration="500">
                    <TooltipTrigger>
                        <FlagIcon class="size-[14px] text-red-400" />
                    </TooltipTrigger>
                    <TooltipContent> Rally point </TooltipContent>
                </Tooltip>
                <Tooltip v-if="system.pinned" :delay-duration="500">
                    <TooltipTrigger>
                        <LockIcon class="size-[14px] text-muted-foreground" />
                    </TooltipTrigger>
                    <TooltipContent> Pinned in place </TooltipContent>
                </Tooltip>
                <Tooltip v-if="system.signatures_count" :delay-duration="500">
                    <TooltipTrigger>
                        <SatelliteDish :class="signatureIconClass" />
                    </TooltipTrigger>
                    <TooltipContent>{{ signatureTooltipText }}</TooltipContent>
                </Tooltip>
                <HasExtraConnections v-if="extra_connections_count" :extra_connections_count="extra_connections_count" />
                <SolarsystemSovereignty :sovereignty="resolvedSolarsystem.sovereignty" :solarsystem-id="resolvedSolarsystem.id">
                    <template #fallback>
                        <SolarsystemEffect :effect="resolvedSolarsystem.effect" v-if="resolvedSolarsystem.effect" />
                    </template>
                </SolarsystemSovereignty>
            </div>
            <SolarsystemRegion
                :region="resolvedSolarsystem.region"
                v-if="resolvedSolarsystem.region && !isWormholeClass(resolvedSolarsystem.class)"
            />
            <SolarsystemStatics v-else-if="resolvedSolarsystem.statics" :statics="resolvedSolarsystem.statics" />
        </div>
        <SolarsystemPilots v-if="pilots.length" :pilots />
    </div>
</template>

<style scoped></style>
