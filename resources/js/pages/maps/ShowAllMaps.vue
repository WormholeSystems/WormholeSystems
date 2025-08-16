<script setup lang="ts">
import MapController from '@/actions/App/Http/Controllers/MapController';
import Logo from '@/components/icons/Logo.vue';
import PlusIcon from '@/components/icons/PlusIcon.vue';
import MapCard from '@/components/MapCard.vue';
import SeoHead from '@/components/SeoHead.vue';
import { Button } from '@/components/ui/button';
import { Input } from '@/components/ui/input';
import { useSearch } from '@/composables/useSearch';
import AppLayout from '@/layouts/AppLayout.vue';
import { TMap } from '@/types/models';
import { Link } from '@inertiajs/vue3';
import { SearchIcon } from 'lucide-vue-next';
import { computed } from 'vue';

const { maps } = defineProps<{
    maps: TMap[];
}>();

const search = useSearch('search', ['maps']);

const filteredMapsCount = computed(() => maps.length);
</script>

<template>
    <AppLayout>
        <SeoHead
            title="Maps"
            description="Manage and explore your wormhole mapping networks. Create, edit, and navigate through dangerous wormhole space with collaborative mapping tools."
            keywords="wormhole maps, eve online mapping, wormhole navigation, space exploration"
        />
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

            <!-- Search and Filter Section -->
            <div class="mb-6">
                <div class="flex flex-col gap-4 sm:flex-row sm:items-center sm:justify-between">
                    <div class="relative max-w-md flex-1">
                        <SearchIcon class="absolute top-1/2 left-3 h-4 w-4 -translate-y-1/2 text-muted-foreground" />
                        <Input v-model="search" placeholder="Search maps..." class="pl-9" />
                    </div>
                    <div class="flex items-center gap-2 text-sm text-muted-foreground">
                        <span v-if="search"> {{ filteredMapsCount }} of {{ maps.length }} maps </span>
                        <span v-else> {{ maps.length }} maps total </span>
                    </div>
                </div>
            </div>

            <!-- Maps Grid -->
            <div v-if="maps.length > 0" class="grid grid-cols-1 gap-6 md:grid-cols-2 lg:grid-cols-3">
                <MapCard v-for="map in maps" :key="map.id" :map="map" />
            </div>

            <!-- Empty State -->
            <div v-else class="grid justify-items-center gap-4">
                <div class="grid size-24 place-items-center rounded-full bg-neutral-200 p-4 dark:bg-neutral-800" v-if="search.length">
                    <SearchIcon class="h-8 w-8 text-muted-foreground" />
                </div>
                <Logo class="h-16 w-16 text-muted-foreground" v-else />
                <h3 class="text-xl font-semibold">
                    {{ search ? 'No maps found' : 'No maps yet' }}
                </h3>
                <p class="max-w-md text-center text-muted-foreground">
                    <template v-if="search">
                        No maps match your search for "{{ search }}". Try adjusting your search terms or create a new map.
                    </template>
                    <template v-else> Get started by creating your first wormhole map to begin tracking connections and systems. </template>
                </p>
                <div class="flex gap-2">
                    <Button v-if="search" variant="outline" @click="search = ''"> Clear Search</Button>
                    <Link :href="MapController.create()">
                        <Button class="flex items-center gap-2">
                            <PlusIcon class="h-4 w-4" />
                            {{ search ? 'Create New Map' : 'Create Your First Map' }}
                        </Button>
                    </Link>
                </div>
            </div>
        </div>
    </AppLayout>
</template>

<style scoped></style>
