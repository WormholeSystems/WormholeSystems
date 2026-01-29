<script setup lang="ts">
import TrashIcon from '@/components/icons/TrashIcon.vue';
import MapConnectionInput from '@/components/signatures/MapConnectionInput.vue';
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
import type { TResolvedSelectedMapSolarsystem } from '@/pages/maps';
import { TSignature, TSolarsystemClass } from '@/types/models';
import { UTCDate } from '@date-fns/utc';
import { syncRefs } from '@vueuse/core';
import { MoreVertical } from 'lucide-vue-next';
import { AcceptableValue } from 'reka-ui';
import { computed, nextTick, ref, toRef } from 'vue';

const { signature, unconnected_connections, connected_connections, selected_map_solarsystem } = defineProps<{
    signature: TSignature;
    is_deleted?: boolean;
    is_new?: boolean;
    is_updated?: boolean;
    unconnected_connections: TProcessedConnection[];
    connected_connections: TProcessedConnection[];
    selected_map_solarsystem: TResolvedSelectedMapSolarsystem;
}>();

const original = toRef(() => signature.signature_id || '');
const signature_id = ref('');
syncRefs(original, signature_id);

const can_write = useHasWritePermission();

// Inline editing state
const editingId = ref(false);
const idInputRef = ref<HTMLInputElement | null>(null);

const selected_connection = computed(() => {
    return (
        unconnected_connections.find((c) => c.id === signature.map_connection_id) ??
        connected_connections.find((c) => c.id === signature.map_connection_id) ??
        null
    );
});

const solarsystem_class = useSolarsystemClass(selected_map_solarsystem);

const availableTypes = computed(() => {
    if (!signature.signature_category_id) return [];
    return getTypesByCategory(signature.signature_category_id).filter((type) =>
        type.spawn_areas?.includes(toStringedSolarsystemClass(solarsystem_class.value)!),
    );
});

const getClassSortOrder = (target_class: TSolarsystemClass | null): number => {
    if (!target_class) return 999;
    const stringed_target_class = toStringedSolarsystemClass(target_class)!;
    if (stringed_target_class === 'h') return 1;
    if (stringed_target_class === 'l') return 2;
    if (stringed_target_class === 'n') return 3;
    if (/^\d+$/.test(stringed_target_class)) {
        return 10 + parseInt(stringed_target_class);
    }
    if (stringed_target_class === 'p') return 200;
    if (stringed_target_class === 'unknown') return 300;
    return 999;
};

const sortedAvailableTypes = computed(() => {
    return availableTypes.value.toSorted((a, b) => getClassSortOrder(a.target_class) - getClassSortOrder(b.target_class));
});

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

const categoryAbbrev: Record<string, string> = {
    Wormhole: 'WH',
    Data: 'Data',
    Relic: 'Relic',
    Ore: 'Ore',
    Gas: 'Gas',
    Combat: 'Combat',
};

function getCategoryAbbrev(name?: string | null): string {
    return name ? (categoryAbbrev[name] ?? name) : '—';
}

function handleChange(data: Record<string, unknown>) {
    updateSignature(signature, data);
}

function handleDelete() {
    deleteSignature(signature);
}

function handleCategoryChange(value: AcceptableValue) {
    handleChange({
        signature_category_id: value as number,
        signature_type_id: null,
        map_connection_id: null,
    });
}

function handleTypeChange(value: AcceptableValue) {
    handleChange({ signature_type_id: value as number });
}

function handleMapConnectionChange(value: AcceptableValue) {
    handleChange({ map_connection_id: value as number | null });
}

function startEditId() {
    if (!can_write.value) return;
    signature_id.value = signature.signature_id || '';
    editingId.value = true;
    nextTick(() => {
        idInputRef.value?.focus();
        idInputRef.value?.select();
    });
}

function handleIdInput(event: Event) {
    const target = event.target as HTMLInputElement;
    let value = target.value.replace(/[^a-zA-Z0-9]/g, '').toUpperCase();
    if (value.length >= 4) {
        value = value.slice(0, 3) + '-' + value.slice(3, 6);
    }
    signature_id.value = value;
}

function saveId() {
    const newId = signature_id.value.trim();
    if (newId !== signature.signature_id) {
        handleChange({ signature_id: newId || null });
    }
    editingId.value = false;
}

function cancelEditId() {
    editingId.value = false;
    signature_id.value = signature.signature_id || '';
}

function handleLifetimeChange(lifetime: AcceptableValue) {
    handleChange({
        lifetime: lifetime as string,
        lifetime_updated_at: formatDateToISO(new UTCDate()),
    });
}

function handleMassStatusChange(mass_status: AcceptableValue) {
    const value = mass_status === 'unknown' ? null : mass_status;
    handleChange({ mass_status: value as string | null });
}
</script>

