<script setup lang="ts">
import MapUserSettingController from '@/actions/App/Http/Controllers/MapUserSettingController';
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card';
import { Checkbox } from '@/components/ui/checkbox';
import { Label } from '@/components/ui/label';
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from '@/components/ui/select';
import SettingsLayout from '@/layouts/SettingsLayout.vue';
import { TMapSummary } from '@/pages/maps';
import { TMapUserSetting } from '@/types/models';
import { router } from '@inertiajs/vue3';
import { AcceptableValue } from 'reka-ui';

const { map, map_user_settings } = defineProps<{
    map: TMapSummary;
    map_user_settings: TMapUserSetting;
}>();

function updateMapUserSettings(settings: Partial<TMapUserSetting>) {
    router.put(MapUserSettingController.update(map.slug).url, settings, {
        preserveScroll: true,
    });
}

function handleKillmailFilterChange(value: AcceptableValue) {
    if (typeof value === 'string' && (value === 'all' || value === 'jspace' || value === 'kspace')) {
        updateMapUserSettings({ killmail_filter: value });
    }
}

function handleToggleThreatLevel(value: boolean | 'indeterminate') {
    if (typeof value === 'boolean') {
        updateMapUserSettings({ show_threat_level: value });
    }
}

function handleToggleStaticsFirst(value: boolean | 'indeterminate') {
    if (typeof value === 'boolean') {
        updateMapUserSettings({ show_statics_first: value });
    }
}

function handleToggleCompactSignatureList(value: boolean | 'indeterminate') {
    if (typeof value === 'boolean') {
        updateMapUserSettings({ compact_signature_list: value });
    }
}
</script>

<template>
    <SettingsLayout :map="map" title="Preferences" description="Configure your personal preferences and tracking settings for this map">
        <div class="space-y-6">
            <Card>
                <CardHeader>
                    <CardTitle class="text-xl font-semibold">Map Preferences</CardTitle>
                    <CardDescription>Configure your personal preferences for this map</CardDescription>
                </CardHeader>
                <CardContent class="space-y-6">
                    <div class="space-y-3">
                        <Label class="text-sm font-medium">Killmail Filter</Label>
                        <Select :model-value="map_user_settings.killmail_filter" @update:model-value="handleKillmailFilterChange">
                            <SelectTrigger class="w-full">
                                <SelectValue placeholder="Select killmail filter" />
                            </SelectTrigger>
                            <SelectContent>
                                <SelectItem value="all">All Systems</SelectItem>
                                <SelectItem value="kspace">K-Space Only</SelectItem>
                                <SelectItem value="jspace">J-Space Only</SelectItem>
                            </SelectContent>
                        </Select>
                        <div class="text-sm text-muted-foreground">Filter which killmails are displayed based on system type</div>
                    </div>

                    <div class="flex items-center justify-between">
                        <div class="space-y-0.5">
                            <Label class="text-sm font-medium">Show Threat Level</Label>
                            <div class="text-sm text-muted-foreground">Display a colored ring around wormhole systems based on killmail activity</div>
                        </div>
                        <Checkbox :model-value="map_user_settings.show_threat_level" @update:model-value="handleToggleThreatLevel" />
                    </div>

                    <div class="flex items-center justify-between">
                        <div class="space-y-0.5">
                            <Label class="text-sm font-medium">Show Statics First</Label>
                            <div class="text-sm text-muted-foreground">
                                Group this system's static wormholes at the top of the signature type selector
                            </div>
                        </div>
                        <Checkbox :model-value="map_user_settings.show_statics_first" @update:model-value="handleToggleStaticsFirst" />
                    </div>

                    <div class="flex items-center justify-between">
                        <div class="space-y-0.5">
                            <Label class="text-sm font-medium">Compact Signature List</Label>
                            <div class="text-sm text-muted-foreground">Show signatures in a denser list with less spacing between rows</div>
                        </div>
                        <Checkbox :model-value="map_user_settings.compact_signature_list" @update:model-value="handleToggleCompactSignatureList" />
                    </div>
                </CardContent>
            </Card>
        </div>
    </SettingsLayout>
</template>
