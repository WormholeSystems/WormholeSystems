<script setup lang="ts">
import TrashIcon from '@/components/icons/TrashIcon.vue';
import MapConnectionSelection from '@/components/signatures/MapConnectionSelection.vue';
import SignatureID from '@/components/signatures/SignatureID.vue';
import SignatureTimeDetails from '@/components/signatures/SignatureTimeDetails.vue';
import { Button } from '@/components/ui/button';
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from '@/components/ui/select';
import { Tooltip, TooltipContent, TooltipTrigger } from '@/components/ui/tooltip';
import { TProcessedConnection } from '@/composables/map';
import { useHasWritePermission } from '@/composables/useHasPermission';
import { TSignatureCategory } from '@/lib/SignatureParser';
import { TMapSolarSystem, TSignature } from '@/types/models';
import { router } from '@inertiajs/vue3';
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

const signatures_options = computed(() => {
    if (!signature.category) {
        return [];
    }
    return possible_signatures[signature.category] || [];
});
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
    router.put(route('signatures.update', signature.id), data, {
        preserveScroll: true,
        preserveState: true,
        only: ['selected_map_solarsystem'],
    });
}

function handleDelete() {
    router.delete(route('signatures.destroy', signature.id), {
        preserveScroll: true,
        preserveState: true,
        only: ['selected_map_solarsystem', 'map'],
    });
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
                :data-scanned="Boolean(signature.signature_id && signature.type)"
            />
            <SignatureID v-model="signature_id" :disabled="!can_write" @submit="handleIDSubmit" :current-value="signature.signature_id" />
        </div>
        <Select :model-value="signature.category" @update:modelValue="handleCategoryChange" :disabled="!can_write">
            <SelectTrigger class="w-full">
                <SelectValue placeholder="Select category" />
            </SelectTrigger>
            <SelectContent>
                <SelectItem v-for="option in options" :key="option" :value="option">
                    {{ option }}
                </SelectItem>
            </SelectContent>
        </Select>
        <Select :model-value="signature.type" @update:model-value="handleTypeChange" :disabled="!can_write">
            <SelectTrigger class="w-full overflow-hidden data-[wormhole=false]:col-span-2" :data-wormhole="signature.category === 'Wormhole'">
                <SelectValue as-child>
                    <span class="truncate">{{ signature.type || 'Select type' }}</span>
                </SelectValue>
            </SelectTrigger>
            <SelectContent>
                <SelectItem v-for="option in signatures_options" :key="option" :value="option">
                    {{ option }}
                </SelectItem>
            </SelectContent>
        </Select>
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
