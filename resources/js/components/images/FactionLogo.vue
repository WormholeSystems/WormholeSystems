<script setup lang="ts">
import EsiImage from '@/components/images/EsiImage.vue';
import { TImageSize } from '@/components/images/index';

const {
    size = 64,
    faction_id,
    faction_name,
} = defineProps<{
    faction_id: number;
    faction_name: string;
    size?: TImageSize;
}>();
</script>

<template>
    <EsiImage :path="`corporations/${faction_id}/logo`" :alt="faction_name" :size />
</template>

<style scoped></style>
