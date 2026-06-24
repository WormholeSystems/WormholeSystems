<script setup lang="ts">
import LockIcon from '@/components/icons/LockIcon.vue';
import SatelliteDish from '@/components/icons/SatelliteDish.vue';
import HasExtraConnections from '@/components/map/HasExtraConnections.vue';
import SolarsystemEffect from '@/components/map/SolarsystemEffect.vue';
import SolarsystemSovereignty from '@/components/map/SolarsystemSovereignty.vue';
import SolarsystemName from '@/components/map/solarsystem/SolarsystemName.vue';
import SolarsystemPilots from '@/components/map/solarsystem/SolarsystemPilots.vue';
import SolarsystemRegion from '@/components/map/solarsystem/SolarsystemRegion.vue';
import SolarsystemStatics from '@/components/map/solarsystem/SolarsystemStatics.vue';
import SolarsystemClass from '@/components/solarsystem/SolarsystemClass.vue';
import { Button } from '@/components/ui/button';
import { Input } from '@/components/ui/input';
import { Popover, PopoverAnchor, PopoverContent } from '@/components/ui/popover';
import { Tooltip, TooltipContent, TooltipTrigger } from '@/components/ui/tooltip';
import { forgetNodeSize, isSystemHovered, isSystemSelected, reportNodeSize, TDataMapSolarSystem, useMapViewMode } from '@/composables/map';
import { isWormholeClass } from '@/const/solarsystemClasses';
import MapSolarsystems from '@/routes/map-solarsystems';
import { TCharacter } from '@/types/models';
import { useForm, usePage } from '@inertiajs/vue3';
import { useElementSize } from '@vueuse/core';
import { Flag as FlagIcon, Home as HomeIcon } from 'lucide-vue-next';
import { computed, onBeforeUnmount, ref, useTemplateRef, watch } from 'vue';

const { map_solarsystem, interactive = true } = defineProps<{
    map_solarsystem: TDataMapSolarSystem;
    pilots: TCharacter[];
    is_active?: boolean;
    is_home?: boolean;
    is_rally?: boolean;
    /** False for static renders (the landing/printable MapView), which must not
     * write into the live map's node-size store or follow its layout mode. */
    interactive?: boolean;
}>();

const form = useForm<{
    alias: string;
    occupier_alias: string;
}>({
    alias: map_solarsystem.alias ?? '',
    occupier_alias: map_solarsystem.occupier_alias ?? '',
});

const resolvedSolarsystem = computed(() => ({
    id: map_solarsystem.solarsystem?.id ?? map_solarsystem.solarsystem_id,
    name: map_solarsystem.solarsystem?.name ?? map_solarsystem.alias ?? '',
    security: map_solarsystem.solarsystem?.security ?? 0,
    class: map_solarsystem.solarsystem?.class ?? 'unknown',
    sovereignty: map_solarsystem.solarsystem?.sovereignty ?? null,
    region: map_solarsystem.solarsystem?.region ?? null,
    statics: map_solarsystem.solarsystem?.statics ?? null,
    effect: map_solarsystem.solarsystem?.effect ?? null,
}));

const page = usePage();
const effectiveThreatLevel = computed(() => {
    const settings = page.props.map_user_settings as { show_threat_level?: boolean } | undefined;
    return settings?.show_threat_level ? (map_solarsystem.threat_level ?? null) : null;
});

const open = ref(false);

const { is_tree_layout } = useMapViewMode();

// The tree layout's fixed width / truncation only applies to the live map; a static
// render must ignore the persisted view mode so the landing map isn't forced into it.
const usesTreeStyle = computed(() => interactive && is_tree_layout.value);

// Selection and hover are derived from interaction state (only on the live map), so
// flipping them re-renders just this node instead of rebuilding the whole array.
const is_selected = computed(() => interactive && isSystemSelected(map_solarsystem));
const is_hovered = computed(() => interactive && isSystemHovered(map_solarsystem.id));

const root = useTemplateRef<HTMLElement>('root');
if (interactive) {
    // Report the node's rendered (base-unit) size so connections can attach to its
    // real edge. The scale lives on an ancestor, so this measures the unscaled box.
    const { width, height } = useElementSize(root, undefined, { box: 'border-box' });
    watch([width, height], ([w, h]) => {
        if (w > 0 && h > 0) reportNodeSize(map_solarsystem.id, { width: w, height: h });
    });
    onBeforeUnmount(() => forgetNodeSize(map_solarsystem.id));
}

