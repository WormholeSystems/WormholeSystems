<script setup lang="ts">
import MapAccessController from '@/actions/App/Http/Controllers/MapAccessController';
import MapPreferencesController from '@/actions/App/Http/Controllers/MapPreferencesController';
import MapRoutingSettingsController from '@/actions/App/Http/Controllers/MapRoutingSettingsController';
import MapSettingsController from '@/actions/App/Http/Controllers/MapSettingsController';
import { Button } from '@/components/ui/button';
import usePermission from '@/composables/usePermission';
import AppLayout from '@/layouts/AppLayout.vue';
import SeoHead from '@/layouts/SeoHead.vue';
import { TMapSummary } from '@/pages/maps';
import { Link, usePage } from '@inertiajs/vue3';
import { ArrowLeft, Route, Settings, User, Users } from 'lucide-vue-next';
import { computed } from 'vue';

interface Props {
    map: TMapSummary;
    title: string;
    description?: string;
}

const props = defineProps<Props>();

const page = usePage();
const { canManageAccess } = usePermission();

const navigationItems = computed(() => {
    const items = [];

    // General - Manager+ only
    if (canManageAccess.value) {
        items.push({
            name: 'General',
            href: MapSettingsController.show(props.map.slug),
            icon: Settings,
            description: 'Map management and basic settings',
        });
    }

    // Preferences - Everyone
    items.push({
        name: 'Preferences',
        href: MapPreferencesController.show(props.map.slug),
        icon: User,
        description: 'Personal tracking and filters',
    });

    // Access - Manager+ only
    if (canManageAccess.value) {
        items.push({
            name: 'Access',
            href: MapAccessController.show(props.map.slug),
            icon: Users,
            description: 'Manage permissions',
        });
    }

    // Routing - Everyone
    items.push({
        name: 'Routing',
        href: MapRoutingSettingsController.show(props.map.slug),
        icon: Route,
        description: 'Route calculation preferences',
    });

    return items;
});

function isCurrentRoute(href: string) {
    return page.url.startsWith(href);
}
</script>

<template>
    <AppLayout>
        <SeoHead
            :title="`${title} - ${map.name}`"
            :description="description || `Configure ${title.toLowerCase()} for ${map.name}`"
            keywords="map settings, wormhole map configuration, eve online"
        />

        <div class="mx-auto max-w-7xl px-4 py-8">
            <!-- Header -->
            <div class="mb-8">
                <div class="mb-4 flex items-center gap-4">
                    <Button variant="ghost" size="sm" as-child>
                        <Link :href="`/maps/${map.slug}`">
                            <ArrowLeft class="mr-2 h-4 w-4" />
                            Back to Map
                        </Link>
                    </Button>
                </div>
                <h1 class="text-3xl font-bold tracking-tight">{{ map.name }} Settings</h1>
                <p class="mt-2 text-muted-foreground">{{ title }}</p>
            </div>

            <div class="grid grid-cols-1 gap-8 lg:grid-cols-4">
                <!-- Sidebar Navigation -->
                <div class="lg:col-span-1">
                    <nav class="space-y-2">
                        <div v-for="item in navigationItems" :key="item.name" class="block">
                            <Link
                                :href="item.href"
                                class="flex items-start gap-3 rounded-lg px-3 py-2 text-sm font-medium transition-colors hover:bg-accent hover:text-accent-foreground"
                                :class="{
                                    'bg-accent text-accent-foreground': isCurrentRoute(item.href.url),
                                    'text-muted-foreground': !isCurrentRoute(item.href.url),
                                }"
                            >
                                <component :is="item.icon" class="mt-0.5 h-4 w-4 flex-shrink-0" />
                                <div>
                                    <div class="font-medium">{{ item.name }}</div>
                                    <div class="mt-0.5 text-xs text-muted-foreground">
                                        {{ item.description }}
                                    </div>
                                </div>
                            </Link>
                        </div>
                    </nav>
                </div>

                <!-- Main Content -->
                <div class="lg:col-span-3">
                    <slot />
                </div>
            </div>
        </div>
    </AppLayout>
</template>
