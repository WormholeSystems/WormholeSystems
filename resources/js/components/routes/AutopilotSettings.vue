<script setup lang="ts">
import SettingsIcon from '@/components/icons/SettingsIcon.vue';
import { Button } from '@/components/ui/button';
import { Checkbox } from '@/components/ui/checkbox';
import { Label } from '@/components/ui/label';
import { Popover, PopoverContent, PopoverTrigger } from '@/components/ui/popover';
import { RadioGroup, RadioGroupItem } from '@/components/ui/radio-group';
import { Tooltip, TooltipContent, TooltipTrigger } from '@/components/ui/tooltip';
import { updateMapUserSettings } from '@/composables/map';
import { useMapUserSettings } from '@/composables/useMapUserSettings';
import { TMassStatus } from '@/types/models';

const map_user_settings = useMapUserSettings();

function handleToggleEol(value: boolean | 'indeterminate') {
    updateMapUserSettings(map_user_settings.value, {
        route_allow_eol: value === true,
    });
}

function handleToggleMass(value: string) {
    updateMapUserSettings(map_user_settings.value, {
        route_allow_mass_status: value as TMassStatus,
    });
}

function handleToggleEveScout(value: boolean | 'indeterminate') {
    updateMapUserSettings(map_user_settings.value, {
        route_use_evescout: value === true,
    });
}
</script>

<template>
    <Popover>
        <PopoverTrigger>
            <Tooltip>
                <TooltipTrigger as-child>
                    <Button variant="secondary" size="icon">
                        <SettingsIcon />
                    </Button>
                </TooltipTrigger>
                <TooltipContent>
                    <p>Autopilot settings</p>
                </TooltipContent>
            </Tooltip>
        </PopoverTrigger>
        <PopoverContent class="w-64 p-3">
            <div class="grid auto-cols-[auto_1fr_auto] gap-x-1 gap-y-1">
                <h4 class="col-span-3 text-xs text-muted-foreground">Wormholes</h4>
                <div class="col-span-3 grid grid-cols-subgrid">
                    <Checkbox :model-value="map_user_settings.route_allow_eol" @update:model-value="handleToggleEol" id="eol-checkbox" />
                    <label for="eol-checkbox" class="cursor-pointer text-xs font-medium"> Allow EOL </label>
                    <span class="text-xs text-muted-foreground">&lt; 4 hours</span>
                </div>

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
            </div>
        </PopoverContent>
    </Popover>
</template>

<style scoped></style>
