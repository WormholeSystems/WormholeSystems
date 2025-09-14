<script setup lang="ts">
import DestinationContextMenu from '@/components/DestinationContextMenu.vue';
import { CharacterImage } from '@/components/images';
import TypeImage from '@/components/images/TypeImage.vue';
import RoutePopover from '@/components/routes/RoutePopover.vue';
import { Button } from '@/components/ui/button';
import { Tooltip, TooltipContent, TooltipTrigger } from '@/components/ui/tooltip';
import { useMapSolarsystems } from '@/composables/map';
import { usePath } from '@/composables/usePath';
import { TCharacter } from '@/types/models';
import { vElementHover } from '@vueuse/components';
import { computed } from 'vue';

const { character } = defineProps<{
    character: TCharacter;
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
            class="col-span-full grid grid-cols-subgrid border-b px-2 py-2 hover:bg-muted/50 data-[inactive=true]:opacity-50"
        >
            <div class="flex items-center gap-2">
                <CharacterImage :character_id="character.id" :character_name="character.name" class="size-6 shrink-0 rounded-lg" />
                <span class="truncate">{{ character.name }}</span>
            </div>

            <Tooltip>
                <TooltipTrigger as-child>
                    <span v-if="character.status?.ship_type" class="flex items-center gap-2 truncate">
                        <TypeImage
                            :type_id="character.status.ship_type.id"
                            :type_name="character.status.ship_type.name"
                            class="size-6 shrink-0 rounded-lg"
                        />
                        <span class="truncate">{{ character.status.ship_type.name }}</span>
                    </span>
                    <span v-else>Unknown Ship</span>
                </TooltipTrigger>
                <TooltipContent>
                    <span v-if="character.status?.ship_name">{{ character.status.ship_name }}</span>
                    <span v-else>Unknown Ship Name</span>
                </TooltipContent>
            </Tooltip>

            <RoutePopover :route="character.route">
                <Button variant="secondary" v-element-hover="onRouteHover" class="h-auto min-h-8 justify-start truncate">
                    <span v-if="map_solarsystem && map_solarsystem.alias" class="flex items-center gap-2 truncate">
                        <span class="truncate">{{ map_solarsystem.alias }}</span>
                        <span class="truncate text-muted-foreground">{{ map_solarsystem.name }}</span>
                        <span v-if="is_docked" class="text-xs text-muted-foreground">(Docked)</span>
                        <span v-else-if="is_scanner" class="text-xs text-amber-500">(Scanner)</span>
                    </span>
                    <span v-else-if="character.status?.solarsystem" class="flex items-center gap-2">
                        <span class="truncate">{{ character.status.solarsystem.name }}</span>
                        <span v-if="is_docked" class="text-xs text-muted-foreground">(Docked)</span>
                        <span v-else-if="is_scanner" class="text-xs text-amber-500">(Scanner)</span>
                    </span>
                </Button>
            </RoutePopover>
            <span v-if="!map_solarsystem && !character.status?.solarsystem" class="text-muted-foreground">Unknown Location</span>
        </div>
    </DestinationContextMenu>
</template>

<style scoped></style>
