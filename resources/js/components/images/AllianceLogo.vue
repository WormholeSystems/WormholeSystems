<script setup lang="ts">
import QuestionIcon from '@/components/icons/QuestionIcon.vue';
import { UseImage } from '@vueuse/components';

const {
    size = 64,
    alliance_id,
    alliance_name,
} = defineProps<{
    alliance_id: number;
    alliance_name: string;
    size?: 64 | 128 | 256;
}>();
</script>

<template>
    <UseImage :src="`https://images.evetech.net/alliances/${alliance_id}/logo?size=${size}`" :alt="`Alliance logo for ${alliance_name}`">
        <template #error>
            <QuestionIcon />
        </template>
    </UseImage>
</template>

<style scoped></style>
