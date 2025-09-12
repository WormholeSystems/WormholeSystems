<script lang="ts" setup>
import { Toaster as Sonner, type ToasterProps } from 'vue-sonner';
import { buttonVariants } from '@/components/ui/button';
import { cn } from '@/lib/utils';

const props = defineProps<ToasterProps>();
</script>

<template>
    <Sonner
        class="toaster group"
        v-bind="props"
        :toastOptions="{
            unstyled: true,
            classes: {
                'toast': 'dark:bg-neutral-950 bg-white text-neutral-500 border text-sm border-border p-4 rounded-lg shadow-lg',
                title: 'text-base font-semibold dark:text-neutral-100 text-neutral-800',
                description: 'text-sm mt-2',
                icon: 'hidden',
                closeButton: 'hidden',
                actionButton: cn(buttonVariants({variant: 'default'}), 'mt-4')
            }
        }"
        descriptionClass="text-neutral-400"
    />
</template>
