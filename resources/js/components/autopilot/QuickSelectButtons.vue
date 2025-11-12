<script setup lang="ts">
import SolarsystemClass from '@/components/solarsystem/SolarsystemClass.vue';
import { Button } from '@/components/ui/button';
import { TSelectedMapSolarsystem, TSolarsystem } from '@/pages/maps';
import { TMapRouteSolarsystem } from '@/types/models';
import { MapPin, Navigation } from 'lucide-vue-next';
import { computed } from 'vue';

const { selected_map_solarsystem, active_character_system, destinations } = defineProps<{
    selected_map_solarsystem?: TSelectedMapSolarsystem | null;
    active_character_system?: TSolarsystem | null;
    destinations: TMapRouteSolarsystem[];
}>();

const emit = defineEmits<{
    selectSystem: [system: TSolarsystem];
}>();

const pinnedDestinations = computed(() => {
    return destinations.filter((dest) => dest.is_pinned);
});

function handleSystemClick(system: TSolarsystem) {
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
            <SolarsystemClass
                :wormhole_class="selected_map_solarsystem.solarsystem.class"
                :security="selected_map_solarsystem.solarsystem.security"
                class="shrink-0"
            />
            <span class="text-xs">{{ selected_map_solarsystem.solarsystem.name }}</span>
        </Button>
        <Button v-if="active_character_system" variant="outline" size="xs" @click="handleSystemClick(active_character_system)" class="h-6 gap-1">
            <Navigation class="size-3 shrink-0 stroke-muted-foreground" />
            <SolarsystemClass :wormhole_class="active_character_system.class" :security="active_character_system.security" class="shrink-0" />
            <span class="text-xs">{{ active_character_system.name }}</span>
        </Button>
        <Button
            v-for="destination in pinnedDestinations"
            :key="destination.id"
            variant="outline"
            size="xs"
            @click="handleSystemClick(destination.solarsystem)"
            class="h-6 gap-1"
        >
            <SolarsystemClass :wormhole_class="destination.solarsystem.class" :security="destination.solarsystem.security" class="shrink-0" />
            <span class="text-xs">{{ destination.solarsystem.name }}</span>
        </Button>
    </div>
</template>
