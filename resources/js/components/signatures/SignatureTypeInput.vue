<script setup lang="ts">
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from '@/components/ui/select';
import { TSignatureType } from '@/types/models';
import { computed } from 'vue';

const { options, rawTypeName } = defineProps<{
    can_write: boolean;
    options: TSignatureType[];
    rawTypeName?: string | null;
}>();

const model = defineModel<number | null>({
    required: true,
});

// Check if we should show the raw type name as actual content
const hasRawTypeName = computed(() => !model.value && rawTypeName);
</script>

<template>
    <Select v-model="model" :disabled="!can_write || !options.length">
        <SelectTrigger class="w-full text-xs">
            <!-- Show raw type name as real content when no type is matched -->
            <span v-if="hasRawTypeName" class="text-foreground">{{ rawTypeName }}</span>
            <SelectValue v-else placeholder="Type" />
        </SelectTrigger>
        <SelectContent>
            <SelectItem v-for="option in options" :key="option.id" :value="option.id">
                {{ option.name }}
            </SelectItem>
        </SelectContent>
    </Select>
</template>

<style scoped></style>
