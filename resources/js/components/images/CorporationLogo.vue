<script setup lang="ts">
import QuestionIcon from '@/components/icons/QuestionIcon.vue';
import { UseImage } from '@vueuse/components';

const {
    size = 64,
    corporation_id,
    corporation_name,
} = defineProps<{
    corporation_id: number;
    corporation_name: string;
    size?: 64 | 128 | 256;
}>();
</script>

<template>
    <UseImage :src="`https://images.evetech.net/corporations/${corporation_id}/logo?size=${size}`" :alt="`Alliance logo for ${corporation_name}`">
        <template #error>
            <QuestionIcon />
        </template>
    </UseImage>
</template>

<style scoped></style>
