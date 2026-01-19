<script setup lang="ts">
import PasteIcon from '@/components/icons/PasteIcon.vue';
import PlusIcon from '@/components/icons/PlusIcon.vue';
import TrashIcon from '@/components/icons/TrashIcon.vue';
import PasteSignatureWarningDialog from '@/components/signatures/PasteSignatureWarningDialog.vue';
import Signature from '@/components/signatures/Signature.vue';
import SignaturesEmptyState from '@/components/signatures/SignaturesEmptyState.vue';
import SortHeader from '@/components/signatures/SortHeader.vue';
import { Button } from '@/components/ui/button';
import { CardAction, CardDescription, CardHeader, CardTitle } from '@/components/ui/card';
import MapPanel from '@/components/ui/map-panel/MapPanel.vue';
import MapPanelContent from '@/components/ui/map-panel/MapPanelContent.vue';
import { Tooltip, TooltipContent, TooltipTrigger } from '@/components/ui/tooltip';
import { createSignature, useSignatures } from '@/composables/map';
import { usePasteSignatures } from '@/composables/map/composables/usePasteSignatures';
import { useSortableSignatures } from '@/composables/map/composables/useSortedSignatures';
import { useActiveMapCharacter } from '@/composables/useActiveMapCharacter';
import useHasWritePermission from '@/composables/useHasWritePermission';
import type { TResolvedSelectedMapSolarsystem } from '@/pages/maps';
import { computed } from 'vue';

const props = defineProps<{
    map_solarsystem: TResolvedSelectedMapSolarsystem | null;
}>();

const { connections } = useSignatures();

const can_write = useHasWritePermission();

const character = useActiveMapCharacter();

const {
    signatures,
    pasted_signatures,
    deleted_signatures,
    deleteMissingSignatures,
    pasteSignatures,
    show_system_mismatch_warning,
    confirmPasteInDifferentSystem,
    cancelPaste,
} = usePasteSignatures(() => props.map_solarsystem);

const { sortPreferences, sorted, updateSortPreferences } = useSortableSignatures(signatures);

const connected_connections = computed(() => {
    return connections.value.filter((connection) => {
        return signatures.value.some((signature) => {
            return signature.map_connection_id === connection.id;
        });
    });
});

const unconnected_connections = computed(() => {
    return connections.value.filter((connection) => {
        return !signatures.value.some((signature) => {
            return signature.map_connection_id === connection.id;
        });
    });
});

function handleSort(column: string) {
    const sortColumn = column as 'id' | 'category' | 'type' | 'age';
    let newDirection: 'asc' | 'desc';

    if (sortPreferences.value.column === sortColumn) {
        newDirection = sortPreferences.value.direction === 'asc' ? 'desc' : 'asc';
    } else {
        newDirection = 'asc';
    }

    updateSortPreferences(sortColumn, newDirection);
}

function createNewSignature() {
    if (!props.map_solarsystem) return;
    createSignature(props.map_solarsystem.id);
}
</script>

<template>
    <!-- Empty state when no system selected -->
    <SignaturesEmptyState v-if="!map_solarsystem" />

    <!-- Signatures list when system is selected -->
    <MapPanel v-if="map_solarsystem" class="overflow-x-hidden">
        <CardHeader>
            <CardTitle
                >Signatures
                <span class="text-muted-foreground" v-if="signatures.length">({{ signatures.length }})</span>
            </CardTitle>
            <CardDescription> All the signatures in this solarsystem. You can paste, copy and clear signatures here. </CardDescription>

            <CardAction class="flex gap-2" v-if="can_write">
                <Tooltip v-if="pasted_signatures">
                    <TooltipTrigger as-child>
                        <Button v-if="pasted_signatures" @click="pasted_signatures = null" variant="secondary"> Unselect </Button>
                    </TooltipTrigger>
                    <TooltipContent> Unselect signatures</TooltipContent>
                </Tooltip>
                <Tooltip v-if="deleted_signatures.length > 0">
                    <TooltipTrigger as-child>
                        <Button @click="deleteMissingSignatures(true)" variant="destructive" size="icon"> <TrashIcon /> </Button>
                    </TooltipTrigger>
                    <TooltipContent> Delete missing signatures and their connections </TooltipContent>
                </Tooltip>
                <Tooltip>
                    <TooltipTrigger as-child>
                        <Button @click="pasteSignatures" variant="secondary" size="icon">
                            <PasteIcon />
                        </Button>
                    </TooltipTrigger>
                    <TooltipContent> Paste signatures from clipboard (Ctrl/Cmd + V)</TooltipContent>
                </Tooltip>
                <Tooltip>
                    <TooltipTrigger as-child>
                        <Button @click="createNewSignature" variant="secondary" size="icon">
                            <PlusIcon />
                        </Button>
                    </TooltipTrigger>
                    <TooltipContent> Create new signature</TooltipContent>
                </Tooltip>
            </CardAction>
        </CardHeader>
        <MapPanelContent>
            <div class="overflow-hidden rounded-lg border">
                <div class="grid grid-cols-[auto_1fr_1fr_1fr_auto_auto] gap-x-2 divide-y">
                    <div class="col-span-full grid grid-cols-subgrid border-b bg-muted/50 px-2 py-1.5 text-xs font-medium text-muted-foreground">
                        <SortHeader
                            label="ID"
                            column="id"
                            :is-current-column="sortPreferences.column === 'id'"
                            :current-direction="sortPreferences.direction"
                            @sort="handleSort"
                        />
                        <SortHeader
                            label="Category"
                            column="category"
                            :is-current-column="sortPreferences.column === 'category'"
                            :current-direction="sortPreferences.direction"
                            @sort="handleSort"
                        />
                        <SortHeader
                            label="Type"
                            column="type"
                            :is-current-column="sortPreferences.column === 'type'"
                            :current-direction="sortPreferences.direction"
                            @sort="handleSort"
                        />
                        <div class="text-left">Connection</div>
                        <SortHeader
                            label="Age"
                            column="age"
                            :is-current-column="sortPreferences.column === 'age'"
                            :current-direction="sortPreferences.direction"
                            @sort="handleSort"
                        />
                        <div></div>
                    </div>
                    <Signature
                        v-for="signature in sorted"
                        :signature="signature"
                        :key="signature.id"
                        :is_deleted="signature.deleted"
                        :is_new="signature.new"
                        :is_updated="signature.updated"
                        :unconnected_connections="unconnected_connections"
                        :connected_connections="connected_connections"
                        :selected_map_solarsystem="map_solarsystem"
                    />
                </div>
                <div v-if="!signatures.length" class="p-4 text-center text-sm text-muted-foreground">No signatures found</div>
            </div>
        </MapPanelContent>
    </MapPanel>

    <!-- Warning dialog for pasting in different system -->
    <PasteSignatureWarningDialog
        v-model:open="show_system_mismatch_warning"
        :target-system="map_solarsystem"
        :character="character"
        @confirm="confirmPasteInDifferentSystem"
        @cancel="cancelPaste"
    />
</template>

<style scoped>
tr:has([data-status='new']) {
    background-color: var(--color-red-100);
}
</style>
