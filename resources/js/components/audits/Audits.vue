<script setup lang="ts">
import Audit from '@/components/audits/Audit.vue';
import MapPanel from '@/components/ui/map-panel/MapPanel.vue';
import MapPanelContent from '@/components/ui/map-panel/MapPanelContent.vue';
import MapPanelHeader from '@/components/ui/map-panel/MapPanelHeader.vue';
import { TAudit } from '@/types/models';

const { audits = [] } = defineProps<{
    audits?: TAudit[];
}>();
</script>

<template>
    <MapPanel>
        <MapPanelHeader card-id="audits">
            Activity
            <span v-if="audits.length" class="ml-1 text-amber-400">{{ audits.length }}</span>
        </MapPanelHeader>
        <MapPanelContent>
            <template v-if="audits.length">
                <Audit v-for="audit in audits" :key="audit.id" :audit="audit" />
            </template>
            <div v-else class="flex h-full flex-col items-center justify-center gap-2 p-4">
                <p class="font-mono text-[10px] tracking-wider text-muted-foreground/60 uppercase">No activity</p>
            </div>
        </MapPanelContent>
    </MapPanel>
</template>

<style scoped></style>
