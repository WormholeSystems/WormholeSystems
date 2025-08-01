<script setup lang="ts">
import { TRegion } from '@/types/models';

defineProps<{
    region: TRegion;
}>();
</script>

<template>
    <span class="col-span-2 block text-xs text-muted-foreground">{{ region.name }}</span>
</template>

<style scoped></style>
