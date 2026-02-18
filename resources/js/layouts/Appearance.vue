<script setup lang="ts">
import MoonIcon from '@/components/icons/MoonIcon.vue';
import SunIcon from '@/components/icons/SunIcon.vue';
import { useAppearance } from '@/composables/useAppearance';

const { appearance, updateAppearance } = useAppearance();
</script>

<template>
    <button
        @click="updateAppearance(appearance === 'dark' ? 'light' : 'dark')"
        class="flex items-center justify-center rounded-md p-1.5 text-muted-foreground transition-colors hover:bg-muted/50 hover:text-foreground"
    >
        <span class="sr-only">Toggle Dark Mode</span>
        <SunIcon v-if="appearance === 'dark'" class="size-4" />
        <MoonIcon v-else class="size-4" />
    </button>
</template>

<style scoped></style>
