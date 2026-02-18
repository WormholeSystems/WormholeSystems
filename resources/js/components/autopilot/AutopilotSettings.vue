<script setup lang="ts">
import SettingsIcon from '@/components/icons/SettingsIcon.vue';
import { Checkbox } from '@/components/ui/checkbox';
import { Label } from '@/components/ui/label';
import MapPanelHeaderActionButton from '@/components/ui/map-panel/MapPanelHeaderActionButton.vue';
import { Popover, PopoverContent, PopoverTrigger } from '@/components/ui/popover';
import { RadioGroup, RadioGroupItem } from '@/components/ui/radio-group';
import { Slider } from '@/components/ui/slider';
import { Tooltip, TooltipContent, TooltipTrigger } from '@/components/ui/tooltip';
import { updateMapUserSettings } from '@/composables/map';
import { useMapUserSettings } from '@/composables/useMapUserSettings';
import { useShowMap } from '@/composables/useShowMap';
import { TLifetimeStatus, TMassStatus } from '@/types/models';
import { ref, watch } from 'vue';

const map_user_settings = useMapUserSettings();
const page = useShowMap();

const localSecurityPenalty = ref(map_user_settings.value.security_penalty);

watch(
    () => map_user_settings.value.security_penalty,
    (newValue) => {
        localSecurityPenalty.value = newValue;
    },
);

function handleLifetimeStatusChange(value: string) {
    updateMapUserSettings(page.props.map.slug, {
        route_allow_lifetime_status: value as TLifetimeStatus,
    });
}

function handleToggleMass(value: string) {
    updateMapUserSettings(page.props.map.slug, {
        route_allow_mass_status: value as TMassStatus,
    });
}

function handleToggleEveScout(value: boolean | 'indeterminate') {
    updateMapUserSettings(page.props.map.slug, {
        route_use_evescout: value === true,
    });
}

function handleRoutePreferenceChange(value: string) {
    if (value === 'shorter' || value === 'safer' || value === 'less_secure') {
        updateMapUserSettings(page.props.map.slug, {
            route_preference: value,
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
        updateMapUserSettings(page.props.map.slug, {
            security_penalty: value[0],
        });
    }
}
</script>

<template>
    <Popover>
        <PopoverTrigger>
            <Tooltip>
                <TooltipTrigger as-child>
                    <MapPanelHeaderActionButton size="icon">
                        <SettingsIcon />
                    </MapPanelHeaderActionButton>
                </TooltipTrigger>
                <TooltipContent>
                    <p>Autopilot settings</p>
                </TooltipContent>
            </Tooltip>
        </PopoverTrigger>
        <PopoverContent class="w-80 p-3">
            <div class="grid auto-cols-[auto_1fr_auto] gap-x-1 gap-y-1">
                <h4 class="col-span-3 text-xs text-muted-foreground">Wormhole Lifetime</h4>
                <RadioGroup
                    :model-value="map_user_settings.route_allow_lifetime_status"
                    @update:model-value="handleLifetimeStatusChange"
                    class="col-span-3 grid grid-cols-subgrid gap-1"
                >
                    <RadioGroupItem value="critical" id="lifetime-critical" />
                    <Label for="lifetime-critical" class="cursor-pointer text-xs font-medium">Critical</Label>
                    <span class="text-xs text-muted-foreground">&lt; 1 hour</span>
                    <RadioGroupItem value="eol" id="lifetime-eol" />
                    <Label for="lifetime-eol" class="cursor-pointer text-xs font-medium">End of Life</Label>
                    <span class="text-xs text-muted-foreground">&lt; 4 hours</span>
                    <RadioGroupItem value="healthy" id="lifetime-healthy" />
                    <Label for="lifetime-healthy" class="cursor-pointer text-xs font-medium">Healthy Only</Label>
                    <span class="text-xs text-muted-foreground">&gt; 4 hours</span>
                </RadioGroup>

                <h4 class="col-span-3 mt-2 text-xs text-muted-foreground">Wormhole Mass</h4>
                <RadioGroup
                    :model-value="map_user_settings.route_allow_mass_status"
                    @update:model-value="handleToggleMass"
                    class="col-span-3 grid grid-cols-subgrid gap-1"
                >
                    <RadioGroupItem value="critical" id="mass-critical" />
                    <Label for="mass-critical" class="cursor-pointer text-xs font-medium">Critical Mass</Label>
                    <span class="text-xs text-muted-foreground">&lt; 10%</span>
                    <RadioGroupItem value="reduced" id="mass-reduced" />
                    <Label for="mass-reduced" class="cursor-pointer text-xs font-medium">Reduced Mass</Label>
                    <span class="text-xs text-muted-foreground">&lt; 50%</span>
                    <RadioGroupItem value="fresh" id="mass-fresh" />
                    <Label for="mass-fresh" class="cursor-pointer text-xs font-medium">High Mass</Label>
                    <span class="text-xs text-muted-foreground">&gt; 50%</span>
                </RadioGroup>
                <h4 class="col-span-3 mt-2 text-xs text-muted-foreground">Information Sources</h4>
                <div class="col-span-3 grid grid-cols-subgrid">
                    <Checkbox :model-value="map_user_settings.route_use_evescout" @update:model-value="handleToggleEveScout" id="evescout-checkbox" />
                    <label for="evescout-checkbox" class="cursor-pointer text-xs font-medium"> Use EVE Scout </label>
                </div>

                <h4 class="col-span-3 mt-2 text-xs text-muted-foreground">Route Preference</h4>
                <RadioGroup
                    :model-value="map_user_settings.route_preference"
                    @update:model-value="handleRoutePreferenceChange"
                    class="col-span-3 grid grid-cols-subgrid gap-1"
                >
                    <RadioGroupItem value="shorter" id="pref-shorter" />
                    <Label for="pref-shorter" class="cursor-pointer text-xs font-medium">Shorter</Label>
                    <span class="text-xs text-muted-foreground">Min jumps</span>
                    <RadioGroupItem value="safer" id="pref-safer" />
                    <Label for="pref-safer" class="cursor-pointer text-xs font-medium">Safer</Label>
                    <span class="text-xs text-muted-foreground">High-sec</span>
                    <RadioGroupItem value="less_secure" id="pref-less-secure" />
                    <Label for="pref-less-secure" class="cursor-pointer text-xs font-medium">Less Secure</Label>
                    <span class="text-xs text-muted-foreground">Low-sec</span>
                </RadioGroup>

                <div v-if="map_user_settings.route_preference !== 'shorter'" class="col-span-3 mt-2 space-y-1">
                    <Label class="text-xs font-medium">Security Penalty: {{ localSecurityPenalty }}%</Label>
                    <Slider
                        :model-value="[localSecurityPenalty]"
                        @update:model-value="handleSecurityPenaltyChange"
                        @value-commit="handleSecurityPenaltyCommit"
                        :min="0"
                        :max="100"
                        :step="5"
                        class="w-full"
                    />
                </div>
            </div>
        </PopoverContent>
    </Popover>
</template>

<style scoped></style>
