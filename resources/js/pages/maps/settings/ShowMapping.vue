<script setup lang="ts">
import MapUserSettingController from '@/actions/App/Http/Controllers/MapUserSettingController';
import PlusIcon from '@/components/icons/PlusIcon.vue';
import SolarsystemEffect from '@/components/map/SolarsystemEffect.vue';
import SolarsystemClass from '@/components/solarsystem/SolarsystemClass.vue';
import { Button } from '@/components/ui/button';
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card';
import { Checkbox } from '@/components/ui/checkbox';
import { Combobox, ComboboxAnchor, ComboboxEmpty, ComboboxGroup, ComboboxInput, ComboboxItem, ComboboxList } from '@/components/ui/combobox';
import { Dialog, DialogContent, DialogDescription, DialogHeader, DialogTitle, DialogTrigger } from '@/components/ui/dialog';
import { Label } from '@/components/ui/label';
import { useMapIgnoredSystems } from '@/composables/useMapIgnoredSystems';
import usePermission from '@/composables/usePermission';
import { useStaticData } from '@/composables/useStaticData';
import SettingsLayout from '@/layouts/SettingsLayout.vue';
import { TMapSummary } from '@/pages/maps';
import { TMapUserSetting } from '@/types/models';
import { TStaticSolarsystem } from '@/types/static-data';
import { router } from '@inertiajs/vue3';
import { Trash2 } from 'lucide-vue-next';
import { computed, ref } from 'vue';

const { map, map_user_settings, map_ignored_systems } = defineProps<{
    map: TMapSummary;
    map_user_settings: TMapUserSetting;
    map_ignored_systems: number[];
}>();

const { canEdit } = usePermission();
const { ignoreSolarsystem, unignoreSolarsystem, clearIgnoreList } = useMapIgnoredSystems();
const { staticData, loadStaticData } = useStaticData();

void loadStaticData();

function updateMapUserSettings(settings: Partial<TMapUserSetting>) {
    router.put(MapUserSettingController.update(map.slug).url, settings, {
        preserveState: true,
        preserveScroll: true,
        only: ['map_user_settings', 'map'],
    });
}

function handleToggleTracking(value: boolean | 'indeterminate') {
    if (typeof value === 'boolean') {
        updateMapUserSettings({ tracking_allowed: value });
    }
}

function handleToggleActiveTracking(value: boolean | 'indeterminate') {
    if (typeof value === 'boolean') {
        updateMapUserSettings({ is_tracking: value });
    }
}

function handlePromptForSignatureChange(value: boolean | 'indeterminate') {
    if (typeof value === 'boolean') {
        updateMapUserSettings({ prompt_for_signature_enabled: value });
    }
}

function handleSuggestAliasChange(value: boolean | 'indeterminate') {
    if (typeof value === 'boolean') {
        updateMapUserSettings({ suggest_alias_enabled: value });
    }
}

function handleAutoConfirmChange(value: boolean | 'indeterminate') {
    if (typeof value === 'boolean') {
        updateMapUserSettings({ auto_confirm_signatures: value });
    }
}

function handleConcatAliasChange(value: boolean | 'indeterminate') {
    if (typeof value === 'boolean') {
        updateMapUserSettings({ concat_alias_disabled: value });
    }
}

function handleCopyBookmarkChange(value: boolean | 'indeterminate') {
    if (typeof value === 'boolean') {
        updateMapUserSettings({ copy_bookmark_enabled: value });
    }
}

const solarsystems = computed(() => staticData.value?.solarsystems ?? []);

const ignoredSolarsystems = computed(() =>
    map_ignored_systems
        .map((id) => solarsystems.value.find((solarsystem) => solarsystem.id === id))
        .filter((solarsystem): solarsystem is TStaticSolarsystem => Boolean(solarsystem)),
);

const adding = ref(false);
const search = ref('');

const filteredSolarsystems = computed(() => {
    const query = search.value.trim().toLowerCase();
    if (!query) {
        return [] as TStaticSolarsystem[];
    }

    return solarsystems.value.filter((solarsystem) => solarsystem.name.toLowerCase().includes(query)).slice(0, 25);
});

const new_solarsystems = computed(() => filteredSolarsystems.value.filter((solarsystem) => !map_ignored_systems.includes(solarsystem.id)));

const existing_solarsystems = computed(() => filteredSolarsystems.value.filter((solarsystem) => map_ignored_systems.includes(solarsystem.id)));

