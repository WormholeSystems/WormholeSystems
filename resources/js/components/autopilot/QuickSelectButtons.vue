<script setup lang="ts">
import SolarsystemClass from '@/components/solarsystem/SolarsystemClass.vue';
import { Button } from '@/components/ui/button';
import { useSolarsystemAliases } from '@/composables/useSolarsystemAliases';
import { useMapSolarsystems } from '@/map/api';
import type { TResolvedMapRouteSolarsystem, TResolvedSelectedMapSolarsystem, TResolvedSolarsystem } from '@/pages/maps';
import { MapPin, Navigation } from 'lucide-vue-next';
import { computed } from 'vue';

const { selected_map_solarsystem, active_character_system, destinations } = defineProps<{
    selected_map_solarsystem?: TResolvedSelectedMapSolarsystem | null;
    active_character_system?: TResolvedSolarsystem | null;
    destinations: TResolvedMapRouteSolarsystem[];
}>();

const emit = defineEmits<{
    selectSystem: [system: TResolvedSolarsystem];
}>();

const { map_solarsystems } = useMapSolarsystems();
const { getAlias } = useSolarsystemAliases(map_solarsystems);

const pinnedDestinations = computed(() => {
    return destinations.filter((dest) => dest.is_pinned);
});

function handleSystemClick(system: TResolvedSolarsystem) {
    emit('selectSystem', system);
}
</script>

<template>
    <div v-if="selected_map_solarsystem?.solarsystem || active_character_system || pinnedDestinations.length > 0" class="mt-1.5 flex flex-wrap gap-1">
        <Button
            v-if="selected_map_solarsystem?.solarsystem"
            variant="outline"
            size="xs"
            @click="handleSystemClick(selected_map_solarsystem.solarsystem)"
            class="h-6 gap-1"
        >
            <MapPin class="size-3 shrink-0 stroke-muted-foreground" />
            <SolarsystemClass :solarsystem_class="selected_map_solarsystem.solarsystem.class" class="shrink-0" />
            <span class="text-xs">
                <span v-if="getAlias(selected_map_solarsystem.solarsystem.id)" class="mr-1">{{
                    getAlias(selected_map_solarsystem.solarsystem.id)
                }}</span>
                <span :class="{ 'text-muted-foreground': getAlias(selected_map_solarsystem.solarsystem.id) }">{{
                    selected_map_solarsystem.solarsystem.name
                }}</span>
            </span>
        </Button>
        <Button v-if="active_character_system" variant="outline" size="xs" @click="handleSystemClick(active_character_system)" class="h-6 gap-1">
            <Navigation class="size-3 shrink-0 stroke-muted-foreground" />
            <SolarsystemClass :solarsystem_class="active_character_system.class" class="shrink-0" />
            <span class="text-xs">
                <span v-if="getAlias(active_character_system.id)" class="mr-1">{{ getAlias(active_character_system.id) }}</span>
                <span :class="{ 'text-muted-foreground': getAlias(active_character_system.id) }">{{ active_character_system.name }}</span>
            </span>
        </Button>
        <Button
            v-for="destination in pinnedDestinations"
            :key="destination.id"
            variant="outline"
            size="xs"
            @click="handleSystemClick(destination.solarsystem)"
            class="h-6 gap-1"
        >
            <SolarsystemClass :solarsystem_class="destination.solarsystem.class" class="shrink-0" />
            <span class="text-xs">
                <span v-if="getAlias(destination.solarsystem.id)" class="mr-1">{{ getAlias(destination.solarsystem.id) }}</span>
                <span :class="{ 'text-muted-foreground': getAlias(destination.solarsystem.id) }">{{ destination.solarsystem.name }}</span>
            </span>
        </Button>
    </div>
</template>
