<script setup lang="ts">
import TrashIcon from '@/components/icons/TrashIcon.vue';
import MapConnectionInput from '@/components/signatures/MapConnectionInput.vue';
import SignatureIdInput from '@/components/signatures/SignatureIdInput.vue';
import SignatureTimeDetails from '@/components/signatures/SignatureTimeDetails.vue';
import SignatureTypeInput from '@/components/signatures/SignatureTypeInput.vue';
import WormholeTypeInput from '@/components/signatures/WormholeTypeInput.vue';
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
import { deleteSignature, getSolarsystemClass, toStringedSolarsystemClass, TProcessedConnection, updateSignature } from '@/composables/map';
import { useSolarsystemClass } from '@/composables/map/composables/useSolarsystemClass';
import { Data } from '@/composables/map/utils/data';
import useHasWritePermission from '@/composables/useHasWritePermission';
import { getTypesByCategory, signatureCategories } from '@/const/signatures';
import { formatDateToISO } from '@/lib/utils';
import { TMapSolarSystem, TSignature, TSolarsystemClass } from '@/types/models';
import { UTCDate } from '@date-fns/utc';
import { syncRefs } from '@vueuse/core';
import { MoreVertical } from 'lucide-vue-next';
import { AcceptableValue } from 'reka-ui';
import { computed, ref, toRef } from 'vue';

const { signature, unconnected_connections, connected_connections, selected_map_solarsystem } = defineProps<{
    signature: TSignature;
    is_deleted?: boolean;
    is_new?: boolean;
    is_updated?: boolean;
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

const solarsystem_class = useSolarsystemClass(selected_map_solarsystem);

// Get available types based on selected category and filter by spawn_areas for wormholes
const availableTypes = computed(() => {
    if (!signature.signature_category_id) return [];

    return getTypesByCategory(signature.signature_category_id).filter((type) =>
        type.spawn_areas?.includes(toStringedSolarsystemClass(solarsystem_class.value)!),
    );
});

// Helper function to get sort order for target class
const getClassSortOrder = (target_class: TSolarsystemClass | null): number => {
    if (!target_class) return 999;

    const stringed_target_class = toStringedSolarsystemClass(target_class)!;

    // Known space first, in order: H, L, N
    if (stringed_target_class === 'h') return 1;
    if (stringed_target_class === 'l') return 2;
    if (stringed_target_class === 'n') return 3;

    // Wormhole space (C1-C18)
    if (/^\d+$/.test(stringed_target_class)) {
        return 10 + parseInt(stringed_target_class);
    }

    // Special classes
    if (stringed_target_class === 'p') return 200; // Pochven
    if (stringed_target_class === 'unknown') return 300;

    return 999;
};

// Sort available types by target class
const sortedAvailableTypes = computed(() => {
    return availableTypes.value.toSorted((a, b) => getClassSortOrder(a.target_class) - getClassSortOrder(b.target_class));
});

// Get wormhole category ID for comparison
const wormholeCategoryId = computed(() => {
    return signatureCategories.find((cat) => cat.name === 'Wormhole')?.id;
});

const isWormhole = computed(() => {
    return signature.signature_category_id === wormholeCategoryId.value;
});

const current_class = computed(() => {
    if (!selected_connection.value?.target) return null;
    return getSolarsystemClass(selected_connection.value.target);
});

const is_scanned = computed(() => {
    if (!signature.signature_id) return false;
    if (!signature.signature_category_id) return false;
    return !(!signature.signature_type_id && !signature.raw_type_name);
});

const is_incomplete = computed(() => {
    return Boolean((signature.signature_id && signature.signature_category) || signature.signature_type_id || signature.raw_type_name);
});

function handleChange(data: Record<any, any>) {
    updateSignature(signature, data);
}

function handleDelete() {
    deleteSignature(signature);
}

function handleCategoryChange(value: AcceptableValue) {
    const signature_category_id = value as number;
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
        class="col-span-full grid grid-cols-subgrid items-center gap-x-1 px-2 py-2 hover:bg-muted/25 data-deleted:bg-red-500/10 data-new:bg-green-500/10 data-updated:bg-amber-500/15"
        :data-deleted="Data(is_deleted)"
        :data-new="Data(is_new)"
        :data-updated="Data(is_updated)"
    >
        <div class="flex items-center gap-2">
            <div
                class="size-2 shrink-0 rounded-full bg-red-500 data-incomplete:bg-amber-500 data-scanned:bg-green-500"
                :data-scanned="Data(is_scanned)"
                :data-incomplete="Data(is_incomplete)"
            />
            <SignatureIdInput
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

        <div :data-not-wormhole="Data(!isWormhole)" class="data-not-wormhole:col-span-2">
            <WormholeTypeInput
                v-if="isWormhole"
                :model-value="signature.signature_type_id"
                @update:model-value="handleTypeChange"
                :can_write="can_write"
                :wormhole_options="sortedAvailableTypes"
                :current_class="current_class"
            />
            <SignatureTypeInput
                v-else
                :model-value="signature.signature_type_id"
                @update:model-value="handleTypeChange"
                :can_write="can_write"
                :options="sortedAvailableTypes"
                :category="signature.signature_category?.name"
                :raw-type-name="signature.raw_type_name"
            />
        </div>

        <div v-if="isWormhole">
            <MapConnectionInput
                :type="signature.signature_type"
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
