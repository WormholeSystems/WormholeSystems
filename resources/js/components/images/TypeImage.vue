<script setup lang="ts">
import EsiImage from '@/components/images/EsiImage.vue';
import { TImageSize, TTypeImageVariants } from '@/components/images/index';

const {
    size = 64,
    type_id,
    type_name,
    variant = 'icon',
} = defineProps<{
    type_id: number;
    type_name: string;
    size?: TImageSize;
    variant?: TTypeImageVariants;
}>();
</script>

<template>
    <EsiImage :path="`types/${type_id}/${variant}`" :alt="type_name" :size />
</template>

<style scoped></style>