const hasUncategorizedSignatures = computed(() => {
    return (map_solarsystem.uncategorized_signatures_count ?? 0) > 0;
});

const signatureTooltipText = computed(() => {
    const total = map_solarsystem.signatures_count;
    const uncategorized = map_solarsystem.uncategorized_signatures_count ?? 0;
    const signatureLabel = total === 1 ? 'signature' : 'signatures';

    return `${total} ${signatureLabel}${uncategorized ? ` ${uncategorized} uncategorized` : ''}`;
});

const signatureIconClass = computed(() => {
    return hasUncategorizedSignatures.value ? 'size-[14px] text-rose-500' : 'size-[14px] text-amber-500';
});

const extra_connections_count = computed(() => {
    const connections_count = map_solarsystem.wormhole_signatures_count ?? 0;
    const mapped_connections_count = map_solarsystem.map_connections_count ?? 0;
    return Math.max(0, connections_count - mapped_connections_count);
});

function handleSubmit() {
    form.put(MapSolarsystems.update(map_solarsystem.id).url, {
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
        :data-solarsystem-id="map_solarsystem.solarsystem_id"
        :data-selected="is_selected"
        :data-hovered="is_hovered"
        :data-status="map_solarsystem.status"
        :data-has-pilots="pilots.length > 0"
        :data-is-active="!!is_active"
        :data-threat-level="effectiveThreatLevel"
        class="grid h-[40px] rounded border border-neutral-300 bg-white text-left text-xs ring-offset-2 ring-offset-neutral-50 transition-colors duration-200 ease-in-out select-none hover:bg-white focus:bg-white data-[has-pilots=true]:h-[60px] data-[hovered=true]:outline-2 data-[hovered=true]:outline-yellow-500 data-[is-active=true]:ring-2 data-[is-active=true]:ring-amber-500 data-[selected=true]:bg-amber-100 data-[status=active]:border-active data-[status=empty]:border-empty data-[status=friendly]:border-friendly data-[status=hostile]:border-hostile data-[status=unknown]:border-unknown data-[is-active=false]:data-[threat-level=critical]:ring-2 data-[is-active=false]:data-[threat-level=critical]:ring-threat-critical data-[is-active=false]:data-[threat-level=high]:ring-2 data-[is-active=false]:data-[threat-level=high]:ring-threat-high dark:border-neutral-700 dark:bg-neutral-900 dark:ring-offset-neutral-900 dark:hover:bg-neutral-800 dark:focus:bg-neutral-800 dark:data-[is-active=true]:ring-2 dark:data-[is-active=true]:ring-amber-500 dark:data-[selected=true]:bg-amber-900 dark:data-[status=active]:border-active dark:data-[status=empty]:border-empty dark:data-[status=friendly]:border-friendly dark:data-[status=hostile]:border-hostile dark:data-[status=unscanned]:border-unscanned"
        :class="{ 'w-[180px]': usesTreeStyle }"
        @dblclick="open = true"
        @drag.prevent
    >
        <div class="row-start-1 grid grid-cols-[auto_1fr_auto] items-center justify-center gap-x-1 px-2">
            <SolarsystemClass :solarsystem_class="resolvedSolarsystem.class" />
            <Popover :open="open" @update:open="(value) => open && (open = value)">
                <PopoverAnchor class="col-start-2 row-start-1 min-w-0">
                    <SolarsystemName :map_solarsystem="map_solarsystem" :truncate="usesTreeStyle" />
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
                <Tooltip v-if="is_home" :delay-duration="500">
                    <TooltipTrigger>
                        <HomeIcon class="size-[14px] text-amber-400" />
                    </TooltipTrigger>
                    <TooltipContent> Home system </TooltipContent>
                </Tooltip>
                <Tooltip v-if="is_rally" :delay-duration="500">
                    <TooltipTrigger>
                        <FlagIcon class="size-[14px] text-red-400" />
                    </TooltipTrigger>
                    <TooltipContent> Rally point </TooltipContent>
                </Tooltip>
                <Tooltip v-if="map_solarsystem.pinned" :delay-duration="500">
                    <TooltipTrigger>
                        <LockIcon class="size-[14px] text-muted-foreground" />
                    </TooltipTrigger>
                    <TooltipContent> Pinned in place </TooltipContent>
                </Tooltip>
                <Tooltip v-if="map_solarsystem.signatures_count" :delay-duration="500">
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
