<script setup lang="ts">
import EsiImage from '@/components/images/EsiImage.vue';
import { TImageSize } from '@/components/images/index';

const {
    size = 64,
    corporation_id,
    corporation_name,
} = defineProps<{
    corporation_id: number;
    corporation_name: string;
    size?: TImageSize;
}>();
</script>

<template>
    <EsiImage :path="`corporations/${corporation_id}/logo`" :alt="corporation_name" :size />
</template>

<style scoped></style>
