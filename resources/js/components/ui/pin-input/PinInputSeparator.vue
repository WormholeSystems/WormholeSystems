<script setup lang="ts">
import { Primitive, type PrimitiveProps, useForwardProps } from 'reka-ui';

const props = defineProps<PrimitiveProps>();
const forwardedProps = useForwardProps(props);
</script>

<template>
    <Primitive
        data-slot="pin-input-separator"
        v-bind="forwardedProps"
        class="w-4 text-center"
    >
        <slot>
            -
        </slot>
    </Primitive>
</template>
