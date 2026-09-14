<script setup lang="ts">
import MapStatisticsPageController from '@/actions/App/Http/Controllers/MapStatisticsPageController';
import MaintainerLeaderboard from '@/components/maps/statistics/MaintainerLeaderboard.vue';
import { Button } from '@/components/ui/button';
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card';
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from '@/components/ui/select';
import AppLayout from '@/layouts/AppLayout.vue';
import SeoHead from '@/layouts/SeoHead.vue';
import { TMapSummary } from '@/pages/maps';
import type { TMaintainerEntry, TMaintainerPeriodOption } from '@/types/models';
import { Link, router } from '@inertiajs/vue3';
import { ArrowLeft, Trophy } from 'lucide-vue-next';

const { map, period, available_periods, entries } = defineProps<{
    map: TMapSummary;
    period: string;
    available_periods: TMaintainerPeriodOption[];
    entries: TMaintainerEntry[];
}>();

function changePeriod(value: unknown): void {
    if (typeof value !== 'string' || value === period) return;

    router.visit(MapStatisticsPageController.show(map.slug, { query: { period: value } }), {
        only: ['period', 'entries'],
        preserveState: true,
        preserveScroll: true,
    });
}
</script>

<template>
    <AppLayout>
        <SeoHead
            :title="`Leaderboard - ${map.name}`"
            :description="`Monthly maintainer leaderboard for ${map.name}`"
            keywords="map leaderboard, maintainer points, eve online wormhole mapping"
        />
        <div class="mx-auto max-w-4xl px-4 py-8">
            <div class="mb-8">
                <Button variant="ghost" size="sm" as-child class="mb-4">
                    <Link :href="`/maps/${map.slug}`" prefetch>
                        <ArrowLeft class="mr-2 h-4 w-4" />
                        Back to Map
                    </Link>
                </Button>
                <h1 class="text-3xl font-bold tracking-tight">{{ map.name }}</h1>
                <p class="mt-2 text-muted-foreground">Maintainer leaderboard</p>
            </div>

            <Card class="gap-0 py-0">
                <CardHeader class="flex flex-col items-start justify-between gap-4 border-b py-4 sm:flex-row sm:items-center">
                    <div class="space-y-1">
                        <CardTitle class="flex items-center gap-2 text-lg">
                            <Trophy class="size-5 text-indigo-400" />
                            Maintainer leaderboard
                        </CardTitle>
                        <CardDescription
                            >Points for signatures added, updated, and removed. Anomalies don't count — only scanning does.</CardDescription
                        >
                    </div>
                    <Select :model-value="period" @update:model-value="changePeriod">
                        <SelectTrigger class="w-full sm:w-56"><SelectValue /></SelectTrigger>
                        <SelectContent>
                            <SelectItem v-for="option in available_periods" :key="option.value" :value="option.value">
                                {{ option.label }}{{ option.is_current ? ' (in progress)' : '' }}
                            </SelectItem>
                        </SelectContent>
                    </Select>
                </CardHeader>
                <CardContent class="p-4">
                    <MaintainerLeaderboard :entries="entries" />
                </CardContent>
            </Card>
        </div>
    </AppLayout>
</template>
