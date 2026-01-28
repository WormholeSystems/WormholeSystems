<script setup lang="ts">
import PasteIcon from '@/components/icons/PasteIcon.vue';
import PlusIcon from '@/components/icons/PlusIcon.vue';
import TrashIcon from '@/components/icons/TrashIcon.vue';
import PasteSignatureWarningDialog from '@/components/signatures/PasteSignatureWarningDialog.vue';
import Signature from '@/components/signatures/Signature.vue';
import SignaturesEmptyState from '@/components/signatures/SignaturesEmptyState.vue';
import MapPanel from '@/components/ui/map-panel/MapPanel.vue';
import MapPanelContent from '@/components/ui/map-panel/MapPanelContent.vue';
import MapPanelHeader from '@/components/ui/map-panel/MapPanelHeader.vue';
import MapPanelHeaderActionButton from '@/components/ui/map-panel/MapPanelHeaderActionButton.vue';
import { Tooltip, TooltipContent, TooltipTrigger } from '@/components/ui/tooltip';
import { createSignature, useSignatures } from '@/composables/map';
import { usePasteSignatures } from '@/composables/map/composables/usePasteSignatures';
import { useSortableSignatures } from '@/composables/map/composables/useSortedSignatures';
import { useActiveMapCharacter } from '@/composables/useActiveMapCharacter';
import useHasWritePermission from '@/composables/useHasWritePermission';
import type { TResolvedSelectedMapSolarsystem } from '@/pages/maps';
import { ArrowDown, ArrowUp } from 'lucide-vue-next';
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

function handleSort(column: 'id' | 'category' | 'type' | 'age') {
    let newDirection: 'asc' | 'desc';

    if (sortPreferences.value.column === column) {
        newDirection = sortPreferences.value.direction === 'asc' ? 'desc' : 'asc';
    } else {
        newDirection = 'asc';
    }

    updateSortPreferences(column, newDirection);
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
        <MapPanelHeader>
            Signatures
            <span v-if="signatures.length" class="ml-1 text-amber-400">{{ signatures.length }}</span>
            <template #actions v-if="can_write">
                <Tooltip v-if="pasted_signatures">
                    <TooltipTrigger as-child>
                        <MapPanelHeaderActionButton v-if="pasted_signatures" @click="pasted_signatures = null"> Unselect </MapPanelHeaderActionButton>
                    </TooltipTrigger>
                    <TooltipContent> Unselect signatures</TooltipContent>
                </Tooltip>
                <Tooltip v-if="deleted_signatures.length > 0">
                    <TooltipTrigger as-child>
                        <MapPanelHeaderActionButton @click="deleteMissingSignatures(true)" variant="destructive" size="icon">
                            <TrashIcon />
                        </MapPanelHeaderActionButton>
                    </TooltipTrigger>
                    <TooltipContent> Delete missing signatures and their connections </TooltipContent>
                </Tooltip>
                <Tooltip>
                    <TooltipTrigger as-child>
                        <MapPanelHeaderActionButton @click="pasteSignatures" size="icon">
                            <PasteIcon />
                        </MapPanelHeaderActionButton>
                    </TooltipTrigger>
                    <TooltipContent> Paste signatures from clipboard (Ctrl/Cmd + V)</TooltipContent>
                </Tooltip>
                <Tooltip>
                    <TooltipTrigger as-child>
                        <MapPanelHeaderActionButton @click="createNewSignature" size="icon">
                            <PlusIcon />
                        </MapPanelHeaderActionButton>
                    </TooltipTrigger>
                    <TooltipContent> Create new signature</TooltipContent>
                </Tooltip>
            </template>
        </MapPanelHeader>
        <MapPanelContent>
            <!-- Header -->
            <div
                class="flex items-center gap-2 border-b border-border/30 bg-muted/20 px-3 py-1.5 font-mono text-[10px] tracking-wider text-muted-foreground uppercase"
            >
                <button class="flex w-16 shrink-0 items-center gap-1 hover:text-foreground" @click="handleSort('id')">
                    <span>ID</span>
                    <ArrowUp v-if="sortPreferences.column === 'id' && sortPreferences.direction === 'asc'" class="size-3" />
                    <ArrowDown v-if="sortPreferences.column === 'id' && sortPreferences.direction === 'desc'" class="size-3" />
                </button>
                <button class="flex w-24 shrink-0 items-center gap-1 hover:text-foreground" @click="handleSort('category')">
                    <span>Cat</span>
                    <ArrowUp v-if="sortPreferences.column === 'category' && sortPreferences.direction === 'asc'" class="size-3" />
                    <ArrowDown v-if="sortPreferences.column === 'category' && sortPreferences.direction === 'desc'" class="size-3" />
                </button>
                <button class="flex min-w-0 flex-1 items-center gap-1 hover:text-foreground" @click="handleSort('type')">
                    <span>Type</span>
                    <ArrowUp v-if="sortPreferences.column === 'type' && sortPreferences.direction === 'asc'" class="size-3" />
                    <ArrowDown v-if="sortPreferences.column === 'type' && sortPreferences.direction === 'desc'" class="size-3" />
                </button>
                <span class="min-w-0 flex-1">Conn</span>
                <button class="flex w-10 shrink-0 items-center justify-end gap-1 hover:text-foreground" @click="handleSort('age')">
                    <span>Age</span>
                    <ArrowUp v-if="sortPreferences.column === 'age' && sortPreferences.direction === 'asc'" class="size-3" />
                    <ArrowDown v-if="sortPreferences.column === 'age' && sortPreferences.direction === 'desc'" class="size-3" />
                </button>
                <span class="w-6 shrink-0"></span>
            </div>

            <!-- Signature rows -->
            <template v-if="sorted.length">
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
            </template>
            <div v-else class="flex h-full flex-col items-center justify-center gap-2 p-4">
                <p class="font-mono text-[10px] tracking-wider text-muted-foreground/60 uppercase">No signatures</p>
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
