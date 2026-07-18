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
import { useMapUserSettings } from '@/composables/useMapUserSettings';
import usePermission from '@/composables/usePermission';
import { getTypesByCategory, signatureCategories } from '@/const/signatures';
import { classSortWeight } from '@/const/solarsystemClasses';
import { Data } from '@/lib/data';
import { formatDateToISO } from '@/lib/utils';
import { deleteSignature, TProcessedConnection, updateMapConnection, updateSignature } from '@/map/api';
import type { TResolvedSelectedMapSolarsystem } from '@/pages/maps';
import { TSignature } from '@/types/models';
import { UTCDate } from '@date-fns/utc';
import type { FormDataConvertible } from '@inertiajs/core';
import { syncRefs } from '@vueuse/core';
import { Check, Cloud, Database, Fan, Gem, Heart, Landmark, MoreVertical, Shield, Swords } from 'lucide-vue-next';
import { AcceptableValue } from 'reka-ui';
import { type Component, computed, nextTick, ref, toRef } from 'vue';

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

const { canEdit: can_write } = usePermission();

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

const solarsystem_class = computed(() => selected_map_solarsystem.solarsystem.class);

const availableTypes = computed(() => {
    if (!signature.signature_category_id) return [];
    return getTypesByCategory(signature.signature_category_id).filter((type) => type.spawn_areas?.includes(solarsystem_class.value));
});

const sortedAvailableTypes = computed(() => {
    return availableTypes.value.toSorted((a, b) => classSortWeight(a.target_class) - classSortWeight(b.target_class));
});

const wormholeCategoryId = computed(() => {
    return signatureCategories.find((cat) => cat.name === 'Wormhole')?.id;
});

const isWormhole = computed(() => {
    return signature.signature_category_id === wormholeCategoryId.value;
});

const current_class = computed(() => {
    if (!selected_connection.value?.target) return null;
    return selected_connection.value.target.solarsystem.class;
});

const map_user_settings = useMapUserSettings();

const static_signatures = computed<string[]>(() => {
    if (!map_user_settings.value.show_statics_first) {
        return [];
    }
    return (selected_map_solarsystem.solarsystem.statics ?? []).map((wormhole_static) => wormhole_static.name);
});

const categoryAbbrev: Record<string, string> = {
    Wormhole: 'WH',
    'Data Site': 'Data',
    'Relic Site': 'Relic',
    'Ore Site': 'Ore',
    'Gas Site': 'Gas',
    'Combat Site': 'Combat',
    'Homefront Operations': 'HF',
};

const categoryIcon: Record<string, Component> = {
    Wormhole: Fan,
    'Data Site': Database,
    'Relic Site': Landmark,
    'Ore Site': Gem,
    'Gas Site': Cloud,
    'Combat Site': Swords,
    'Homefront Operations': Shield,
};

const categoryColor: Record<string, string> = {
    Wormhole: 'text-sky-400',
    'Data Site': 'text-cyan-400',
    'Relic Site': 'text-amber-400',
    'Combat Site': 'text-green-400',
    'Gas Site': 'text-orange-400',
    'Ore Site': 'text-yellow-400',
    'Homefront Operations': 'text-rose-400',
};

function getCategoryAbbrev(name?: string | null): string {
    return name ? (categoryAbbrev[name] ?? name) : '—';
}

function handleChange(data: Record<string, FormDataConvertible>) {
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

function handleTogglePreserveMass() {
    if (!selected_connection.value) return;
    updateMapConnection(selected_connection.value, { preserve_mass: !selected_connection.value.preserve_mass });
}
</script>

<template>
    <div
        class="flex items-center gap-2 border-b border-border/30 px-3 hover:bg-muted/30 data-deleted:bg-red-500/10 data-new:bg-green-500/10 data-updated:bg-amber-500/15"
        :class="map_user_settings.compact_signature_list ? 'py-0.5' : 'py-1.5'"
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
                class="w-full rounded border border-border/50 bg-background/50 px-1.5 font-mono text-xs uppercase focus:border-primary focus:outline-none"
                :class="map_user_settings.compact_signature_list ? 'h-5' : 'h-6'"
                maxlength="7"
                placeholder="XXX-XXX"
            />
            <button
                v-else
                class="flex items-center font-mono text-xs hover:text-amber-400"
                :class="[can_write ? 'cursor-pointer' : 'cursor-default', map_user_settings.compact_signature_list ? 'h-5' : 'h-6']"
                @click="startEditId"
            >
                {{ signature.signature_id || '---' }}
            </button>
        </div>

        <!-- Category -->
        <div class="w-24 shrink-0">
            <Select :model-value="signature.signature_category_id" @update:modelValue="handleCategoryChange" :disabled="!can_write">
                <SelectTrigger class="w-full text-xs" :class="map_user_settings.compact_signature_list ? '!h-5 !py-0' : ''">
                    <SelectValue placeholder="Category">
                        <span class="flex items-center gap-1">
                            <component
                                :is="categoryIcon[signature.signature_category?.name ?? '']"
                                class="size-3 shrink-0"
                                :class="categoryColor[signature.signature_category?.name ?? '']"
                            />
                            {{ getCategoryAbbrev(signature.signature_category?.name) }}
                        </span>
                    </SelectValue>
                </SelectTrigger>
                <SelectContent>
                    <SelectItem v-for="category in signatureCategories" :key="category.id" :value="category.id" class="text-xs">
                        <span class="flex items-center gap-1.5">
                            <component :is="categoryIcon[category.name]" class="size-3 shrink-0" :class="categoryColor[category.name]" />
                            {{ category.name }}
                        </span>
                    </SelectItem>
                </SelectContent>
            </Select>
        </div>

        <!-- Type / Wormhole Info: capped for wormhole rows so the connection column gets the extra room. -->
        <div class="min-w-0 flex-1" :class="{ 'max-w-44': isWormhole }">
            <WormholeTypeInput
                v-if="isWormhole"
                :model-value="signature.signature_type_id"
                @update:model-value="handleTypeChange"
                :can_write="can_write"
                :wormhole_options="sortedAvailableTypes"
                :current_class="current_class"
                :static_signatures="static_signatures"
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
                :disabled="!can_write"
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
                    <Button
                        variant="ghost"
                        size="icon"
                        class="text-muted-foreground hover:text-foreground"
                        :class="map_user_settings.compact_signature_list ? 'size-5' : 'size-6'"
                    >
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

                        <template v-if="selected_connection">
                            <DropdownMenuItem @select.prevent="handleTogglePreserveMass" class="text-xs">
                                <Heart class="mr-2 size-3.5" />
                                Preserve mass
                                <Check v-if="selected_connection.preserve_mass" class="ml-auto size-3.5" />
                            </DropdownMenuItem>

                            <DropdownMenuSeparator />
                        </template>
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
