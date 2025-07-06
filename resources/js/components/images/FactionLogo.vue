<script setup lang="ts">
import QuestionIcon from '@/components/icons/QuestionIcon.vue';
import { UseImage } from '@vueuse/components';

const {
    size = 64,
    faction_id,
    faction_name,
} = defineProps<{
    faction_id: number;
    faction_name: string;
    size?: 64 | 128 | 256;
}>();
</script>

<template>
    <UseImage :src="`https://images.evetech.net/corporations/${faction_id}/logo?size=${size}`" :alt="`Alliance logo for ${faction_name}`">
        <template #error>
            <QuestionIcon />
        </template>
    </UseImage>
</template>

<style scoped></style>
