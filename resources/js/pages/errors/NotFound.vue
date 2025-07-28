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
    <ErrorPage :status="status" title="Page Not Found" :message="message" page-title="Not Found - 404" />
</template>
