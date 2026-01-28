<script setup lang="ts">
import MapPanel from '@/components/ui/map-panel/MapPanel.vue';
import MapPanelContent from '@/components/ui/map-panel/MapPanelContent.vue';
import MapPanelHeader from '@/components/ui/map-panel/MapPanelHeader.vue';
</script>

<template>
    <MapPanel>
        <MapPanelHeader> System </MapPanelHeader>
        <MapPanelContent>
            <div class="flex h-full flex-col items-center justify-center gap-2 p-4">
                <p class="font-mono text-[10px] tracking-wider text-muted-foreground/60 uppercase">Select a system</p>
            </div>
        </MapPanelContent>
    </MapPanel>
</template>
