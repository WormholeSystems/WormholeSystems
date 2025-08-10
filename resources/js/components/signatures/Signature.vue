<script setup lang="ts">
import TrashIcon from '@/components/icons/TrashIcon.vue';
import MapConnectionSelection from '@/components/signatures/MapConnectionSelection.vue';
import SignatureID from '@/components/signatures/SignatureID.vue';
import SignatureTimeDetails from '@/components/signatures/SignatureTimeDetails.vue';
import SignatureTypeSelection from '@/components/signatures/SignatureTypeSelection.vue';
import WormholeTypeSelection from '@/components/signatures/WormholeTypeSelection.vue';
import { Button } from '@/components/ui/button';
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from '@/components/ui/select';
import { Tooltip, TooltipContent, TooltipTrigger } from '@/components/ui/tooltip';
import { deleteSignature, TProcessedConnection, updateSignature } from '@/composables/map';
import { useHasWritePermission } from '@/composables/useHasPermission';
import { TSignatureCategory } from '@/lib/SignatureParser';
import { TMapSolarSystem, TSignature } from '@/types/models';
import { syncRefs } from '@vueuse/core';
import { AcceptableValue } from 'reka-ui';
import { computed, ref, toRef } from 'vue';

const { signature, unconnected_connections, possible_signatures, connected_connections } = defineProps<{
    signature: TSignature;
    deleted?: boolean;
    new?: boolean;
    updated?: boolean;
    unconnected_connections: TProcessedConnection[];
    connected_connections: TProcessedConnection[];
    selected_map_solarsystem: TMapSolarSystem;
    possible_signatures: any;
}>();

const original = toRef(() => signature.signature_id || '');
const signature_id = ref('');

syncRefs(original, signature_id);

const can_write = useHasWritePermission();

const selected_connection = computed(() => {
    return (
        unconnected_connections.find((c) => c.id === signature.map_connection_id) ??
        connected_connections.find((c) => c.id === signature.map_connection_id) ??
        null
    );
});

const options = ['Wormhole', 'Gas Site', 'Ore Site', 'Combat Site', 'Data Site', 'Relic Site', 'Unknown'];

function handleChange(data: Record<any, any>) {
    updateSignature(signature, data);
}

function handleDelete() {
    deleteSignature(signature);
}

function handleCategoryChange(value: AcceptableValue) {
    const type = signature.category !== value ? null : signature.type;
    const category = value as TSignatureCategory;
    const map_connection_id = null;
    handleChange({
        category,
        type,
        map_connection_id,
    });
}

function handleTypeChange(value: AcceptableValue) {
    const type = value as string;
    handleChange({
        type,
    });
}

function handleMapConnectionChange(value: AcceptableValue) {
    const map_connection_id = value as number | null;
    handleChange({
        map_connection_id,
    });
}

function handleIDSubmit() {
    handleChange({
        signature_id: signature_id.value.toString().trim() || null,
    });
}
</script>

<template>
    <div
        class="col-span-full grid grid-cols-subgrid items-center py-2 *:first:pl-2 *:last:pr-2 data-[deleted=true]:bg-red-500/10 data-[new=true]:bg-green-500/10 data-[updated=true]:bg-amber-500/15"
        :data-deleted="deleted"
        :data-new="$props.new"
        :data-updated="updated"
    >
        <div class="flex items-center gap-2">
            <div
                class="inline-block size-2 rounded-full data-[scanned=false]:bg-red-500 data-[scanned=true]:bg-green-500"
                :data-scanned="Boolean(signature.signature_id && signature.category)"
            />
            <SignatureID v-model="signature_id" :disabled="!can_write" @submit="handleIDSubmit" :current-value="signature.signature_id" />
        </div>
        <Select :model-value="signature.category" @update:modelValue="handleCategoryChange" :disabled="!can_write">
            <SelectTrigger class="w-full overflow-hidden">
                <SelectValue placeholder="Category" />
            </SelectTrigger>
            <SelectContent>
                <SelectItem v-for="option in options" :key="option" :value="option">
                    {{ option }}
                </SelectItem>
            </SelectContent>
        </Select>
        <WormholeTypeSelection
            :model-value="signature.type"
            @update:model-value="handleTypeChange"
            :can_write="can_write"
            :possible_signatures="possible_signatures"
            v-if="signature.category === 'Wormhole'"
        />
        <SignatureTypeSelection
            v-else
            :model-value="signature.type"
            @update:model-value="handleTypeChange"
            :can_write="can_write"
            :options="signature.category ? possible_signatures[signature.category] : []"
            :category="signature.category"
        />
        <MapConnectionSelection
            v-if="signature.category === 'Wormhole'"
            :selected="selected_connection"
            :unconnected_connections="unconnected_connections"
            :connected_connections="connected_connections"
            :model-value="signature.map_connection_id"
            @update:model-value="handleMapConnectionChange"
        />
        <SignatureTimeDetails :category="signature.category" :selected_connection="selected_connection" :signature="signature" />
        <div class="flex gap-2" v-if="can_write">
            <Tooltip>
                <TooltipTrigger as-child>
                    <Button variant="secondary" @click="handleDelete">
                        <TrashIcon />
                    </Button>
                </TooltipTrigger>
                <TooltipContent> Delete Signature</TooltipContent>
            </Tooltip>
        </div>
    </div>
</template>

<style scoped></style>
