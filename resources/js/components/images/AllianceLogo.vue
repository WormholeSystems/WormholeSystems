<script setup lang="ts">
import EsiImage from '@/components/images/EsiImage.vue';
import { TImageSize } from '@/components/images/index';

const {
    size = 64,
    alliance_id,
    alliance_name,
} = defineProps<{
    alliance_id: number;
    alliance_name: string | null;
    size?: TImageSize;
}>();
</script>

<template>
    <EsiImage :path="`alliances/${alliance_id}/logo`" :alt="alliance_name ?? 'Alliance'" :size />
</template>

<style scoped></style>
