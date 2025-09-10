<script setup lang="ts">
import TrashIcon from '@/components/icons/TrashIcon.vue';
import MapConnectionSelection from '@/components/signatures/MapConnectionSelection.vue';
import SignatureID from '@/components/signatures/SignatureID.vue';
import SignatureTimeDetails from '@/components/signatures/SignatureTimeDetails.vue';
import SignatureTypeSelection from '@/components/signatures/SignatureTypeSelection.vue';
import WormholeTypeSelection from '@/components/signatures/WormholeTypeSelection.vue';
import { Button } from '@/components/ui/button';
import {
    DropdownMenu,
    DropdownMenuContent,
    DropdownMenuItem,
    DropdownMenuRadioGroup,
    DropdownMenuRadioItem,
    DropdownMenuSeparator,
    DropdownMenuTrigger,
} from '@/components/ui/dropdown-menu';
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from '@/components/ui/select';
import { deleteSignature, TProcessedConnection, updateSignature } from '@/composables/map';
import useHasWritePermission from '@/composables/useHasWritePermission';
import { TSignatureCategory } from '@/lib/SignatureParser';
import { formatDateToISO } from '@/lib/utils';
import { TMapSolarSystem, TSignature } from '@/types/models';
import { UTCDate } from '@date-fns/utc';
import { syncRefs } from '@vueuse/core';
import { MoreVertical } from 'lucide-vue-next';
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

function handleLifetimeChange(lifetime: string) {
    handleChange({
        lifetime: lifetime,
        lifetime_updated_at: formatDateToISO(new UTCDate()),
    });
}

function handleMassStatusChange(mass_status: string) {
    const value = mass_status === 'unknown' ? null : mass_status;
    handleChange({ mass_status: value });
}
</script>

<template>
    <div
        class="col-span-full grid grid-cols-subgrid items-center gap-x-1 px-2 py-2 hover:bg-muted/25 data-[deleted=true]:bg-red-500/10 data-[new=true]:bg-green-500/10 data-[updated=true]:bg-amber-500/15"
        :data-deleted="deleted"
        :data-new="$props.new"
        :data-updated="updated"
    >
        <div class="flex items-center gap-2">
            <div
                class="size-2 shrink-0 rounded-full bg-red-500 data-[incomplete=true]:bg-amber-500 data-[scanned=true]:bg-green-500"
                :data-scanned="Boolean(signature.signature_id && signature.category && signature.type)"
                :data-incomplete="!signature.signature_id || !signature.category || !signature.type"
            />
            <SignatureID
                v-model="signature_id"
                :disabled="!can_write"
                @submit="handleIDSubmit"
                :current-value="signature.signature_id"
                class="font-mono text-xs"
            />
        </div>

        <div>
            <Select :model-value="signature.category" @update:modelValue="handleCategoryChange" :disabled="!can_write">
                <SelectTrigger class="w-full text-xs">
                    <SelectValue placeholder="Category" />
                </SelectTrigger>
                <SelectContent>
                    <SelectItem v-for="option in options" :key="option" :value="option">
                        {{ option }}
                    </SelectItem>
                </SelectContent>
            </Select>
        </div>

        <div :class="signature.category !== 'Wormhole' ? 'col-span-2' : ''">
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
        </div>

        <div v-if="signature.category === 'Wormhole'">
            <MapConnectionSelection
                :selected="selected_connection"
                :unconnected_connections="unconnected_connections"
                :connected_connections="connected_connections"
                :model-value="signature.map_connection_id"
                @update:model-value="handleMapConnectionChange"
            />
        </div>

        <SignatureTimeDetails :category="signature.category" :selected_connection="selected_connection" :signature="signature" />

        <!-- Actions Dropdown -->
        <div class="text-right">
            <DropdownMenu v-if="can_write">
                <DropdownMenuTrigger as-child>
                    <Button variant="ghost" size="icon" class="text-muted-foreground">
                        <MoreVertical class="h-4 w-4" />
                    </Button>
                </DropdownMenuTrigger>
                <DropdownMenuContent align="end">
                    <template v-if="signature.category === 'Wormhole'">
                        <!-- Mass Status Options -->
                        <DropdownMenuRadioGroup :model-value="signature.mass_status || 'unknown'" @update:model-value="handleMassStatusChange">
                            <DropdownMenuRadioItem value="fresh">
                                <span class="flex items-center gap-2">
                                    <span class="inline-block size-2 rounded-full bg-neutral-500" />
                                    Fresh Mass
                                </span>
                            </DropdownMenuRadioItem>
                            <DropdownMenuRadioItem value="reduced">
                                <span class="flex items-center gap-2">
                                    <span class="inline-block size-2 rounded-full bg-amber-500" />
                                    Reduced Mass
                                </span>
                            </DropdownMenuRadioItem>
                            <DropdownMenuRadioItem value="critical">
                                <span class="flex items-center gap-2">
                                    <span class="inline-block size-2 rounded-full bg-red-500" />
                                    Critical Mass
                                </span>
                            </DropdownMenuRadioItem>
                        </DropdownMenuRadioGroup>

                        <DropdownMenuSeparator />

                        <!-- Lifetime Options -->
                        <DropdownMenuRadioGroup :model-value="signature.lifetime" @update:model-value="handleLifetimeChange">
                            <DropdownMenuRadioItem value="healthy" class="flex items-center justify-between gap-2">
                                <span class="flex items-center gap-2">
                                    <span class="inline-block size-2 rounded-full bg-neutral-500" />
                                    Healthy
                                </span>
                            </DropdownMenuRadioItem>
                            <DropdownMenuRadioItem value="eol" class="flex items-center justify-between gap-2">
                                <span class="flex items-center gap-2">
                                    <span class="inline-block size-2 rounded-full bg-purple-500" />
                                    End of Life
                                </span>
                                <span class="text-muted-foreground">&lt; 4h</span>
                            </DropdownMenuRadioItem>
                            <DropdownMenuRadioItem value="critical" class="flex items-center justify-between gap-2">
                                <span class="flex items-center gap-2">
                                    <span class="inline-block size-2 rounded-full bg-red-500" />
                                    Critical
                                </span>
                                <span class="text-muted-foreground">&lt; 1h</span>
                            </DropdownMenuRadioItem>
                        </DropdownMenuRadioGroup>

                        <DropdownMenuSeparator />
                    </template>

                    <!-- Delete Option -->
                    <DropdownMenuItem @click="handleDelete" class="text-destructive focus:text-destructive">
                        <span class="flex items-center gap-2">
                            <TrashIcon class="text-destructive" />
                            Delete Signature
                        </span>
                    </DropdownMenuItem>
                </DropdownMenuContent>
            </DropdownMenu>
        </div>
    </div>
</template>
