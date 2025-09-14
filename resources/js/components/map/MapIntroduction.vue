<script setup lang="ts">
import MapScopeController from '@/actions/App/Http/Controllers/MapScopeController';
import { Button } from '@/components/ui/button';
import { Dialog, DialogContent, DialogDescription, DialogHeader, DialogTitle } from '@/components/ui/dialog';
import { Switch } from '@/components/ui/switch';
import { updateMapUserSettings } from '@/composables/map';
import { TMapUserSetting } from '@/types/models';
import { useLocalStorage } from '@vueuse/core';
import { ArrowLeft, ArrowRight, CheckCircle, ExternalLink, Eye, MapPin, Route, Settings, Shield, Zap } from 'lucide-vue-next';
import { computed } from 'vue';

const props = defineProps<{
    mapUserSettings: TMapUserSetting;
    userScopes: string[];
    isVisible: boolean;
    mapSlug: string;
}>();

const totalSteps = 4;

const currentStep = useLocalStorage(`map_onboarding_step_${props.mapSlug}`, 1);

// Check which scopes the user has granted
const hasLocationScope = computed(() => props.userScopes.includes('esi-location.read_location.v1'));
const hasOnlineScope = computed(() => props.userScopes.includes('esi-location.read_online.v1'));
const hasShipScope = computed(() => props.userScopes.includes('esi-location.read_ship_type.v1'));
const hasWaypointScope = computed(() => props.userScopes.includes('esi-ui.write_waypoint.v1'));

const scopeItems = computed(() => [
    {
        scope: 'esi-location.read_location.v1',
        name: 'Character Location',
        description: 'Track your current location in real-time and show it to other map users',
        icon: MapPin,
        granted: hasLocationScope,
    },
    {
        scope: 'esi-location.read_online.v1',
        name: 'Online Status',
        description: "Show when you're online so others know you're active",
        icon: Zap,
        granted: hasOnlineScope,
    },
    {
        scope: 'esi-location.read_ship_type.v1',
        name: 'Ship Information',
        description: "Display what ship you're flying to provide intel to your team",
        icon: Shield,
        granted: hasShipScope,
    },
    {
        scope: 'esi-ui.write_waypoint.v1',
        name: 'Set Waypoints',
        description: 'Allow the map to set waypoints directly in your EVE client',
        icon: Route,
        granted: hasWaypointScope,
    },
]);

const nextStep = () => {
    if (currentStep.value < totalSteps) {
        currentStep.value++;
    }
};

const prevStep = () => {
    if (currentStep.value > 1) {
        currentStep.value--;
    }
};

function updateSetting(key: 'tracking_allowed' | 'is_tracking', value: boolean) {
    updateMapUserSettings(props.mapUserSettings, {
        [key]: value,
    });
}

function completeOnboarding() {
    updateMapUserSettings(props.mapUserSettings, {
        introduction_confirmed_at: new Date().toISOString(),
    });
}

const getScopeUrl = (scopes: string[]) => {
    return MapScopeController.show(props.mapSlug, {
        query: { scopes: scopes.join(',') },
    }).url;
};

const missingScopes = computed(() => scopeItems.value.filter((item) => !item.granted.value));

const missingScopesLink = computed(
    () =>
        MapScopeController.show(props.mapUserSettings.map_id, {
            query: { scopes: missingScopes.value.map((item) => item.scope).join(',') },
        }).url,
);

const stepTitles = ['Welcome to the Map', 'Grant Permissions', 'Configure Settings', 'Ready to Explore'];

const stepDescriptions = [
    "Let's get you set up for the best mapping experience",
    'Grant EVE Online API permissions for enhanced functionality',
    'Customize your visibility and tracking preferences',
    "You're all set! Start exploring the galaxy",
];
</script>

