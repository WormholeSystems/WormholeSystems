<script setup lang="ts">
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from '@/components/ui/select';
import { ref } from 'vue';

const { options } = defineProps<{
    options: string[];
    category: string | null;
    can_write: boolean;
}>();

const model = defineModel<string | null>({
    required: true,
});

const open = ref(false);
</script>

<template>
    <Select v-model:model-value="model" :disabled="!can_write || !category" v-model:open="open">
        <SelectTrigger class="col-span-2 w-full overflow-hidden">
            <SelectValue>{{ model ?? 'Type' }}</SelectValue>
        </SelectTrigger>
        <SelectContent v-if="category">
            <template v-if="open">
                <SelectItem v-for="option in options" :key="option" :value="option">
                    {{ option }}
                </SelectItem>
            </template>
        </SelectContent>
    </Select>
</template>

<style scoped></style>
