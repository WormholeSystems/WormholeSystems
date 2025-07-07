<script setup lang="ts">
import EsiImage from '@/components/images/EsiImage.vue';
import { TImageSize } from '@/components/images/index';

const {
    size = 64,
    character_id,
    character_name,
} = defineProps<{
    character_id: number;
    character_name: string;
    size?: TImageSize;
}>();
</script>

<template>
    <EsiImage :path="`characters/${character_id}/portrait`" :alt="character_name" :size />
</template>

<style scoped></style>
