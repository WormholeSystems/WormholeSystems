<script setup lang="ts">
import { Toaster } from '@/components/ui/sonner';
import { useNotifications } from '@/composables/useNotifications';
import 'vue-sonner/style.css';

useNotifications();
</script>

<template>
    <Toaster position="top-right" />
</template>

<style scoped></style>