<template>
    <div
        class="flex items-center gap-2 border-b border-border/30 px-3 py-1.5 hover:bg-muted/30 data-deleted:bg-red-500/10 data-new:bg-green-500/10 data-updated:bg-amber-500/15"
        :data-deleted="Data(is_deleted)"
        :data-new="Data(is_new)"
        :data-updated="Data(is_updated)"
    >
        <!-- Signature ID -->
        <div class="w-16 shrink-0">
            <input
                v-if="editingId"
                ref="idInputRef"
                :value="signature_id"
                @input="handleIdInput"
                @blur="saveId"
                @keydown.enter="saveId"
                @keydown.escape="cancelEditId"
                class="h-6 w-full rounded border border-border/50 bg-background/50 px-1.5 font-mono text-xs uppercase focus:border-primary focus:outline-none"
                maxlength="7"
                placeholder="XXX-XXX"
            />
            <button
                v-else
                class="font-mono text-xs hover:text-amber-400"
                :class="can_write ? 'cursor-pointer' : 'cursor-default'"
                @click="startEditId"
            >
                {{ signature.signature_id || '---' }}
            </button>
        </div>

        <!-- Category -->
        <div class="w-24 shrink-0">
            <Select :model-value="signature.signature_category_id" @update:modelValue="handleCategoryChange" :disabled="!can_write">
                <SelectTrigger class="h-6 w-full text-xs">
                    <SelectValue placeholder="Category">
                        {{ getCategoryAbbrev(signature.signature_category?.name) }}
                    </SelectValue>
                </SelectTrigger>
                <SelectContent>
                    <SelectItem v-for="category in signatureCategories" :key="category.id" :value="category.id" class="text-xs">
                        {{ category.name }}
                    </SelectItem>
                </SelectContent>
            </Select>
        </div>

        <!-- Type / Wormhole Info -->
        <div class="min-w-0 flex-1">
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

        <!-- Connection (only for wormholes) -->
        <div v-if="isWormhole" class="min-w-0 flex-1">
            <MapConnectionInput
                :type="signature.signature_type"
                :selected="selected_connection"
                :unconnected_connections="unconnected_connections"
                :connected_connections="connected_connections"
                :model-value="signature.map_connection_id"
                @update:model-value="handleMapConnectionChange"
            />
        </div>

        <!-- Age -->
        <div class="w-10 shrink-0 text-right">
            <SignatureTimeDetails :category="signature.signature_category?.name" :selected_connection="selected_connection" :signature="signature" />
        </div>

        <!-- Actions -->
        <div class="w-6 shrink-0">
            <DropdownMenu v-if="can_write">
                <DropdownMenuTrigger as-child>
                    <Button variant="ghost" size="icon" class="size-6 text-muted-foreground hover:text-foreground">
                        <MoreVertical class="size-3.5" />
                    </Button>
                </DropdownMenuTrigger>
                <DropdownMenuContent align="end" class="w-44">
                    <template v-if="isWormhole">
                        <!-- Mass Status Options -->
                        <DropdownMenuRadioGroup :model-value="signature.mass_status || 'unknown'" @update:model-value="handleMassStatusChange">
                            <DropdownMenuRadioItem value="fresh" class="text-xs">
                                <span class="flex items-center gap-2">
                                    <span class="inline-block size-2 rounded-full bg-neutral-500" />
                                    Fresh Mass
                                </span>
                            </DropdownMenuRadioItem>
                            <DropdownMenuRadioItem value="reduced" class="text-xs">
                                <span class="flex items-center gap-2">
                                    <span class="inline-block size-2 rounded-full bg-amber-500" />
                                    Reduced Mass
                                </span>
                            </DropdownMenuRadioItem>
                            <DropdownMenuRadioItem value="critical" class="text-xs">
                                <span class="flex items-center gap-2">
                                    <span class="inline-block size-2 rounded-full bg-red-500" />
                                    Critical Mass
                                </span>
                            </DropdownMenuRadioItem>
                        </DropdownMenuRadioGroup>

                        <DropdownMenuSeparator />

                        <!-- Lifetime Options -->
                        <DropdownMenuRadioGroup :model-value="signature.lifetime" @update:model-value="handleLifetimeChange">
                            <DropdownMenuRadioItem value="healthy" class="text-xs">
                                <span class="flex items-center gap-2">
                                    <span class="inline-block size-2 rounded-full bg-neutral-500" />
                                    Healthy
                                </span>
                            </DropdownMenuRadioItem>
                            <DropdownMenuRadioItem value="eol" class="text-xs">
                                <span class="flex items-center gap-2">
                                    <span class="inline-block size-2 rounded-full bg-purple-500" />
                                    End of Life
                                </span>
                            </DropdownMenuRadioItem>
                            <DropdownMenuRadioItem value="critical" class="text-xs">
                                <span class="flex items-center gap-2">
                                    <span class="inline-block size-2 rounded-full bg-red-500" />
                                    Critical
                                </span>
                            </DropdownMenuRadioItem>
                        </DropdownMenuRadioGroup>

                        <DropdownMenuSeparator />
                    </template>

                    <!-- Delete Option -->
                    <DropdownMenuItem @click="handleDelete" class="text-xs text-destructive focus:text-destructive">
                        <TrashIcon class="mr-2 size-3.5" />
                        Delete Signature
                    </DropdownMenuItem>
                </DropdownMenuContent>
            </DropdownMenu>
        </div>
    </div>
</template>
