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
        preserveState: true,
        preserveScroll: true,
        only: ['map_user_settings'],
    });
}

function handleToggleEol(value: boolean | 'indeterminate') {
    updateMapUserSettings({
        route_allow_eol: value === true,
    });
}

function handleToggleEveScout(value: boolean | 'indeterminate') {
    updateMapUserSettings({
        route_use_evescout: value === true,
    });
}

function handleMassStatusChange(value: AcceptableValue) {
    if (typeof value === 'string' && (value === 'critical' || value === 'reduced' || value === 'fresh')) {
        updateMapUserSettings({
            route_allow_mass_status: value,
        });
    }
}
</script>

<template>
    <SettingsLayout :map="map" title="Routing Settings" description="Configure how routes are calculated and displayed for this map">
        <div class="space-y-6">
            <Card>
                <CardHeader>
                    <CardTitle class="text-xl font-semibold">Route Calculation</CardTitle>
                    <CardDescription>Configure how routes are calculated for this map</CardDescription>
                </CardHeader>
                <CardContent class="space-y-6">
                    <div class="flex items-center justify-between">
                        <div class="space-y-0.5">
                            <Label class="text-sm font-medium">Allow End of Life Connections</Label>
                            <div class="text-sm text-muted-foreground">
                                Include wormhole connections that are near end of life in route calculations
                            </div>
                        </div>
                        <Checkbox :model-value="map_user_settings.route_allow_eol" @update:model-value="handleToggleEol" />
                    </div>

                    <div class="flex items-center justify-between">
                        <div class="space-y-0.5">
                            <Label class="text-sm font-medium">Use EVE Scout Data</Label>
                            <div class="text-sm text-muted-foreground">Include EVE Scout wormhole data in route calculations</div>
                        </div>
                        <Checkbox :model-value="map_user_settings.route_use_evescout" @update:model-value="handleToggleEveScout" />
                    </div>

                    <div class="space-y-3">
                        <Label class="text-sm font-medium">Mass Status Filter</Label>
                        <Select :model-value="map_user_settings.route_allow_mass_status" @update:model-value="handleMassStatusChange">
                            <SelectTrigger class="w-full">
                                <SelectValue placeholder="Select mass status filter" />
                            </SelectTrigger>
                            <SelectContent>
                                <SelectItem value="critical">All Connections</SelectItem>
                                <SelectItem value="reduced">Fresh & Reduced Mass</SelectItem>
                                <SelectItem value="fresh">Fresh Mass Only</SelectItem>
                            </SelectContent>
                        </Select>
                        <div class="text-sm text-muted-foreground">Set the minimum mass level for connections to include in route calculations</div>
                    </div>
                </CardContent>
            </Card>
        </div>
    </SettingsLayout>
</template>
