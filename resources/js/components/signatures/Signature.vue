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
import { signatureCategories, getTypesByCategory } from '@/const/signatures';
import { formatDateToISO } from '@/lib/utils';
import { TMapSolarSystem, TSignature } from '@/types/models';
import { UTCDate } from '@date-fns/utc';
import { syncRefs } from '@vueuse/core';
import { MoreVertical } from 'lucide-vue-next';
import { AcceptableValue } from 'reka-ui';
import { computed, ref, toRef } from 'vue';

const { signature, unconnected_connections, connected_connections, selected_map_solarsystem } = defineProps<{
    signature: TSignature;
    deleted?: boolean;
    new?: boolean;
    updated?: boolean;
    unconnected_connections: TProcessedConnection[];
    connected_connections: TProcessedConnection[];
    selected_map_solarsystem: TMapSolarSystem;
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

// Get current solarsystem class for filtering
const currentSystemClass = computed(() => {
    const sysClass = selected_map_solarsystem.class;
    if (sysClass >= 1 && sysClass <= 18) return String(sysClass);
    if (sysClass === 25) return 'pv'; // Pochven
    
    // Known space
    const security = selected_map_solarsystem.solarsystem?.security;
    if (security === undefined) return null;
    if (security >= 0.5) return 'hs';
    if (security > 0.0) return 'ls';
    return 'ns';
});

// Get available types based on selected category and filter by spawn_areas for wormholes
const availableTypes = computed(() => {
    if (!signature.signature_category_id) return [];
    
    const types = getTypesByCategory(signature.signature_category_id);
    
    // Filter wormhole types by current system's spawn area
    if (isWormhole.value && currentSystemClass.value) {
        return types.filter(type => {
            if (!type.spawn_areas || type.spawn_areas.length === 0) return true;
            return type.spawn_areas.includes(currentSystemClass.value!);
        });
    }
    
    return types;
});

// Helper function to get sort order for target class
const getClassSortOrder = (targetClass: string | null): number => {
    if (!targetClass) return 999;
    
    // Known space first, in order: H, L, N
    if (targetClass === 'hs') return 1;
    if (targetClass === 'ls') return 2;
    if (targetClass === 'ns') return 3;
    
    // Wormhole space (C1-C18)
    if (/^\d+$/.test(targetClass)) {
        return 10 + parseInt(targetClass);
    }
    
    // Special classes
    if (targetClass === 'pv') return 200; // Pochven
    if (targetClass === 'unknown') return 300;
    
    return 999;
};

// Sort available types by target class
const sortedAvailableTypes = computed(() => {
    return availableTypes.value.sort((a, b) => getClassSortOrder(a.target_class) - getClassSortOrder(b.target_class));
});

// Get wormhole category ID for comparison
const wormholeCategoryId = computed(() => {
    return signatureCategories.find(cat => cat.name === 'Wormhole')?.id;
});

const isWormhole = computed(() => {
    return signature.signature_category_id === wormholeCategoryId.value;
});

function handleChange(data: Record<any, any>) {
    updateSignature(signature, data);
}

function handleDelete() {
    deleteSignature(signature);
}

function handleCategoryChange(value: AcceptableValue) {
    const categoryId = value as number;
    const signature_category_id = categoryId;
    const signature_type_id = null; // Clear type when category changes
    const map_connection_id = null;
    handleChange({
        signature_category_id,
        signature_type_id,
        map_connection_id,
    });
}

function handleTypeChange(value: AcceptableValue) {
    const signature_type_id = value as number;
    handleChange({
        signature_type_id,
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
                :data-scanned="signature.signature_id && signature.signature_category_id && signature.signature_type_id ? true : false"
                :data-incomplete="signature_id && (signature.signature_category_id || signature.signature_type_id) ? true : false"
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
            <Select :model-value="signature.signature_category_id" @update:modelValue="handleCategoryChange" :disabled="!can_write">
                <SelectTrigger class="w-full text-xs">
                    <SelectValue placeholder="Category" />
                </SelectTrigger>
                <SelectContent>
                    <SelectItem v-for="category in signatureCategories" :key="category.id" :value="category.id">
                        {{ category.name }}
                    </SelectItem>
                </SelectContent>
            </Select>
        </div>

        <div :class="!isWormhole ? 'col-span-2' : ''">
            <WormholeTypeSelection
                v-if="isWormhole"
                v-model="signature.signature_type_id"
                @update:model-value="handleTypeChange"
                :can_write="can_write"
                :wormhole_options="sortedAvailableTypes"
            />
            <SignatureTypeSelection
                v-else
                v-model="signature.signature_type_id"
                @update:model-value="handleTypeChange"
                :can_write="can_write"
                :options="sortedAvailableTypes"
                :category="signature.signature_category?.name"
            />
        </div>

        <div v-if="isWormhole">
            <MapConnectionSelection
                :selected="selected_connection"
                :unconnected_connections="unconnected_connections"
                :connected_connections="connected_connections"
                :model-value="signature.map_connection_id"
                @update:model-value="handleMapConnectionChange"
            />
        </div>

        <SignatureTimeDetails :category="signature.signature_category?.name" :selected_connection="selected_connection" :signature="signature" />

        <!-- Actions Dropdown -->
        <div class="text-right">
            <DropdownMenu v-if="can_write">
                <DropdownMenuTrigger as-child>
                    <Button variant="ghost" size="icon" class="text-muted-foreground">
                        <MoreVertical class="h-4 w-4" />
                    </Button>
                </DropdownMenuTrigger>
                <DropdownMenuContent align="end">
                    <template v-if="signature.signature_category?.name === 'Wormhole'">
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
