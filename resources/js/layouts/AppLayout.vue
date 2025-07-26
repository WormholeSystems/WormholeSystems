<script setup lang="ts">
import Notifications from '@/components/Notifications.vue';
import AppContent from '@/layouts/AppContent.vue';
import AppHeader from '@/layouts/AppHeader.vue';
import AppShell from '@/layouts/AppShell.vue';
import type { BreadcrumbItemType } from '@/types';

interface Props {
    breadcrumbs?: BreadcrumbItemType[];
}

withDefaults(defineProps<Props>(), {
    breadcrumbs: () => [],
});
</script>

<template>
    <AppShell class="flex-col">
        <AppHeader :breadcrumbs="breadcrumbs" />
        <AppContent>
            <slot />
        </AppContent>
        <Notifications />
    </AppShell>
</template>
