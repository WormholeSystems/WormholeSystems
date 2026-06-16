<script setup lang="ts">
import SolarsystemSearchResult from '@/components/solarsystem/SolarsystemSearchResult.vue';
import { ComboboxEmpty, ComboboxItem, ComboboxList } from '@/components/ui/combobox';
import type { TStaticSolarsystem } from '@/types/static-data';

const { solarsystems, aliases } = defineProps<{
    solarsystems: TStaticSolarsystem[];
    aliases?: Map<number, string>;
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
        <ComboboxEmpty class="p-2">No systems found.</ComboboxEmpty>
        <div class="grid grid-cols-[auto_1fr_1fr_auto] items-center gap-x-2 p-1">
            <ComboboxItem
                v-for="system in solarsystems"
                :key="system.id"
                :value="system.name"
                @select="handleSelect(system)"
                class="col-span-full grid grid-cols-subgrid"
            >
                <SolarsystemSearchResult :solarsystem="system" :alias="aliases?.get(system.id) ?? null" />
            </ComboboxItem>
        </div>
    </ComboboxList>
</template>
