<script setup lang="ts">
import CheckIcon from '@/components/icons/CheckIcon.vue';
import TimesIcon from '@/components/icons/TimesIcon.vue';
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
import { useForm } from '@inertiajs/vue3';
import { AcceptableValue } from 'reka-ui';
import { computed, watch } from 'vue';

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

const form = useForm(() => ({
    signature_id: signature.signature_id || '',
    category: signature.category,
    map_connection_id: signature.map_connection_id,
    type: signature.type,
}));

const signatures_options = computed(() => {
    if (!form.category) {
        return [];
    }
    return possible_signatures[form.category] || [];
});

const can_write = useHasWritePermission();

const selected_connection = computed(() => {
    return (
        unconnected_connections.find((c) => c.id === form.map_connection_id) ??
        connected_connections.find((c) => c.id === form.map_connection_id) ??
        null
    );
});

const options = ['Wormhole', 'Gas Site', 'Ore Site', 'Combat Site', 'Data Site', 'Relic Site', 'Unknown'];

watch(
    () => signature.signature_id,
    () => {
        form.signature_id = signature.signature_id || '';
    },
);
watch(
    () => signature.category,
    () => {
        form.category = signature.category;
    },
);
watch(
    () => signature.type,
    () => {
        form.type = signature.type;
    },
);

watch(
    () => signature.map_connection_id,
    () => {
        form.map_connection_id = signature.map_connection_id;
    },
);

function handleSubmit() {
    form.put(route('signatures.update', signature.id), {
        preserveScroll: true,
        preserveState: true,
        only: ['selected_map_solarsystem'],
        onSuccess: () => {
            form.reset();
        },
    });
}

function handleDelete() {
    form.delete(route('signatures.destroy', signature.id), {
        preserveScroll: true,
        preserveState: true,
        only: ['selected_map_solarsystem', 'map'],
    });
}

function handleCategoryChange(value: AcceptableValue) {
    form.type = form.category !== value ? null : form.type;
    form.category = value as TSignatureCategory;
    form.map_connection_id = null;
}

function handleReset() {
    form.reset();
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
            <SignatureID v-model="form.signature_id" :disabled="!can_write" />
        </div>
        <Select :model-value="form.category" @update:modelValue="handleCategoryChange" :disabled="!can_write">
            <SelectTrigger class="w-full">
                <SelectValue placeholder="Select category" />
            </SelectTrigger>
            <SelectContent>
                <SelectItem v-for="option in options" :key="option" :value="option">
                    {{ option }}
                </SelectItem>
            </SelectContent>
        </Select>
        <Select v-model:model-value="form.type" :disabled="!can_write">
            <SelectTrigger class="w-full overflow-hidden data-[wormhole=false]:col-span-2" :data-wormhole="form.category === 'Wormhole'">
                <SelectValue as-child>
                    <span class="truncate">{{ form.type || 'Select type' }}</span>
                </SelectValue>
            </SelectTrigger>
            <SelectContent>
                <SelectItem v-for="option in signatures_options" :key="option" :value="option">
                    {{ option }}
                </SelectItem>
            </SelectContent>
        </Select>
        <MapConnectionSelection
            v-if="form.category === 'Wormhole'"
            :selected="selected_connection"
            :unconnected_connections="unconnected_connections"
            :connected_connections="connected_connections"
            v-model="form.map_connection_id"
        />
        <SignatureTimeDetails :category="form.category" :selected_connection="selected_connection" :signature="signature" />
        <div class="flex gap-2" v-if="can_write">
            <template v-if="form.isDirty">
                <Tooltip>
                    <TooltipTrigger as-child>
                        <Button variant="secondary" size="icon" @click="handleReset">
                            <TimesIcon />
                        </Button>
                    </TooltipTrigger>
                    <TooltipContent> Reset Changes</TooltipContent>
                </Tooltip>
                <Tooltip>
                    <TooltipTrigger as-child>
                        <Button variant="secondary" @click="handleSubmit">
                            <CheckIcon />
                        </Button>
                    </TooltipTrigger>
                    <TooltipContent> Save Signature</TooltipContent>
                </Tooltip>
            </template>
            <Tooltip v-else>
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
