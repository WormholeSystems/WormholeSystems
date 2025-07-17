<script setup lang="ts">
import { CharacterImage } from '@/components/images';
import TypeImage from '@/components/images/TypeImage.vue';
import { TableCell, TableRow } from '@/components/ui/table';
import { Tooltip, TooltipContent, TooltipTrigger } from '@/components/ui/tooltip';
import { useMapSolarsystems } from '@/composables/map';
import { TCharacter } from '@/types/models';
import { vElementHover } from '@vueuse/components';
import { computed } from 'vue';

const { character } = defineProps<{
    character: TCharacter;
}>();

const { map_solarsystems, setHoveredMapSolarsystem } = useMapSolarsystems();

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

function onHover(hovered: boolean) {
    if (map_solarsystem.value) {
        setHoveredMapSolarsystem(map_solarsystem.value.id, hovered);
    }
}
</script>

<template>
    <TableRow ref="row" v-element-hover="onHover" :data-inactive="is_inactive" class="data-[inactive=true]:opacity-20">
        <TableCell>
            <div class="flex gap-2">
                <CharacterImage :character_id="character.id" :character_name="character.name" class="size-6 rounded-lg" />
                {{ character.name }}
            </div>
        </TableCell>
        <TableCell>
            <div class="flex items-center">
                <Tooltip>
                    <TooltipTrigger as-child>
                        <span v-if="character.status?.ship_type" class="flex items-center gap-2">
                            <TypeImage
                                :type_id="character.status.ship_type.id"
                                :type_name="character.status.ship_type.name"
                                class="size-6 rounded-lg"
                            />
                            {{ character.status.ship_type.name }}</span
                        >
                        <span v-else>Unknown Ship</span>
                    </TooltipTrigger>
                    <TooltipContent>
                        <span v-if="character.status?.ship_name">{{ character.status.ship_name }}</span>
                        <span v-else>Unknown Ship Name</span>
                    </TooltipContent>
                </Tooltip>
            </div>
        </TableCell>
        <TableCell>
            <span v-if="map_solarsystem && map_solarsystem.alias" class="flex gap-2">
                <span>{{ map_solarsystem.alias }}</span>
                <span class="text-muted-foreground"> {{ map_solarsystem.name }}</span>
                <span v-if="is_docked" class="text-xs text-muted-foreground">(Docked)</span>
            </span>
            <span v-else-if="character.status?.solarsystem">
                <span>{{ character.status.solarsystem.name }}</span>
                <span v-if="is_docked" class="text-xs text-muted-foreground">(Docked)</span>
            </span>
            <span v-else>Unknown Location</span>
        </TableCell>
    </TableRow>
</template>

<style scoped></style>
