<script setup lang="ts">
import SolarsystemSovereignty from '@/components/map/SolarsystemSovereignty.vue';
import SolarsystemClass from '@/components/solarsystem/SolarsystemClass.vue';
import SolarsystemEffect from '@/components/solarsystem/SolarsystemEffect.vue';
import { ComboboxEmpty, ComboboxItem, ComboboxList } from '@/components/ui/combobox';
import { TSolarsystem } from '@/pages/maps';

const { solarsystems } = defineProps<{
    solarsystems: TSolarsystem[];
}>();

const emit = defineEmits<{
    select: [system: TSolarsystem];
}>();

function handleSelect(system: TSolarsystem) {
    emit('select', system);
}
</script>

<template>
    <ComboboxList class="w-[600px]">
        <div class="grid grid-cols-[auto_1fr_auto] gap-2 p-2">
            <ComboboxEmpty class="col-span-full">No systems found.</ComboboxEmpty>
            <ComboboxItem
                v-for="system in solarsystems"
                :key="system.id"
                :value="system.name"
                @select="handleSelect(system)"
                class="col-span-full grid grid-cols-subgrid items-center py-1.5"
            >
                <div class="flex justify-center">
                    <SolarsystemClass :wormhole_class="system.class" :security="system.security" />
                </div>
                <div class="min-w-0 text-xs">
                    <span class="font-medium">{{ system.name }}</span>
                    <span class="text-muted-foreground"> · {{ system.region?.name }}</span>
                </div>
                <div class="flex justify-center">
                    <SolarsystemSovereignty v-if="system.sovereignty" :sovereignty="system.sovereignty" />
                    <SolarsystemEffect v-else-if="system.effect" :effect="system.effect.name" />
                </div>
            </ComboboxItem>
        </div>
    </ComboboxList>
</template>
