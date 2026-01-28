<script setup lang="ts">
import DestinationContextMenu from '@/components/autopilot/DestinationContextMenu.vue';
import RoutePopover from '@/components/autopilot/RoutePopover.vue';
import { CharacterImage } from '@/components/images';
import TypeImage from '@/components/images/TypeImage.vue';
import SolarsystemSovereignty from '@/components/map/SolarsystemSovereignty.vue';
import SolarsystemEffect from '@/components/solarsystem/SolarsystemEffect.vue';
import { Tooltip, TooltipContent, TooltipTrigger } from '@/components/ui/tooltip';
import { useMapSolarsystems } from '@/composables/map';
import { usePath } from '@/composables/usePath';
import { useStaticSolarsystem } from '@/composables/useStaticSolarsystems';
import { TCharacter } from '@/types/models';
import { vElementHover } from '@vueuse/components';
import { computed } from 'vue';

const { character } = defineProps<{
    character: TCharacter;
}>();

const { map_solarsystems, setHoveredMapSolarsystem } = useMapSolarsystems();
const statusSolarsystem = useStaticSolarsystem(() => character.status?.solarsystem_id ?? null);

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
            ref="row"
            v-element-hover="onHover"
            :data-inactive="is_inactive"
            class="flex items-center gap-2 border-b border-border/30 px-3 py-1.5 hover:bg-muted/30 data-[inactive=true]:opacity-50"
        >
            <CharacterImage :character_id="character.id" :character_name="character.name" class="size-5 shrink-0 rounded" />

            <span class="flex-1 truncate text-xs">{{ character.name }}</span>

            <Tooltip>
                <TooltipTrigger as-child>
                    <div v-if="character.status?.ship_type" class="flex items-center gap-1">
                        <TypeImage :type_id="character.status.ship_type.id" :type_name="character.status.ship_type.name" class="size-4" />
                        <span class="max-w-24 truncate font-mono text-[10px] text-muted-foreground">
                            {{ character.status.ship_type.name }}
                        </span>
                    </div>
                </TooltipTrigger>
                <TooltipContent>
                    <span v-if="character.status?.ship_name">{{ character.status.ship_name }}</span>
                    <span v-else>Unknown Ship Name</span>
                </TooltipContent>
            </Tooltip>

            <SolarsystemSovereignty
                v-if="statusSolarsystem"
                :sovereignty="statusSolarsystem.sovereignty"
                :solarsystem-id="statusSolarsystem.id"
                class="size-4 shrink-0"
            >
                <template #fallback>
                    <SolarsystemEffect v-if="statusSolarsystem.effect" :effect="statusSolarsystem.effect.name" />
                </template>
            </SolarsystemSovereignty>

            <RoutePopover :route="character.route">
                <button v-element-hover="onRouteHover" class="flex items-center gap-1 text-xs hover:text-foreground">
                    <span v-if="map_solarsystem?.alias" class="font-mono text-xs font-medium">{{ map_solarsystem.alias }}</span>
                    <span v-else-if="statusSolarsystem" class="font-mono text-xs">{{ statusSolarsystem.name }}</span>
                    <span v-else class="text-muted-foreground">--</span>
                    <span v-if="is_docked" class="text-[10px] text-muted-foreground">(D)</span>
                    <span v-else-if="is_scanner" class="text-[10px] text-amber-400">(S)</span>
                </button>
            </RoutePopover>
        </div>
    </DestinationContextMenu>
</template>

<style scoped></style>
