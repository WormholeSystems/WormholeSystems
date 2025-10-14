<script setup lang="ts">
import WormholeOption from '@/components/signatures/WormholeOption.vue';
import { Select, SelectContent, SelectGroup, SelectItem, SelectLabel, SelectSeparator, SelectTrigger } from '@/components/ui/select';
import type { TSignatureType } from '@/types/models';
import { computed, shallowRef } from 'vue';

const { wormhole_options, current_class } = defineProps<{
    can_write: boolean;
    wormhole_options: TSignatureType[];
    current_class: string | number | null;
}>();

const model = defineModel<number | null>({
    required: true,
});

const k162_options = computed<TSignatureType[]>(() => {
    return wormhole_options.filter((option: TSignatureType) => option.signature === 'K162').filter(filterByCurrentClass);
});

const wormholes = computed<TSignatureType[]>(() => {
    return wormhole_options.filter((option: TSignatureType) => option.signature !== 'K162').filter(filterByCurrentClass);
});

const selected_signature = computed(() => {
    return wormhole_options.find((option: TSignatureType) => option.id === model.value) || null;
});

const open = shallowRef(false);

function filterByCurrentClass(option: TSignatureType) {
    if (!current_class) {
        return true;
    }
    return option.target_class === current_class.toString();
}
</script>

<template>
    <Select v-model="model" :disabled="!can_write" v-model:open="open">
        <SelectTrigger class="w-full text-xs">
            <WormholeOption v-if="selected_signature" :wormhole="selected_signature" />
            <template v-else>
                <span class="truncate">Type</span>
            </template>
        </SelectTrigger>
        <SelectContent class="max-h-100">
            <template v-if="open">
                <SelectGroup>
                    <SelectLabel class="text-muted-foreground">K162</SelectLabel>
                    <SelectItem v-for="option in k162_options" :key="option.id" :value="option.id">
                        <WormholeOption :wormhole="option" />
                    </SelectItem>
                </SelectGroup>
                <SelectSeparator />
                <SelectGroup>
                    <SelectLabel class="text-muted-foreground">Wormholes</SelectLabel>
                    <SelectItem v-for="option in wormholes" :key="option.id" :value="option.id">
                        <WormholeOption :wormhole="option" />
                    </SelectItem>
                </SelectGroup>
            </template>
        </SelectContent>
    </Select>
</template>

<style scoped></style>
