<script setup lang="ts">
import MoonIcon from '@/components/icons/MoonIcon.vue';
import SunIcon from '@/components/icons/SunIcon.vue';
import { Button } from '@/components/ui/button';
import { useAppearance } from '@/composables/useAppearance';

const { appearance, updateAppearance } = useAppearance();
</script>

<template>
    <Button @click="updateAppearance(appearance === 'dark' ? 'light' : 'dark')" variant="outline" size="icon" class="h-9 w-9">
        <span class="sr-only">Toggle Dark Mode</span>
        <SunIcon v-if="appearance === 'dark'" class="h-4 w-4" />
        <MoonIcon v-else class="h-4 w-4" />
    </Button>
</template>

<style scoped></style>
