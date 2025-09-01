<script setup lang="ts">
import { Input } from '@/components/ui/input';

const { currentValue } = defineProps<{
    disabled?: boolean;
    currentValue: string | null;
}>();

const model = defineModel<string>({
    required: true,
});

const emit = defineEmits<{
    (e: 'submit'): void;
}>();

function handleInput(event: Event) {
    const target = event.target as HTMLInputElement;
    let value = target.value.replace(/[^a-zA-Z0-9]/g, '').toUpperCase();

    // Format as XXX-XXX if length is appropriate
    if (value.length >= 4) {
        value = value.slice(0, 3) + '-' + value.slice(3, 6);
    }

    model.value = value;
}

function handleSubmit() {
    emit('submit');
}

function handleKeydown(event: KeyboardEvent) {
    if (event.key === 'Enter' && currentValue !== model.value) {
        handleSubmit();
    }
}

function handleBlur() {
    if (currentValue !== model.value && (model.value.length === 7 || model.value.length === 0)) {
        handleSubmit();
    }
}
</script>

<template>
    <Input
        :model-value="model"
        @input="handleInput"
        @keydown="handleKeydown"
        @blur="handleBlur"
        :disabled="disabled"
        placeholder="XXX-XXX"
        maxlength="7"
        class="w-[10ch] font-mono"
    />
</template>

<style scoped></style>
