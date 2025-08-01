<script setup lang="ts">
import { useTemplateRef } from 'vue';

const new_connection_handle = useTemplateRef('new_connection_handle');
defineExpose({
    new_connection_handle: new_connection_handle!,
});
</script>

<template>
    <div
        ref="new_connection_handle"
        class="absolute top-1/2 left-full hidden h-4 w-4 -translate-x-1/2 -translate-y-1/2 cursor-pointer rounded-full border border-neutral-300 bg-white group-hover:block dark:border-neutral-600 dark:bg-neutral-700"
    ></div>
</template>

<style scoped></style>
