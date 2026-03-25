<script setup lang="ts">
import { AllianceLogo, CorporationLogo } from '@/components/images';
import MapPanel from '@/components/ui/map-panel/MapPanel.vue';
import MapPanelContent from '@/components/ui/map-panel/MapPanelContent.vue';
import MapPanelHeader from '@/components/ui/map-panel/MapPanelHeader.vue';
import { TThreatAnalysis } from '@/types/models';
import { UTCDate } from '@date-fns/utc';
import { Deferred } from '@inertiajs/vue3';
import { formatDistanceToNow } from 'date-fns';
import { ExternalLink, Globe, MapPin } from 'lucide-vue-next';

const { threat_analysis } = defineProps<{
    threat_analysis?: TThreatAnalysis | null;
}>();

function analyzedTimeAgo(): string | null {
    if (!threat_analysis?.threat_analyzed_at) return null;
    return formatDistanceToNow(new UTCDate(threat_analysis.threat_analyzed_at), { addSuffix: true });
}

function threatLevelLabel(level: string): string {
    return level.charAt(0).toUpperCase() + level.slice(1);
}

function threatLevelColor(level: string): string {
    const colors: Record<string, string> = {
        critical: 'text-red-600',
        high: 'text-orange-500',
        unknown: 'text-muted-foreground',
    };
    return colors[level] ?? 'text-muted-foreground';
}

function threatLevelBg(level: string): string {
    const colors: Record<string, string> = {
        critical: 'bg-red-600/10',
        high: 'bg-orange-500/10',
        unknown: 'bg-muted/50',
    };
    return colors[level] ?? 'bg-muted/50';
}

function zkillboardUrl(type: string, id: number): string {
    return `https://zkillboard.com/${type}/${id}/`;
}
</script>

<template>
    <MapPanel>
        <MapPanelHeader card-id="threat-analysis"> Wormhole Threat Analysis </MapPanelHeader>
        <MapPanelContent>
            <Deferred data="threat_analysis">
                <template #fallback>
                    <div class="space-y-3 p-3">
                        <div class="h-6 w-24 animate-pulse rounded bg-muted" />
                        <div class="h-4 w-full animate-pulse rounded bg-muted" />
                        <div class="h-4 w-3/4 animate-pulse rounded bg-muted" />
                    </div>
                </template>

                <div v-if="threat_analysis" class="space-y-3 p-3">
                    <div class="flex items-center justify-between">
                        <span
                            :class="[threatLevelColor(threat_analysis.threat_level), threatLevelBg(threat_analysis.threat_level)]"
                            class="rounded-md px-2 py-0.5 text-xs font-semibold"
                        >
                            {{ threatLevelLabel(threat_analysis.threat_level) }}
                        </span>
                        <a
                            :href="`https://zkillboard.com/system/${threat_analysis.solarsystem_id}/`"
                            target="_blank"
                            rel="noopener"
                            class="flex items-center gap-1 text-xs text-muted-foreground hover:text-foreground"
                        >
                            zKillboard
                            <ExternalLink class="size-3" />
                        </a>
                    </div>

                    <div v-if="threat_analysis.threat_data.length" class="space-y-1">
                        <div class="text-xs font-medium text-muted-foreground">Top Entities</div>
                        <div
                            v-for="entity in threat_analysis.threat_data"
                            :key="entity.id"
                            class="flex items-center gap-2 rounded px-2 py-1.5 text-xs hover:bg-muted/50"
                        >
                            <AllianceLogo
                                v-if="entity.type === 'alliance'"
                                :alliance_id="entity.id"
                                :alliance_name="entity.name"
                                :size="32"
                                class="size-5 shrink-0 rounded"
                            />
                            <CorporationLogo
                                v-else-if="entity.type === 'corporation'"
                                :corporation_id="entity.id"
                                :corporation_name="entity.name"
                                :size="32"
                                class="size-5 shrink-0 rounded"
                            />
                            <div class="min-w-0 flex-1">
                                <div class="truncate">{{ entity.name }}</div>
                                <div class="text-muted-foreground">{{ entity.kills }} kills</div>
                            </div>
                            <div v-if="entity.type !== 'unknown'" class="flex shrink-0 items-center gap-1">
                                <a
                                    :href="zkillboardUrl(entity.type, entity.id)"
                                    target="_blank"
                                    rel="noopener"
                                    class="rounded p-1 text-muted-foreground hover:bg-muted hover:text-foreground"
                                    title="zKillboard"
                                >
                                    <Globe class="size-3" />
                                </a>
                                <a
                                    :href="`${zkillboardUrl(entity.type, entity.id)}system/${threat_analysis.solarsystem_id}/`"
                                    target="_blank"
                                    rel="noopener"
                                    class="rounded p-1 text-muted-foreground hover:bg-muted hover:text-foreground"
                                    title="zKillboard in this system"
                                >
                                    <MapPin class="size-3" />
                                </a>
                            </div>
                        </div>
                    </div>

                    <div v-else class="text-xs text-muted-foreground">No significant activity detected.</div>

                    <div v-if="analyzedTimeAgo()" class="text-2xs text-muted-foreground">Analyzed {{ analyzedTimeAgo() }}</div>
                </div>

                <div v-else class="p-3 text-xs text-muted-foreground">No threat data available for this system.</div>
            </Deferred>
        </MapPanelContent>
    </MapPanel>
</template>