function handleSolarsystemSelect(solarsystem: TStaticSolarsystem) {
    ignoreSolarsystem(solarsystem.id, {
        onSuccess() {
            adding.value = false;
            search.value = '';
        },
    });
}
</script>

<template>
    <SettingsLayout :map="map" title="Mapping" description="Configure tracking behavior and which systems should never be auto-mapped">
        <div class="space-y-6">
            <Card>
                <CardHeader>
                    <CardTitle class="text-xl font-semibold">Tracking</CardTitle>
                    <CardDescription>Configure how your character's location is tracked on this map</CardDescription>
                </CardHeader>
                <CardContent class="space-y-6">
                    <div class="flex items-center justify-between">
                        <div class="space-y-0.5">
                            <Label class="text-sm font-medium">Location Tracking</Label>
                            <div class="text-sm text-muted-foreground">Allow this map to track your character's location</div>
                        </div>
                        <Checkbox :model-value="map_user_settings.tracking_allowed" @update:model-value="handleToggleTracking" />
                    </div>

                    <div class="flex items-center justify-between" v-if="map_user_settings.tracking_allowed">
                        <div class="space-y-0.5">
                            <Label class="text-sm font-medium">Active Tracking</Label>
                            <div class="text-sm text-muted-foreground">Currently broadcasting your location to this map</div>
                        </div>
                        <Checkbox :model-value="map_user_settings.is_tracking" @update:model-value="handleToggleActiveTracking" />
                    </div>

                    <div class="flex items-center justify-between">
                        <div class="space-y-0.5">
                            <Label class="text-sm font-medium">Prompt for Signature</Label>
                            <div class="text-sm text-muted-foreground">
                                Determines if you are prompted to select the signature you jumped through when entering a new system
                            </div>
                        </div>
                        <Checkbox
                            :model-value="map_user_settings.prompt_for_signature_enabled"
                            @update:model-value="handlePromptForSignatureChange"
                        />
                    </div>

                    <div class="flex items-center justify-between">
                        <div class="space-y-0.5">
                            <Label class="text-sm font-medium">Auto confirm signatures</Label>
                            <div class="text-sm text-muted-foreground">
                                Automatically confirm the signature dialog to generate aliases and bookmarks without user input
                            </div>
                        </div>
                        <Checkbox :model-value="map_user_settings.auto_confirm_signatures" @update:model-value="handleAutoConfirmChange" />
                    </div>

                    <div class="flex items-center justify-between">
                        <div class="space-y-0.5">
                            <Label class="text-sm font-medium">Suggest Alias</Label>
                            <div class="text-sm text-muted-foreground">
                                Pre-fill a guessed chain alias (1, 11, 12…) for new systems you jump into while tracking
                            </div>
                        </div>
                        <Checkbox :model-value="map_user_settings.suggest_alias_enabled" @update:model-value="handleSuggestAliasChange" />
                    </div>

                    <div class="flex items-center justify-between">
                        <div class="space-y-0.5">
                            <Label class="text-sm font-medium">Disable Alias Concatination</Label>
                            <div class="text-sm text-muted-foreground">
                                The suggested alias will always start with a 1 and ignore what was filled in the origin
                            </div>
                        </div>
                        <Checkbox :model-value="map_user_settings.concat_alias_disabled" @update:model-value="handleConcatAliasChange" />
                    </div>

                    <div class="flex items-center justify-between">
                        <div class="space-y-0.5">
                            <Label class="text-sm font-medium">Copy Bookmark</Label>
                            <div class="text-sm text-muted-foreground">
                                Copy the connection bookmark name to your clipboard after selecting the signature you jumped through
                            </div>
                        </div>
                        <Checkbox :model-value="map_user_settings.copy_bookmark_enabled" @update:model-value="handleCopyBookmarkChange" />
                    </div>
                </CardContent>
            </Card>

            <Card>
                <CardHeader class="flex flex-row items-start justify-between gap-4">
                    <div class="space-y-1.5">
                        <CardTitle class="text-xl font-semibold">Ignored Systems</CardTitle>
                        <CardDescription>
                            Systems on this list are never added to the map automatically while tracking. This list is shared by everyone on the map.
                        </CardDescription>
                    </div>
                    <Dialog v-model:open="adding" v-if="canEdit">
                        <DialogTrigger as-child>
                            <Button variant="outline" size="sm">
                                <PlusIcon class="mr-2 h-4 w-4" />
                                Add System
                            </Button>
                        </DialogTrigger>
                        <DialogContent>
                            <DialogHeader>
                                <DialogTitle>Add Ignored System</DialogTitle>
                                <DialogDescription>Search for a solar system to exclude it from auto-mapping.</DialogDescription>
                            </DialogHeader>
                            <Combobox class="rounded-lg border bg-neutral-900">
                                <ComboboxAnchor>
                                    <ComboboxInput v-model="search" auto-focus />
                                </ComboboxAnchor>
                                <ComboboxList class="w-115" align="start">
                                    <ComboboxEmpty>No results found</ComboboxEmpty>
                                    <ComboboxGroup heading="Search Results" v-if="new_solarsystems.length > 0" class="grid grid-cols-[auto_1fr_auto]">
                                        <ComboboxItem
                                            v-for="solarsystem in new_solarsystems"
                                            :key="solarsystem.id"
                                            :value="solarsystem.name"
                                            @select.prevent="() => handleSolarsystemSelect(solarsystem)"
                                            class="col-span-full grid grid-cols-subgrid"
                                        >
                                            <div class="justify-self-center">
                                                <SolarsystemClass
                                                    :wormhole_class="solarsystem.class"
                                                    :security="solarsystem.security"
                                                    :name="solarsystem.name"
                                                />
                                            </div>
                                            <span class="whitespace-nowrap">{{ solarsystem.name }}</span>
                                            <span class="truncate text-muted-foreground" v-if="!solarsystem.class">{{
                                                solarsystem.region?.name
                                            }}</span>
                                            <div class="justify-self-end" v-else-if="solarsystem.effect">
                                                <SolarsystemEffect :effect="solarsystem.effect" />
                                            </div>
                                        </ComboboxItem>
                                    </ComboboxGroup>
                                    <ComboboxGroup
                                        heading="Already Ignored"
                                        v-if="existing_solarsystems.length > 0"
                                        class="grid grid-cols-[auto_1fr_auto]"
                                    >
                                        <ComboboxItem
                                            v-for="solarsystem in existing_solarsystems"
                                            :key="solarsystem.id"
                                            :value="solarsystem.name"
                                            class="col-span-full grid grid-cols-subgrid"
                                            disabled
                                        >
                                            <div class="justify-self-center">
                                                <SolarsystemClass
                                                    :wormhole_class="solarsystem.class"
                                                    :security="solarsystem.security"
                                                    :name="solarsystem.name"
                                                />
                                            </div>
                                            <span class="whitespace-nowrap">{{ solarsystem.name }}</span>
                                            <span class="truncate text-muted-foreground" v-if="!solarsystem.class">{{
                                                solarsystem.region?.name
                                            }}</span>
                                            <div class="justify-self-end" v-else-if="solarsystem.effect">
                                                <SolarsystemEffect :effect="solarsystem.effect" />
                                            </div>
                                        </ComboboxItem>
                                    </ComboboxGroup>
                                </ComboboxList>
                            </Combobox>
                        </DialogContent>
                    </Dialog>
                </CardHeader>
                <CardContent class="space-y-4">
                    <p v-if="ignoredSolarsystems.length === 0" class="text-sm text-muted-foreground">No systems are being ignored.</p>

                    <ul v-else class="divide-y divide-border rounded-lg border">
                        <li v-for="solarsystem in ignoredSolarsystems" :key="solarsystem.id" class="flex items-center gap-3 px-3 py-2">
                            <SolarsystemClass :wormhole_class="solarsystem.class" :security="solarsystem.security" :name="solarsystem.name" />
                            <span class="font-medium">{{ solarsystem.name }}</span>
                            <span class="truncate text-sm text-muted-foreground">{{ solarsystem.region?.name }}</span>
                            <Button
                                v-if="canEdit"
                                variant="ghost"
                                size="icon"
                                class="ml-auto text-muted-foreground hover:text-destructive"
                                @click="unignoreSolarsystem(solarsystem.id)"
                            >
                                <Trash2 class="h-4 w-4" />
                            </Button>
                        </li>
                    </ul>

                    <div v-if="canEdit && ignoredSolarsystems.length > 0" class="flex justify-end">
                        <Button variant="ghost" size="sm" class="text-muted-foreground hover:text-destructive" @click="clearIgnoreList()">
                            Clear all
                        </Button>
                    </div>
                </CardContent>
            </Card>
        </div>
    </SettingsLayout>
</template>
