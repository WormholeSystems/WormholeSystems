<script setup lang="ts">
import { PinInput, PinInputGroup, PinInputSeparator, PinInputSlot } from '@/components/ui/pin-input';

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

function handleUpdate(value: string[]) {
    model.value = [...value.slice(0, 3), '-', ...value.slice(3, 6)].join('').toUpperCase();
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
    <div @focusout="handleBlur">
        <PinInput :model-value="model.replace('-', '').split('')" @update:modelValue="handleUpdate" :disabled @keydown="handleKeydown">
            <PinInputGroup>
                <PinInputSlot :index="0" />
                <PinInputSlot :index="1" />
                <PinInputSlot :index="2" />
                <PinInputSeparator />
                <PinInputSlot :index="3" />
                <PinInputSlot :index="4" />
                <PinInputSlot :index="5" />
            </PinInputGroup>
        </PinInput>
    </div>
</template>

<style scoped></style>
