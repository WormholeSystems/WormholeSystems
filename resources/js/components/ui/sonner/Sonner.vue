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
        :close-button="true"
        :toastOptions="{
            unstyled: true,
            classes: {
                toast: 'bg-card text-foreground border border-border/50 border-l-2 border-l-muted-foreground text-sm px-4 py-3 shadow-lg flex items-start gap-3 relative',
                title: 'text-sm font-medium text-foreground',
                description: 'text-xs text-muted-foreground mt-1',
                icon: 'hidden',
                closeButton: 'absolute top-2 right-2 size-5 flex items-center justify-center text-muted-foreground/60 hover:text-foreground transition-colors',
                actionButton: cn(buttonVariants({variant: 'default', size: 'sm'}), 'mt-3'),
                success: 'border-l-green-500',
                error: 'border-l-red-500',
                warning: 'border-l-amber-500',
                info: 'border-l-blue-500',
            }
        }"
    />
</template>
