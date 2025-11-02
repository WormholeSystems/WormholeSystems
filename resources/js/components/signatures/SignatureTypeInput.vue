<script setup lang="ts">
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from '@/components/ui/select';
import { TSignatureType } from '@/types/models';
import { computed, ref } from 'vue';

const { options, rawTypeName } = defineProps<{
    can_write: boolean;
    options: TSignatureType[];
    rawTypeName?: string | null;
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
    <Select v-model="model" :disabled="!can_write || !options.length" v-model:open="open">
        <SelectTrigger class="w-full text-xs">
            <span v-if="hasRawTypeName" class="text-foreground">{{ rawTypeName }}</span>
            <SelectValue v-else placeholder="Type">
                <span>{{ selected_option?.name }}</span>
            </SelectValue>
        </SelectTrigger>
        <SelectContent>
            <template v-if="open">
                <SelectItem v-for="option in options" :key="option.id" :value="option.id">
                    {{ option.name }}
                </SelectItem>
            </template>
        </SelectContent>
    </Select>
</template>

<style scoped></style>
