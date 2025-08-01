<script setup lang="ts">
import { useTemplateRef } from 'vue';

const handle = useTemplateRef('handle');

defineExpose({
    handle: handle!,
});
</script>

<template>
    <div
        ref="handle"
        class="absolute top-[1px] left-1/2 hidden h-2 w-12 -translate-x-1/2 -translate-y-1/2 cursor-move rounded border border-neutral-300 bg-white group-hover:z-50 group-hover:block dark:border-neutral-600 dark:bg-neutral-700"
    ></div>
</template>

<style scoped></style>
