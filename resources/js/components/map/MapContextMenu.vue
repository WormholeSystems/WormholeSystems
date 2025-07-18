<script setup lang="ts">
import { Button } from '@/components/ui/button';
import { ContextMenuContent, ContextMenuItem, ContextMenuSeparator } from '@/components/ui/context-menu';
import { Dialog, DialogContent, DialogDescription, DialogFooter, DialogHeader, DialogTitle } from '@/components/ui/dialog';
import { useMapAction, useMapSolarsystems } from '@/composables/map';
import { ref } from 'vue';

const { removeAllMapSolarsystems, sortMapSolarsystemsByRegion, organizeMapSolarsystems, removeSelectedMapSolarsystems } = useMapAction();

const { map_solarsystems_selected } = useMapSolarsystems();

const is_clearing_map = ref(false);

function handleDelete() {
    is_clearing_map.value = true;
}

function handleConfirmDelete() {
    removeAllMapSolarsystems();
    is_clearing_map.value = false;
}

function handeCancelDelete() {
    is_clearing_map.value = false;
}
</script>

<template>
    <ContextMenuContent>
        <template v-if="map_solarsystems_selected.length">
            <ContextMenuItem @select="removeSelectedMapSolarsystems"> Remove solarsystems</ContextMenuItem>
            <ContextMenuItem @select="sortMapSolarsystemsByRegion"> Sort by region</ContextMenuItem>
            <ContextMenuItem @select="organizeMapSolarsystems"> Organize solarsystems</ContextMenuItem>
            <ContextMenuSeparator />
        </template>
        <ContextMenuItem @select="handleDelete"> Clear map</ContextMenuItem>
    </ContextMenuContent>
    <Dialog v-model:open="is_clearing_map">
        <DialogContent>
            <DialogHeader>
                <DialogTitle> Clear map</DialogTitle>
                <DialogDescription>
                    Are you sure you want to clear the map? This will remove all solarsystems that are not pinned.
                </DialogDescription>
            </DialogHeader>
            <DialogFooter>
                <Button variant="outline" @click="handeCancelDelete"> Cancel</Button>
                <Button variant="destructive" @click="handleConfirmDelete"> Clear map</Button>
            </DialogFooter>
        </DialogContent>
    </Dialog>
</template>

<style scoped></style>
