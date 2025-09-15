<script setup lang="ts">
import ErrorPage from '@/pages/errors/ErrorPage.vue';

interface Props {
    status: number;
    query: string;
    message: string;
}

defineProps<Props>();
</script>

<template>
    <ErrorPage :status="status" title="Server Error" :message="message" page-title="Server Error - 500" />
</template>

<style scoped></style>
