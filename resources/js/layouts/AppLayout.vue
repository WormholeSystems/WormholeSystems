<script setup lang="ts">
import { TooltipProvider } from '@/components/ui/tooltip';
import Notifications from '@/components/user/Notifications.vue';
import AppContent from '@/layouts/AppContent.vue';
import AppHeader from '@/layouts/AppHeader.vue';
import type { BreadcrumbItemType } from '@/types';

interface Props {
    breadcrumbs?: BreadcrumbItemType[];
}

withDefaults(defineProps<Props>(), {
    breadcrumbs: () => [],
});
</script>

<template>
    <TooltipProvider>
        <div class="flex flex-col">
            <AppHeader />
            <AppContent>
                <slot />
            </AppContent>
            <Notifications />
        </div>
    </TooltipProvider>
</template>
