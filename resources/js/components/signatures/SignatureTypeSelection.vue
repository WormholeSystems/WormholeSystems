<script setup lang="ts">
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from '@/components/ui/select';
import type { TSignatureType } from '@/const/signatures';

const { options, category } = defineProps<{
    can_write: boolean;
    options: TSignatureType[];
    category: string | null | undefined;
}>();

const model = defineModel<number | null>({
    required: true,
});
</script>

<template>
    <Select v-model="model" :disabled="!can_write || !options.length">
        <SelectTrigger class="w-full text-xs">
            <SelectValue placeholder="Type" />
        </SelectTrigger>
        <SelectContent>
            <SelectItem v-for="option in options" :key="option.id" :value="option.id">
                {{ option.name }}
            </SelectItem>
        </SelectContent>
    </Select>
</template>

<style scoped></style>

