<script setup lang="ts">
import { PinInput, PinInputGroup, PinInputSeparator, PinInputSlot } from '@/components/ui/pin-input';

defineProps<{
    disabled?: boolean;
}>();

const model = defineModel<string>({
    required: true,
});

function handleUpdate(value: string[]) {
    model.value = [...value.slice(0, 3), '-', ...value.slice(3, 6)].join('').toUpperCase();
}
</script>

<template>
    <PinInput :model-value="model.replace('-', '').split('')" @update:modelValue="handleUpdate" :disabled>
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
</template>

<style scoped></style>
