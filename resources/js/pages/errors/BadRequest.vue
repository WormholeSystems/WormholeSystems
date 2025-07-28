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
    <ErrorPage :status="status" title="Bad Request" :message="message" page-title="Bad Request - 400" />
</template>