<template>
    <Dialog :open="isVisible">
        <DialogContent class="max-h-[90vh] w-full overflow-y-auto md:max-w-4xl">
            <DialogHeader>
                <DialogTitle class="flex items-center gap-2 text-xl">
                    <MapPin class="h-6 w-6 text-primary" />
                    {{ stepTitles[currentStep - 1] }}
                </DialogTitle>
                <DialogDescription class="text-base">
                    {{ stepDescriptions[currentStep - 1] }}
                </DialogDescription>
            </DialogHeader>

            <!-- Progress Indicator -->
            <div class="mb-6 flex items-center gap-2">
                <div v-for="step in totalSteps" :key="step" class="flex-1">
                    <div class="h-2 rounded-full transition-all duration-300" :class="step <= currentStep ? 'bg-primary' : 'bg-muted'"></div>
                </div>
                <span class="ml-2 text-sm text-muted-foreground"> {{ currentStep }} / {{ totalSteps }} </span>
            </div>

            <!-- Step Content -->
            <div class="space-y-6">
                <!-- Step 1: Welcome -->
                <div v-if="currentStep === 1" class="space-y-6">
                    <div class="space-y-4 text-center">
                        <div class="mx-auto flex h-16 w-16 items-center justify-center rounded-full bg-primary/10">
                            <MapPin class="h-8 w-8 text-primary" />
                        </div>
                        <h3 class="text-lg font-semibold">Welcome to WormholeSystems!</h3>
                        <p class="mx-auto max-w-md text-muted-foreground">
                            This quick setup will help you get the most out of your mapping experience. We'll guide you through granting permissions
                            and configuring your preferences.
                        </p>
                    </div>

                    <div class="rounded-lg bg-muted/50 p-4">
                        <h4 class="mb-2 font-medium">What we'll set up:</h4>
                        <div class="space-y-2 text-sm">
                            <div class="flex items-center gap-2">
                                <Shield class="h-4 w-4 text-primary" />
                                <span>EVE Online API permissions for real-time data</span>
                            </div>
                            <div class="flex items-center gap-2">
                                <Settings class="h-4 w-4 text-primary" />
                                <span>Your visibility and tracking preferences</span>
                            </div>
                            <div class="flex items-center gap-2">
                                <Eye class="h-4 w-4 text-primary" />
                                <span>Location sharing settings</span>
                            </div>
                        </div>
                    </div>
                </div>

                <!-- Step 2: Grant Permissions -->
                <div v-if="currentStep === 2" class="space-y-6">
                    <div class="space-y-4">
                        <p class="text-muted-foreground">These permissions enhance your mapping experience. All are optional.</p>

                        <div class="divide-y">
                            <div v-for="item in scopeItems" :key="item.scope" class="flex items-center justify-between py-6">
                                <div class="flex items-start gap-3">
                                    <component :is="item.icon" class="mt-0.5 h-5 w-5 text-muted-foreground" />
                                    <div>
                                        <h4 class="font-medium">{{ item.name }}</h4>
                                        <p class="text-sm text-muted-foreground">{{ item.description }}</p>
                                    </div>
                                </div>
                                <div class="flex items-center gap-2">
                                    <CheckCircle v-if="item.granted.value" class="h-5 w-5 text-green-600" />
                                    <Button v-else size="sm" variant="outline" class="gap-1" as-child>
                                        <a :href="getScopeUrl([item.scope])">
                                            <ExternalLink class="h-3 w-3" />
                                            Grant
                                        </a>
                                    </Button>
                                </div>
                            </div>
                        </div>
                    </div>
                </div>

                <!-- Step 3: Configure Settings -->
                <div v-if="currentStep === 3" class="space-y-6">
                    <div class="space-y-4">
                        <!-- Location Sharing -->
                        <div class="space-y-3 rounded-lg border p-4">
                            <div class="flex items-center justify-between">
                                <div class="flex items-center gap-3">
                                    <Eye class="h-5 w-5 text-muted-foreground" />
                                    <div>
                                        <h4 class="font-medium">Location Sharing</h4>
                                        <p class="text-sm text-muted-foreground">Allow other users to see your location on the map</p>
                                    </div>
                                </div>
                                <Switch
                                    :model-value="mapUserSettings.tracking_allowed"
                                    @update:modelValue="(checked: boolean) => updateSetting('tracking_allowed', Boolean(checked))"
                                />
                            </div>
                            <div class="pl-8">
                                <div class="flex items-center gap-2 text-sm">
                                    <span class="font-medium">Status:</span>
                                    <span :class="mapUserSettings.tracking_allowed ? 'text-green-600' : 'text-red-600'">
                                        {{ mapUserSettings.tracking_allowed ? 'Visible to others' : 'Hidden from others' }}
                                    </span>
                                </div>
                            </div>
                        </div>

                        <!-- Auto-tracking -->
                        <div class="space-y-3 rounded-lg border p-4">
                            <div class="flex items-center justify-between">
                                <div class="flex items-center gap-3">
                                    <Zap class="h-5 w-5 text-muted-foreground" />
                                    <div>
                                        <h4 class="font-medium">Auto-tracking</h4>
                                        <p class="text-sm text-muted-foreground">Automatically track your movement through systems</p>
                                    </div>
                                </div>
                                <Switch
                                    :modelValue="mapUserSettings.is_tracking"
                                    @update:modelValue="(checked) => updateSetting('is_tracking', Boolean(checked))"
                                />
                            </div>
                            <div class="pl-8">
                                <div class="flex items-center gap-2 text-sm">
                                    <span class="font-medium">Status:</span>
                                    <span :class="mapUserSettings.is_tracking ? 'text-green-600' : 'text-amber-600'">
                                        {{ mapUserSettings.is_tracking ? 'Active' : 'Manual only' }}
                                    </span>
                                </div>
                            </div>
                        </div>
                    </div>
                </div>

                <!-- Step 4: Ready to Explore -->
                <div v-if="currentStep === 4" class="space-y-6">
                    <div class="space-y-4 text-center">
                        <div class="mx-auto flex h-16 w-16 items-center justify-center rounded-full bg-green-100 dark:bg-green-900/20">
                            <CheckCircle class="h-8 w-8 text-green-600" />
                        </div>
                        <h3 class="text-lg font-semibold">You're All Set!</h3>
                        <p class="mx-auto max-w-md text-muted-foreground">
                            Your map is configured and ready to use. Here's a quick overview of what you can do.
                        </p>
                    </div>

                    <!-- Feature Overview -->
                    <div class="grid gap-4 md:grid-cols-2">
                        <div class="space-y-2 rounded-lg border p-4">
                            <div class="flex items-center gap-2">
                                <MapPin class="h-5 w-5 text-primary" />
                                <h4 class="font-medium">System Navigation</h4>
                            </div>
                            <p class="text-sm text-muted-foreground">Click on systems to view details, signatures, and real-time intel.</p>
                        </div>

                        <div class="space-y-2 rounded-lg border p-4">
                            <div class="flex items-center gap-2">
                                <Eye class="h-5 w-5 text-primary" />
                                <h4 class="font-medium">Visibility Controls</h4>
                            </div>
                            <p class="text-sm text-muted-foreground">Use the eye icon to toggle your visibility to other users.</p>
                        </div>

                        <div class="space-y-2 rounded-lg border p-4">
                            <div class="flex items-center gap-2">
                                <Route class="h-5 w-5 text-primary" />
                                <h4 class="font-medium">Route Planning</h4>
                            </div>
                            <p class="text-sm text-muted-foreground">Plan routes and set waypoints directly from the map interface.</p>
                        </div>

                        <div class="space-y-2 rounded-lg border p-4">
                            <div class="flex items-center gap-2">
                                <Settings class="h-5 w-5 text-primary" />
                                <h4 class="font-medium">Advanced Settings</h4>
                            </div>
                            <p class="text-sm text-muted-foreground">Access the settings menu for routing options and preferences.</p>
                        </div>
                    </div>

                    <!-- Status Summary -->
                    <div class="space-y-3 rounded-lg bg-muted/50 p-4">
                        <h4 class="font-medium">Your Current Setup:</h4>
                        <div class="grid gap-2 text-sm">
                            <div class="flex items-center justify-between">
                                <span>Permissions:</span>
                                <span :class="missingScopes.length === 0 ? 'text-green-600' : 'text-amber-600'">
                                    {{
                                        missingScopes.length === 0
                                            ? 'All granted'
                                            : `${scopeItems.length - missingScopes.length}/${scopeItems.length} granted`
                                    }}
                                </span>
                            </div>
                            <div class="flex items-center justify-between">
                                <span>Location Sharing:</span>
                                <span :class="mapUserSettings.tracking_allowed ? 'text-green-600' : 'text-red-600'">
                                    {{ mapUserSettings.tracking_allowed ? 'Enabled' : 'Disabled' }}
                                </span>
                            </div>
                            <div class="flex items-center justify-between">
                                <span>Auto-tracking:</span>
                                <span :class="mapUserSettings.is_tracking ? 'text-green-600' : 'text-amber-600'">
                                    {{ mapUserSettings.is_tracking ? 'Active' : 'Manual' }}
                                </span>
                            </div>
                        </div>
                    </div>
                </div>
            </div>

            <!-- Navigation -->
            <div class="flex items-center justify-between pt-6">
                <Button @click="prevStep" :disabled="currentStep === 1" variant="outline" class="gap-2">
                    <ArrowLeft class="h-4 w-4" />
                    Previous
                </Button>

                <div class="flex gap-2">
                    <Button v-if="currentStep < totalSteps && currentStep !== 2" @click="nextStep" class="gap-2">
                        Next
                        <ArrowRight class="h-4 w-4" />
                    </Button>
                    <template v-else-if="currentStep === 2">
                        <template v-if="missingScopes.length > 0">
                            <Button as-child class="gap-2">
                                <a :href="missingScopesLink"> Grant all </a>
                            </Button>
                            <Button @click="nextStep" variant="outline" class="gap-2">
                                Skip for now
                                <ArrowRight class="h-4 w-4" />
                            </Button>
                        </template>
                        <Button @click="nextStep" class="gap-2" v-else>
                            Next
                            <ArrowRight class="h-4 w-4" />
                        </Button>
                    </template>

                    <Button v-if="currentStep === totalSteps" @click="completeOnboarding" class="gap-2">
                        <span>Start Exploring!</span>
                        <ArrowRight class="h-4 w-4" />
                    </Button>
                </div>
            </div>
        </DialogContent>
    </Dialog>
</template>
