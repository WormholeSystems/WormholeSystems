<script lang="ts" setup>
import { Toaster as Sonner, type ToasterProps } from 'vue-sonner';

const props = defineProps<ToasterProps>();
</script>

<template>
    <Sonner
        class="toaster group"
        v-bind="props"
        :toastOptions="{
            unstyled: true,
            classes: {
                'toast': 'bg-neutral-950 text-neutral-500 border text-sm border-border p-4 rounded-lg shadow-lg',
                title: 'text-base font-semibold text-neutral-100',
                description: 'text-sm mt-2',
                icon: 'hidden',
                closeButton: 'hidden',
            }
        }"
        descriptionClass="text-neutral-400"
    />
</template>
