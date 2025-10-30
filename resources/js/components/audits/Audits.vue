<script setup lang="ts">
import Audit from '@/components/audits/Audit.vue';
import { CardDescription, CardHeader, CardTitle } from '@/components/ui/card';
import MapPanel from '@/components/ui/map-panel/MapPanel.vue';
import MapPanelContent from '@/components/ui/map-panel/MapPanelContent.vue';
import { TAudit } from '@/types/models';

const { audits = [] } = defineProps<{
    audits?: TAudit[];
}>();
</script>

<template>
    <MapPanel>
        <CardHeader>
            <CardTitle> Audits</CardTitle>
            <CardDescription> A list of the last changes made to the map solarsystem.</CardDescription>
        </CardHeader>
        <MapPanelContent inner-class="border-0 bg-transparent">
            <div class="relative max-h-60 overflow-x-hidden overflow-y-scroll mask-b-from-90% mask-alpha pr-1">
                <div class="rounded-lg border bg-white dark:bg-neutral-900/40">
                    <div class="grid grid-cols-[auto_1fr_auto_auto] gap-x-4">
                        <Audit v-for="audit in audits" :key="audit.id" :audit="audit" />
                        <div v-if="audits.length === 0" class="col-span-4 p-4 text-center text-sm text-muted-foreground">
                            No audits found for this solarsystem.
                        </div>
                    </div>
                </div>
            </div>
        </MapPanelContent>
    </MapPanel>
</template>

<style scoped></style>
