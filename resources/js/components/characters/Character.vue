<script setup lang="ts">
import DestinationContextMenu from '@/components/autopilot/DestinationContextMenu.vue';
import RoutePopover from '@/components/autopilot/RoutePopover.vue';
import { CharacterImage } from '@/components/images';
import TypeImage from '@/components/images/TypeImage.vue';
import SolarsystemSovereignty from '@/components/map/SolarsystemSovereignty.vue';
import SolarsystemClass from '@/components/solarsystem/SolarsystemClass.vue';
import SolarsystemEffect from '@/components/solarsystem/SolarsystemEffect.vue';
import { Tooltip, TooltipContent, TooltipTrigger } from '@/components/ui/tooltip';
import { useMapSolarsystems } from '@/composables/map';
import { usePath } from '@/composables/usePath';
import { TCharacter } from '@/types/models';
import type { TStaticSolarsystem } from '@/types/static-data';
import { vElementHover } from '@vueuse/components';
import { computed } from 'vue';

const { character, static_solarsystem } = defineProps<{
    character: TCharacter;
    static_solarsystem: TStaticSolarsystem | null;
}>();

const { map_solarsystems, setHoveredMapSolarsystem } = useMapSolarsystems();

const { setPath } = usePath();

const map_solarsystem = computed(() => {
    return map_solarsystems.value.find((solarsystem) => solarsystem.solarsystem_id === character.status?.solarsystem_id);
});

const is_docked = computed(() => {
    return !!(character.status?.structure_id || character.status?.station_id);
});

const is_inactive = computed(() => {
    if (is_docked.value) return true;

    return character.status?.ship_type?.name === 'Capsule';
});

const is_scanner = computed(() => {
    return character.status?.ship_type?.group_id === 830;
});

const jump_count = computed(() => {
    if (!character.route || character.route.length <= 1) {
        return null;
    }

    return character.route.length - 1;
});

const next_hop = computed(() => {
    if (!character.route || character.route.length < 2) {
        return null;
    }

    return character.route[1];
});

function onHover(hovered: boolean) {
    if (map_solarsystem.value) {
        setHoveredMapSolarsystem(map_solarsystem.value.id, hovered);
    }
}

function onRouteHover(hovered: boolean) {
    if (hovered) {
        const routeToShow = character.route ?? [];
        setPath(routeToShow);
    } else {
        setPath(null);
    }
}
</script>

<template>
    <DestinationContextMenu :solarsystem_id="character.status?.solarsystem_id ?? 0">
        <div
            v-element-hover="onHover"
            :data-inactive="is_inactive"
            class="col-span-full grid grid-cols-subgrid items-center gap-2 border-b border-border/30 px-3 py-1.5 hover:bg-muted/30 data-[inactive=true]:opacity-50"
        >
            <CharacterImage :character_id="character.id" :character_name="character.name" class="size-5 shrink-0 rounded" />

            <div class="flex min-w-0 items-center gap-1">
                <span class="truncate text-xs">{{ character.name }}</span>
                <span v-if="is_docked" class="shrink-0 text-[10px] text-muted-foreground">(D)</span>
                <Tooltip v-else-if="is_scanner">
                    <TooltipTrigger as-child>
                        <span class="shrink-0 text-[10px] text-amber-400">(S)</span>
                    </TooltipTrigger>
                    <TooltipContent>Scanner</TooltipContent>
                </Tooltip>
            </div>

            <Tooltip>
                <TooltipTrigger as-child>
                    <div v-if="character.status?.ship_type" class="flex min-w-0 items-center gap-1">
                        <TypeImage :type_id="character.status.ship_type.id" :type_name="character.status.ship_type.name" class="size-4 shrink-0" />
                        <span class="truncate font-mono text-[10px] text-muted-foreground">
                            {{ character.status.ship_type.name }}
                        </span>
                    </div>
                    <div v-else></div>
                </TooltipTrigger>
                <TooltipContent>
                    <span v-if="character.status?.ship_name">{{ character.status.ship_name }}</span>
                    <span v-else>Unknown Ship Name</span>
                </TooltipContent>
            </Tooltip>

            <div class="justify-self-center">
                <SolarsystemClass v-if="static_solarsystem" :wormhole_class="static_solarsystem.class" :security="static_solarsystem.security" />
            </div>

            <div class="min-w-0 truncate text-xs">
                <span v-if="map_solarsystem?.alias" class="font-medium">{{ map_solarsystem.alias }}</span>
                <span v-else-if="static_solarsystem" class="font-medium">{{ static_solarsystem.name }}</span>
                <span v-else class="text-muted-foreground">--</span>
                <span v-if="static_solarsystem?.region" class="text-muted-foreground"> · {{ static_solarsystem.region.name }}</span>
            </div>

            <div class="flex items-center justify-center">
                <SolarsystemSovereignty
                    v-if="static_solarsystem"
                    :sovereignty="static_solarsystem.sovereignty"
                    :solarsystem-id="static_solarsystem.id"
                    class="size-4"
                >
                    <template #fallback>
                        <SolarsystemEffect v-if="static_solarsystem.effect" :effect="static_solarsystem.effect.name" />
                    </template>
                </SolarsystemSovereignty>
            </div>

            <RoutePopover :route="character.route">
                <button
                    v-element-hover="onRouteHover"
                    class="flex min-w-0 cursor-pointer items-center gap-1.5 justify-self-end hover:text-foreground"
                >
                    <span v-if="next_hop" class="truncate text-[10px] text-muted-foreground">{{ next_hop.name }}</span>
                    <span
                        v-if="jump_count !== null"
                        class="shrink-0 font-mono text-xs font-medium"
                        :class="{
                            'text-green-400': jump_count < 8,
                            'text-amber-400': jump_count >= 8 && jump_count < 15,
                            'text-red-400': jump_count >= 15,
                        }"
                    >
                        {{ jump_count }}j
                    </span>
                    <span v-else class="font-mono text-[10px] text-muted-foreground/60">--</span>
                </button>
            </RoutePopover>
        </div>
    </DestinationContextMenu>
</template>

<style scoped></style>
