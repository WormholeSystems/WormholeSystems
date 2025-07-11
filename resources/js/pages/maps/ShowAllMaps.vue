<script setup lang="ts">
import PlusIcon from '@/components/icons/PlusIcon.vue';
import SatelliteDish from '@/components/icons/SatelliteDish.vue';
import TelescopeIcon from '@/components/icons/TelescopeIcon.vue';
import { Badge } from '@/components/ui/badge';
import { Button } from '@/components/ui/button';
import { Card, CardContent, CardFooter, CardHeader, CardTitle } from '@/components/ui/card';
import AppLayout from '@/layouts/AppLayout.vue';
import { TMap } from '@/types/models';
import { Head, Link } from '@inertiajs/vue3';

defineProps<{
    maps: TMap[];
}>();

const getBadgeVariant = (count: number): 'default' | 'secondary' | 'destructive' => {
    if (count === 0) return 'secondary';
    if (count < 5) return 'default';
    return 'default';
};
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
                    <Link :href="route('maps.create')">
                        <Button class="flex items-center gap-2">
                            <PlusIcon class="h-4 w-4" />
                            Create New Map
                        </Button>
                    </Link>
                </div>
            </div>

            <!-- Maps Grid -->
            <div v-if="maps.length > 0" class="grid grid-cols-1 gap-6 md:grid-cols-2 lg:grid-cols-3">
                <Card v-for="map in maps" :key="map.id">
                    <CardHeader>
                        <Link class="group flex items-center gap-3" :href="route('maps.show', map.slug)">
                            <div class="rounded-lg bg-primary/10 p-2">
                                <TelescopeIcon class="h-5 w-5 text-primary" />
                            </div>
                            <div class="min-w-0 flex-1">
                                <CardTitle class="truncate transition-colors group-hover:text-primary">
                                    {{ map.name }}
                                </CardTitle>
                            </div>
                        </Link>
                    </CardHeader>

                    <CardContent>
                        <div class="grid grid-cols-2 gap-4">
                            <div class="flex items-center gap-2">
                                <SatelliteDish class="h-4 w-4 text-muted-foreground" />
                                <span class="text-sm text-muted-foreground">Systems</span>
                                <Badge :variant="getBadgeVariant(map.map_solarsystems_count!)">
                                    {{ map.map_solarsystems_count }}
                                </Badge>
                            </div>
                        </div>
                    </CardContent>
                    <CardFooter class="justify-between px-4">
                        <div class="flex items-center gap-2">
                            <div class="h-2 w-2 rounded-full bg-green-500"></div>
                            <span class="text-sm text-muted-foreground">Active</span>
                        </div>
                        <Button variant="ghost" as-child>
                            <Link :href="route('maps.show', map.slug)">View Map →</Link>
                        </Button>
                    </CardFooter>
                </Card>
            </div>

            <div v-else class="grid justify-items-center gap-4">
                <div class="grid size-24 place-items-center rounded-full bg-neutral-800 p-4">
                    <TelescopeIcon class="text-4xl" />
                </div>
                <h3 class="text-xl font-semibold">No maps yet</h3>
                <p class="text-muted-foreground">Get started by creating your first wormhole map to begin tracking connections and systems.</p>
                <Link :href="route('maps.create')">
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
