<script setup lang="ts">
import { useCommandPalette } from '@/composables/useCommandPalette';
import { Search } from 'lucide-vue-next';

const { openPalette } = useCommandPalette();

const isMac = typeof navigator !== 'undefined' && navigator.userAgent.toUpperCase().includes('MAC');
</script>

<template>
    <button
        type="button"
        class="flex w-full max-w-sm items-center gap-2 rounded border border-border/50 bg-background px-3 py-1.5 text-sm text-muted-foreground hover:border-border hover:text-foreground"
        @click="openPalette"
    >
        <Search class="size-4 shrink-0" />
        <span class="flex-1 text-left">Search...</span>
        <kbd class="shrink-0 rounded border border-border/50 px-1.5 py-0.5 font-mono text-[10px]">{{ isMac ? '⌘K' : 'Ctrl K' }}</kbd>
    </button>
</template>

<style scoped></style>
