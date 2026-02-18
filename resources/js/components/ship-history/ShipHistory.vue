<script setup lang="ts">
import { CharacterImage, TypeImage } from '@/components/images';
import MapPanel from '@/components/ui/map-panel/MapPanel.vue';
import MapPanelContent from '@/components/ui/map-panel/MapPanelContent.vue';
import MapPanelHeader from '@/components/ui/map-panel/MapPanelHeader.vue';
import { TShowMapProps } from '@/pages/maps';
import { AppPageProps } from '@/types';
import { UTCDate } from '@date-fns/utc';
import { usePage } from '@inertiajs/vue3';
import { useNow } from '@vueuse/core';
import { differenceInDays, differenceInHours, differenceInMinutes, format } from 'date-fns';
import { computed } from 'vue';

const page = usePage<AppPageProps<TShowMapProps>>();

const now = useNow({ interval: 60_000 });

function formatTimeAgo(date: string) {
    const d = new UTCDate(date);
    const diffInMinutes = differenceInMinutes(now.value, d);
    const diffInHours = differenceInHours(now.value, d);
    const diffInDays = differenceInDays(now.value, d);

    if (diffInDays > 0) return `${diffInDays}d ago`;
    if (diffInHours > 0) return `${diffInHours}h ago`;
    if (diffInMinutes > 0) return `${diffInMinutes}m ago`;
    return 'just now';
}

const shipHistory = computed(() => page.props.ship_history ?? []);
</script>

<template>
    <MapPanel>
        <MapPanelHeader card-id="ship-history">
            Ship History
            <span v-if="shipHistory.length" class="ml-1 text-amber-400">{{ shipHistory.length }}</span>
        </MapPanelHeader>
        <MapPanelContent>
            <template v-if="shipHistory.length">
                <div
                    v-for="entry in shipHistory"
                    :key="entry.id"
                    class="flex items-center gap-2 border-b border-border/30 px-3 py-1.5 hover:bg-muted/30"
                >
                    <TypeImage :type_id="entry.ship_type_id" class="size-5 shrink-0 rounded" :type_name="entry.ship_type?.name || ''" />
                    <div class="flex min-w-0 flex-1 flex-col">
                        <span class="truncate text-xs">{{ entry.name || entry.ship_type?.name || 'Unknown' }}</span>
                        <span class="truncate font-mono text-[10px] text-muted-foreground">{{ entry.ship_type?.name }}</span>
                    </div>
                    <div class="flex items-center gap-1">
                        <CharacterImage
                            v-if="entry.character"
                            :character_id="entry.character.id"
                            :character_name="entry.character.name"
                            class="size-4 shrink-0 rounded"
                        />
                        <span class="max-w-20 truncate font-mono text-[10px] text-muted-foreground">{{ entry.character?.name ?? 'Unknown' }}</span>
                    </div>
                    <span class="font-mono text-[10px] text-muted-foreground" :title="format(new UTCDate(entry.updated_at), 'MMM dd, HH:mm')">
                        {{ formatTimeAgo(entry.updated_at) }}
                    </span>
                </div>
            </template>
            <div v-else class="flex h-full flex-col items-center justify-center gap-2 p-4">
                <p class="font-mono text-[10px] tracking-wider text-muted-foreground/60 uppercase">No ship history</p>
            </div>
        </MapPanelContent>
    </MapPanel>
</template>
