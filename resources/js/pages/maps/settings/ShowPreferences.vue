<script setup lang="ts">
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card';
import { Checkbox } from '@/components/ui/checkbox';
import { Label } from '@/components/ui/label';
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from '@/components/ui/select';
import SettingsLayout from '@/layouts/SettingsLayout.vue';
import MapUserSettings from '@/routes/map-user-settings';
import { TMap, TMapUserSetting } from '@/types/models';
import { router } from '@inertiajs/vue3';
import { AcceptableValue } from 'reka-ui';

const { map, map_user_settings } = defineProps<{
    map: TMap;
    map_user_settings: TMapUserSetting;
}>();

function updateMapUserSettings(settings: Partial<TMapUserSetting>) {
    router.put(MapUserSettings.update(map.slug), settings, {
        preserveScroll: true,
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

function handleKillmailFilterChange(value: AcceptableValue) {
    if (typeof value === 'string' && (value === 'all' || value === 'jspace' || value === 'kspace')) {
        updateMapUserSettings({ killmail_filter: value });
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
                </CardContent>
            </Card>
        </div>
    </SettingsLayout>
</template>
