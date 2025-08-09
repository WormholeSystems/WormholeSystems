<script setup lang="ts">
import WormholeOption from '@/components/signatures/WormholeOption.vue';
import { Select, SelectContent, SelectGroup, SelectItem, SelectLabel, SelectSeparator, SelectTrigger } from '@/components/ui/select';
import { TWormholeDefinition } from '@/const/signatures';
import { computed, shallowRef } from 'vue';

const { possible_signatures } = defineProps<{
    can_write: boolean;
    possible_signatures: Record<string, TWormholeDefinition[]>;
}>();

const model = defineModel<string | null>({
    required: true,
});

const wormhole_options = computed(() => {
    return possible_signatures['Wormhole'] || [];
});

const k162_options = computed<TWormholeDefinition[]>(() => {
    return wormhole_options.value.filter((option: TWormholeDefinition) => option.signature === 'K162');
});

const wormholes = computed<TWormholeDefinition[]>(() => {
    return wormhole_options.value.filter((option: TWormholeDefinition) => option.signature !== 'K162');
});

const selected_signature = computed(() => {
    return wormhole_options.value.find((option: TWormholeDefinition) => option.name === model.value) || null;
});

const open = shallowRef(false);
</script>

<template>
    <Select v-model="model" :disabled="!can_write" v-model:open="open">
        <SelectTrigger class="w-full overflow-hidden">
            <WormholeOption v-if="selected_signature" :wormhole="selected_signature" />
            <template v-else>
                <span class="truncate">Type</span>
            </template>
        </SelectTrigger>
        <SelectContent class="max-h-100">
            <template v-if="open">
                <SelectGroup>
                    <SelectLabel class="text-muted-foreground">K162</SelectLabel>
                    <SelectItem v-for="option in k162_options" :key="option.name" :value="option.name">
                        <WormholeOption :wormhole="option" />
                    </SelectItem>
                </SelectGroup>
                <SelectSeparator />
                <SelectGroup>
                    <SelectLabel class="text-muted-foreground">Wormholes</SelectLabel>
                    <SelectItem v-for="option in wormholes" :key="option.name" :value="option.name">
                        <WormholeOption :wormhole="option" />
                    </SelectItem>
                </SelectGroup>
            </template>
        </SelectContent>
    </Select>
</template>

<style scoped></style>
