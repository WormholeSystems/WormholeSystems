<script setup lang="ts">
import { CardDescription, CardHeader, CardTitle } from '@/components/ui/card';
import MapPanel from '@/components/ui/map-panel/MapPanel.vue';
import MapPanelContent from '@/components/ui/map-panel/MapPanelContent.vue';
</script>

<template>
    <MapPanel class="overflow-hidden">
        <CardHeader>
            <CardTitle>Signatures</CardTitle>
            <CardDescription> All the signatures in this solarsystem. You can paste, copy and clear signatures here. </CardDescription>
        </CardHeader>
        <MapPanelContent>
            <div class="flex h-full min-h-32 items-center justify-center rounded-lg border">
                <p class="text-center text-sm text-muted-foreground">Select a solarsystem to view signatures</p>
            </div>
        </MapPanelContent>
    </MapPanel>
</template>
