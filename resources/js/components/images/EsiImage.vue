<script setup lang="ts">
import QuestionIcon from '@/components/icons/QuestionIcon.vue';
import { TImageSize } from '@/components/images/index';
import { UseImage } from '@vueuse/components';

const {
    size = 64,
    path,
    alt,
} = defineProps<{
    size?: TImageSize;
    path: string;
    alt: string;
}>();
</script>

<template>
    <UseImage :src="`https://images.evetech.net/${path}?size=${size}`" :alt>
        <template #error>
            <QuestionIcon />
        </template>
    </UseImage>
</template>

<style scoped></style>
