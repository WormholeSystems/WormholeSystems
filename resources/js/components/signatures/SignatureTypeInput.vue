<script setup lang="ts">
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from '@/components/ui/select';
import { TSignatureType } from '@/types/models';
import { computed, ref } from 'vue';

const { options, rawTypeName, category } = defineProps<{
    can_write: boolean;
    options: TSignatureType[];
    rawTypeName?: string | null;
    category?: string;
}>();

const model = defineModel<number | null>({
    required: true,
});

const hasRawTypeName = computed(() => !model.value && rawTypeName);

const open = ref(false);

const selected_option = computed(() => {
    return options.find((option) => option.id === model.value) || null;
});
</script>

<template>
    <Select v-model="model" :disabled="!can_write || !category" v-model:open="open">
        <SelectTrigger class="h-6 w-full text-xs">
            <span v-if="hasRawTypeName" class="truncate text-foreground">{{ rawTypeName }}</span>
            <SelectValue v-else placeholder="Type">
                <span v-if="selected_option" class="truncate">{{ selected_option?.name }}</span>
            </SelectValue>
        </SelectTrigger>
        <SelectContent class="max-h-72">
            <template v-if="open">
                <SelectItem v-for="option in options" :key="option.id" :value="option.id" class="text-xs">
                    {{ option.name }}
                </SelectItem>
                <SelectItem v-if="!options.length" :value="null" class="text-xs text-muted-foreground"> Unknown </SelectItem>
            </template>
        </SelectContent>
    </Select>
</template>

<style scoped></style>
