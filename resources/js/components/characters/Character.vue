<script setup lang="ts">
import { CharacterImage } from '@/components/images';
import TypeImage from '@/components/images/TypeImage.vue';
import UnknownImage from '@/components/images/UnknownImage.vue';
import { useMapSolarsystems } from '@/composables/map';
import { TCharacter } from '@/types/models';
import { vElementHover } from '@vueuse/components';
import { computed } from 'vue';

const { character } = defineProps<{
    character: TCharacter;
}>();

const { map_solarsystems, setHoveredMapSolarsystem } = useMapSolarsystems();

function onHover(hovered: boolean) {
    setHoveredMapSolarsystem(map_solarsystem.value!.id, hovered);
}

const map_solarsystem = computed(() => {
    return map_solarsystems.value.find((solarsystem) => solarsystem.solarsystem_id === character.status?.solarsystem_id);
});
</script>

<template>
    <div
        class="col-span-full grid h-12 grid-cols-subgrid items-center gap-2 border-b border-neutral-700 py-2 text-xs last:border-b-0"
        v-element-hover="onHover"
    >
        <div class="grid grid-cols-[auto_auto_auto] gap-0.5 gap-x-2">
            <TypeImage
                class="size-8 rounded-lg"
                v-if="character.status?.ship_type"
                :type_id="character.status.ship_type?.id"
                :type_name="character.status.ship_type.name"
            />
            <UnknownImage class="size-8 rounded-lg" v-else />
            <CharacterImage :character_id="character.id" :character_name="character.name" class="size-8" />
        </div>
        <div class="">
            <span>{{ character.name }}</span>
            <p class="text-muted-foreground">
                <span v-if="character.status?.ship_type">{{ character.status?.ship_type?.name }} | {{ character.status?.ship_name }}</span>
                <span v-else>Unknown Ship</span>
            </p>
        </div>
        <div class="text-right" v-if="map_solarsystem">
            <span>{{ map_solarsystem?.alias }} {{ character.status?.solarsystem?.name }}</span>
            <p class="text-muted-foreground">
                <span v-if="character.status?.station_id || character.status?.structure_id">Docked</span>
                <span v-else>In Space</span>
            </p>
        </div>
        <div v-else class="text-right">
            <span>Unknown</span>
            <p class="text-muted-foreground">-</p>
        </div>
    </div>
</template>

<style scoped></style>
