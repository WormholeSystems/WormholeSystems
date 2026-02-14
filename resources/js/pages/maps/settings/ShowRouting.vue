<script setup lang="ts">
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card';
import { Checkbox } from '@/components/ui/checkbox';
import { Label } from '@/components/ui/label';
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from '@/components/ui/select';
import { Slider } from '@/components/ui/slider';
import SettingsLayout from '@/layouts/SettingsLayout.vue';
import { TMapSummary } from '@/pages/maps';
import MapUserSettings from '@/routes/map-user-settings';
import { TLifetimeStatus, TMapUserSetting, TRoutePreference } from '@/types/models';
import { router } from '@inertiajs/vue3';
import { AcceptableValue } from 'reka-ui';
import { ref, watch } from 'vue';

const { map, map_user_settings } = defineProps<{
    map: TMapSummary;
    map_user_settings: TMapUserSetting;
}>();

// Local ref for security penalty for smooth dragging
const localSecurityPenalty = ref(map_user_settings.security_penalty);

// Watch for server updates
watch(
    () => map_user_settings.security_penalty,
    (newValue) => {
        localSecurityPenalty.value = newValue;
    },
);

function updateMapUserSettings(settings: Partial<TMapUserSetting>) {
    router.put(MapUserSettings.update(map_user_settings.id), settings, {
        preserveState: true,
        preserveScroll: true,
        only: ['map_user_settings', 'map'],
    });
}

function handleLifetimeStatusChange(value: AcceptableValue) {
    if (typeof value === 'string' && (value === 'critical' || value === 'eol' || value === 'healthy')) {
        updateMapUserSettings({
            route_allow_lifetime_status: value as TLifetimeStatus,
        });
    }
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

function handleRoutePreferenceChange(value: AcceptableValue) {
    if (typeof value === 'string' && (value === 'shorter' || value === 'safer' || value === 'less_secure')) {
        updateMapUserSettings({
            route_preference: value as TRoutePreference,
        });
    }
}

function handleSecurityPenaltyChange(value: number[] | undefined) {
    if (value && value[0] !== undefined) {
        localSecurityPenalty.value = value[0];
    }
}

function handleSecurityPenaltyCommit(value: number[]) {
    if (value[0] !== undefined) {
        updateMapUserSettings({
            security_penalty: value[0],
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
                    <div class="space-y-3">
                        <Label class="text-sm font-medium">Lifetime Status Filter</Label>
                        <Select :model-value="map_user_settings.route_allow_lifetime_status" @update:model-value="handleLifetimeStatusChange">
                            <SelectTrigger class="w-full">
                                <SelectValue placeholder="Select lifetime status filter" />
                            </SelectTrigger>
                            <SelectContent>
                                <SelectItem value="critical">All Connections</SelectItem>
                                <SelectItem value="eol">Healthy & EOL</SelectItem>
                                <SelectItem value="healthy">Healthy Only</SelectItem>
                            </SelectContent>
                        </Select>
                        <div class="text-sm text-muted-foreground">Set the minimum lifetime for connections to include in route calculations</div>
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

                    <div class="space-y-3">
                        <Label class="text-sm font-medium">Route Preference</Label>
                        <Select :model-value="map_user_settings.route_preference" @update:model-value="handleRoutePreferenceChange">
                            <SelectTrigger class="w-full">
                                <SelectValue placeholder="Select route preference" />
                            </SelectTrigger>
                            <SelectContent>
                                <SelectItem value="shorter">Shorter</SelectItem>
                                <SelectItem value="safer">Safer</SelectItem>
                                <SelectItem value="less_secure">Less Secure</SelectItem>
                            </SelectContent>
                        </Select>
                        <div class="text-sm text-muted-foreground">
                            <strong>Shorter:</strong> Find routes with the fewest jumps<br />
                            <strong>Safer:</strong> Prefer high-security space, avoid low-sec and null-sec<br />
                            <strong>Less Secure:</strong> Prefer low-security space, avoid high-sec and null-sec
                        </div>
                    </div>

                    <div class="space-y-3" v-if="map_user_settings.route_preference !== 'shorter'">
                        <Label class="text-sm font-medium">Security Penalty: {{ localSecurityPenalty }}%</Label>
                        <Slider
                            :model-value="[localSecurityPenalty]"
                            @update:model-value="handleSecurityPenaltyChange"
                            @value-commit="handleSecurityPenaltyCommit"
                            :min="0"
                            :max="100"
                            :step="5"
                            class="w-full"
                        />
                        <div class="text-sm text-muted-foreground">
                            Controls the exponential penalty multiplier (0-100). Higher values make the route preference more strict, lower values
                            allow more flexibility. Default is 50.
                        </div>
                    </div>
                </CardContent>
            </Card>
        </div>
    </SettingsLayout>
</template>
