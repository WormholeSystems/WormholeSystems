<script setup lang="ts">
import MapController from '@/actions/App/Http/Controllers/MapController';
import PlusIcon from '@/components/icons/PlusIcon.vue';
import TelescopeIcon from '@/components/icons/TelescopeIcon.vue';
import MapCard from '@/components/MapCard.vue';
import { Button } from '@/components/ui/button';
import AppLayout from '@/layouts/AppLayout.vue';
import { TMap } from '@/types/models';
import { Head, Link } from '@inertiajs/vue3';

defineProps<{
    maps: TMap[];
}>();
</script>

<template>
    <AppLayout>
        <Head title="Maps" />
        <div class="p-8">
            <!-- Header Section -->
            <div class="mb-8">
                <div class="flex items-center justify-between">
                    <div>
                        <h1 class="text-3xl font-bold tracking-tight text-foreground">Maps</h1>
                        <p class="mt-2 text-muted-foreground">Manage and explore your wormhole mapping networks</p>
                    </div>
                    <Link :href="MapController.create()">
                        <Button class="flex items-center gap-2">
                            <PlusIcon class="h-4 w-4" />
                            Create New Map
                        </Button>
                    </Link>
                </div>
            </div>

            <!-- Maps Grid -->
            <div v-if="maps.length > 0" class="grid grid-cols-1 gap-6 md:grid-cols-2 lg:grid-cols-3">
                <MapCard v-for="map in maps" :key="map.id" :map="map" />
            </div>

            <div v-else class="grid justify-items-center gap-4">
                <div class="grid size-24 place-items-center rounded-full bg-neutral-800 p-4">
                    <TelescopeIcon class="text-4xl" />
                </div>
                <h3 class="text-xl font-semibold">No maps yet</h3>
                <p class="text-muted-foreground">Get started by creating your first wormhole map to begin tracking connections and systems.</p>
                <Link :href="MapController.create()">
                    <Button class="flex items-center gap-2">
                        <PlusIcon class="h-4 w-4" />
                        Create Your First Map
                    </Button>
                </Link>
            </div>
        </div>
    </AppLayout>
</template>

<style scoped></style>
