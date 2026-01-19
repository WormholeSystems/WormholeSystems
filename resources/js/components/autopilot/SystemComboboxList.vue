<script setup lang="ts">
import SolarsystemSovereignty from '@/components/map/SolarsystemSovereignty.vue';
import SolarsystemClass from '@/components/solarsystem/SolarsystemClass.vue';
import SolarsystemEffect from '@/components/solarsystem/SolarsystemEffect.vue';
import { ComboboxEmpty, ComboboxItem, ComboboxList } from '@/components/ui/combobox';
import type { TStaticSolarsystem } from '@/types/static-data';

const { solarsystems } = defineProps<{
    solarsystems: TStaticSolarsystem[];
}>();

const emit = defineEmits<{
    select: [system: TStaticSolarsystem];
}>();

function handleSelect(system: TStaticSolarsystem) {
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
                    <SolarsystemSovereignty :sovereignty="system.sovereignty" :solarsystem-id="system.id">
                        <template #fallback>
                            <SolarsystemEffect v-if="system.effect" :effect="system.effect.name" />
                        </template>
                    </SolarsystemSovereignty>
                </div>
            </ComboboxItem>
        </div>
    </ComboboxList>
</template>
