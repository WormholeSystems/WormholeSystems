<script setup lang="ts">
import { computed } from 'vue';

interface Props {
    class?: string;
}

const props = defineProps<Props>();
const className = computed(() => props.class);
</script>

<template>
    <main class="" :class="className">
        <slot />
    </main>
</template>
