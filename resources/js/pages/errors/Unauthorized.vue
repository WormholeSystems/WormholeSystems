<script setup lang="ts">
import ErrorPage from '@/components/ErrorPage.vue';

interface Props {
    status: number;
    query: string;
    message: string;
}

defineProps<Props>();
</script>

<template>
    <ErrorPage :status="status" title="Access Denied" :message="message" page-title="Unauthorized - 403" />
</template>

<style scoped></style>
