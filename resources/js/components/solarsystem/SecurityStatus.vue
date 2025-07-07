<script setup lang="ts">
import { computed } from 'vue';

const { security } = defineProps<{
    security: number;
}>();

const securityStatus = computed(() => {
    if (security >= 0.5) return 'hs'; // High Security
    if (security >= 0.1) return 'ls'; // Low Security
    return 'ns'; // Null Security
});
</script>

<template>
    <span :data-security="securityStatus" class="data-[security=hs]:text-hs data-[security=ls]:text-ls data-[security=ns]:text-ns">
        {{ security.toFixed(1) }}
    </span>
</template>

<style scoped></style>
