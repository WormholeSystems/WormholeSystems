<script setup lang="ts">
import MapAccessController from '@/actions/App/Http/Controllers/MapAccessController';
import MapPreferencesController from '@/actions/App/Http/Controllers/MapPreferencesController';
import MapSettingsController from '@/actions/App/Http/Controllers/MapSettingsController';
import TrackingIcon from '@/components/icons/TrackingIcon.vue';
import StaleConnectionsBadge from '@/components/map/StaleConnectionsBadge.vue';
import SolarsystemClass from '@/components/solarsystem/SolarsystemClass.vue';
import { Button } from '@/components/ui/button';
import { Dialog, DialogContent, DialogDescription, DialogFooter, DialogHeader, DialogTitle } from '@/components/ui/dialog';
import { Tooltip, TooltipContent, TooltipTrigger } from '@/components/ui/tooltip';
import { useTracking } from '@/composables/signatures/useTracking';
import { useActiveMapCharacter } from '@/composables/useActiveMapCharacter';
import { UseMapLayoutReturn } from '@/composables/useMapLayout';
import usePermission from '@/composables/usePermission';
import { usePing } from '@/composables/usePing';
import { useStaticSolarsystem } from '@/composables/useStaticSolarsystems';
import { updateMapUserSettings } from '@/map/api';
import type { TMap } from '@/pages/maps';
import type { TMapUserSetting } from '@/types/models';
import { Link } from '@inertiajs/vue3';
import { useConnectionStatus } from '@laravel/echo-vue';
import { ConnectionStatus } from 'laravel-echo';
import { AlertTriangle, Eye, EyeOff, LayoutGrid, Map as MapIcon, Settings, ShieldAlert, Wifi, WifiOff } from 'lucide-vue-next';
import { computed, ref } from 'vue';
import CommandPaletteButton from './CommandPaletteButton.vue';
import TrackingSignatureDialog from './TrackingSignatureDialog.vue';

const { map, map_user_settings, layout } = defineProps<{
    map: TMap;
    map_user_settings: TMapUserSetting;
    layout: UseMapLayoutReturn;
}>();

// Initialize tracking
usePing(map);

const character = useActiveMapCharacter();
const { canEdit, canManageAccess } = usePermission();

const echoStatus = typeof window !== 'undefined' ? useConnectionStatus() : ref<ConnectionStatus>('connected');
const connectionStatus = computed(() => echoStatus.value);

// Get pilot's current location
const currentSolarsystem = useStaticSolarsystem(() => character.value?.status?.solarsystem_id ?? null);

// Tracking
const {
    toggle: toggleTracking,
    is_tracking,
    is_tracking_allowed,
    can_track,
    signatures,
    show_signature_modal,
    handleSelectSignature,
    origin_map_solarsystem,
    target_solarsystem,
    suggested_alias,
} = useTracking();

const targetSolarsystemName = computed(() => target_solarsystem.value?.name || currentSolarsystem.value?.name || null);

// Confirm before leaving layout edit mode so users don't lose unsaved changes by
// clicking the toggle again instead of Save.
const showCancelLayoutDialog = ref(false);

function handleLayoutToggle() {
    if (layout.isEditMode.value && layout.hasUnsavedChanges.value) {
        showCancelLayoutDialog.value = true;
        return;
    }
    layout.toggleEditMode();
}

function saveLayoutAndExit() {
    showCancelLayoutDialog.value = false;
    layout.saveLayout();
}

function discardLayoutAndExit() {
    showCancelLayoutDialog.value = false;
    layout.revertChanges();
    layout.toggleEditMode();
}

// Location visibility toggle
function handleToggleVisibility() {
    updateMapUserSettings(map.slug, {
        tracking_allowed: !map_user_settings.tracking_allowed,
    });
}

function handleToggleThreatLevel() {
    updateMapUserSettings(map.slug, {
        show_threat_level: !map_user_settings.show_threat_level,
    });
}

const isConnected = computed(() => connectionStatus.value === 'connected');
const isConnecting = computed(() => connectionStatus.value === 'connecting' || connectionStatus.value === 'reconnecting');

const connectionStatusText = computed(() => {
    switch (connectionStatus.value) {
        case 'connected':
            return 'Connected';
        case 'connecting':
            return 'Connecting...';
        case 'reconnecting':
            return 'Reconnecting...';
        case 'disconnected':
            return 'Disconnected';
        case 'failed':
            return 'Connection Failed';
        default:
            return 'Unknown';
    }
});

const settingsUrl = computed(() => {
    if (canManageAccess.value) {
        return MapSettingsController.show(map.slug).url;
    }

    return MapPreferencesController.show(map.slug).url;
});
</script>

<template>
    <div class="relative flex h-10 shrink-0 items-center gap-2 border-b border-border/50 bg-muted/30 px-2 sm:gap-3 sm:px-3">
        <!-- Centered info badges -->
        <div class="absolute top-1/2 left-1/2 z-10 flex -translate-x-1/2 -translate-y-1/2 items-center gap-2">
            <!-- Active Character Access Warning -->
            <Tooltip v-if="$page.props.auth.user && !$page.props.active_character_has_access">
                <TooltipTrigger as-child>
                    <component
                        :is="canManageAccess ? Link : 'div'"
                        :href="canManageAccess ? MapAccessController.show(map.slug) : undefined"
                        class="flex items-center gap-1.5 rounded-full border border-yellow-500/40 bg-yellow-500/15 px-3 py-1 text-xs font-medium text-yellow-500 shadow-sm transition-colors hover:bg-yellow-500/25"
                    >
                        <AlertTriangle class="size-3.5 shrink-0 animate-pulse" />
                        <span class="whitespace-nowrap">Limited map access</span>
                    </component>
                </TooltipTrigger>
                <TooltipContent side="bottom">
                    <p class="text-xs font-medium text-yellow-500">Limited Access</p>
                    <p class="max-w-xs text-xs text-muted-foreground">
                        Your active character{{
                            $page.props.auth.user.active_character?.name ? ` ${$page.props.auth.user.active_character.name}` : ''
                        }}
                        is not on this map's access list. You are viewing through another character's access.{{
                            canManageAccess ? ' Click to manage access.' : ''
                        }}
                    </p>
                </TooltipContent>
            </Tooltip>

            <!-- Stale Connections Cleanup -->
            <StaleConnectionsBadge />
        </div>

        <!-- Map Name -->
        <div class="flex items-center gap-2">
            <MapIcon class="size-4 text-muted-foreground" />
            <span class="hidden font-mono text-xs font-medium sm:inline">{{ map.name }}</span>
        </div>

        <div class="hidden h-4 w-px bg-border/50 sm:block" />

        <!-- Search -->
        <div class="hidden flex-1 sm:block">
            <CommandPaletteButton />
        </div>

        <!-- Spacer for mobile -->
        <div class="flex-1 sm:hidden" />

        <!-- Pilot Location -->
        <div v-if="character && currentSolarsystem" class="hidden items-center gap-2 lg:flex">
            <span class="text-[10px] tracking-wider text-muted-foreground uppercase">Location</span>
            <div class="flex items-center gap-1.5">
                <SolarsystemClass :solarsystem_class="currentSolarsystem.class" :name="currentSolarsystem.name" />
                <span class="text-xs">{{ currentSolarsystem.name }}</span>
            </div>
        </div>

        <div class="hidden h-4 w-px bg-border/50 lg:block" />

        <!-- Connection Status -->
        <Tooltip>
            <TooltipTrigger as-child>
                <div class="flex items-center gap-1.5">
                    <div
                        class="size-2 rounded-full"
                        :class="{
                            'bg-green-500': isConnected,
                            'animate-pulse bg-yellow-500': isConnecting,
                            'bg-red-500': !isConnected && !isConnecting,
                        }"
                    />
                    <Wifi v-if="isConnected" class="hidden size-3.5 text-muted-foreground sm:block" />
                    <WifiOff v-else class="hidden size-3.5 text-muted-foreground sm:block" />
                </div>
            </TooltipTrigger>
            <TooltipContent side="bottom">
                <p class="text-xs">{{ connectionStatusText }}</p>
            </TooltipContent>
        </Tooltip>

        <div class="hidden h-4 w-px bg-border/50 md:block" />

        <!-- Location Visibility Toggle -->
        <Tooltip v-if="canEdit">
            <TooltipTrigger as-child>
                <button
                    @click="handleToggleVisibility"
                    class="flex items-center gap-1.5 rounded px-1.5 py-1 text-xs transition-colors sm:px-2"
                    :class="
                        map_user_settings.tracking_allowed
                            ? 'bg-green-500/20 text-green-400 hover:bg-green-500/30'
                            : 'bg-muted text-muted-foreground hover:bg-muted/80'
                    "
                >
                    <Eye v-if="map_user_settings.tracking_allowed" class="size-3.5" />
                    <EyeOff v-else class="size-3.5" />
                    <span class="hidden md:inline">Visible</span>
                </button>
            </TooltipTrigger>
            <TooltipContent side="bottom">
                <p class="text-xs font-medium">Location Monitoring</p>
                <p class="text-xs text-muted-foreground">
                    {{ map_user_settings.tracking_allowed ? 'Enabled' : 'Disabled' }} - Share location with map users
                </p>
            </TooltipContent>
        </Tooltip>

        <!-- Tracking Toggle -->
        <Tooltip v-if="canEdit">
            <TooltipTrigger as-child>
                <button
                    @click="toggleTracking"
                    :disabled="!can_track"
                    class="flex items-center gap-1.5 rounded px-1.5 py-1 text-xs transition-colors disabled:cursor-not-allowed disabled:opacity-50 sm:px-2"
                    :class="is_tracking ? 'bg-amber-500/20 text-amber-400 hover:bg-amber-500/30' : 'bg-muted text-muted-foreground hover:bg-muted/80'"
                >
                    <TrackingIcon class="size-3.5" />
                    <span class="hidden md:inline">Tracking</span>
                </button>
            </TooltipTrigger>
            <TooltipContent side="bottom">
                <template v-if="!is_tracking_allowed">
                    <p class="text-xs font-medium text-red-500">Tracking Unavailable</p>
                    <p class="text-xs text-muted-foreground">Enable location monitoring first</p>
                </template>
                <template v-else-if="!character">
                    <p class="text-xs font-medium text-red-500">Character Status Unknown</p>
                    <p class="max-w-xs text-xs text-muted-foreground">Active character must be online with scopes granted</p>
                </template>
                <template v-else>
                    <p class="text-xs font-medium">Auto-Tracking</p>
                    <p class="text-xs text-muted-foreground">{{ is_tracking ? 'Enabled' : 'Disabled' }} - Track system changes</p>
                </template>
            </TooltipContent>
        </Tooltip>

        <!-- Threat Analysis Toggle -->
        <Tooltip>
            <TooltipTrigger as-child>
                <button
                    @click="handleToggleThreatLevel"
                    class="flex items-center gap-1.5 rounded px-1.5 py-1 text-xs transition-colors sm:px-2"
                    :class="
                        map_user_settings.show_threat_level
                            ? 'bg-red-500/20 text-red-400 hover:bg-red-500/30'
                            : 'bg-muted text-muted-foreground hover:bg-muted/80'
                    "
                >
                    <ShieldAlert class="size-3.5" />
                    <span class="hidden md:inline">Threats</span>
                </button>
            </TooltipTrigger>
            <TooltipContent side="bottom">
                <p class="text-xs font-medium">Wormhole Threat Analysis</p>
                <p class="text-xs text-muted-foreground">
                    {{ map_user_settings.show_threat_level ? 'Showing' : 'Hidden' }} - Threat level rings on systems
                </p>
            </TooltipContent>
        </Tooltip>

        <div class="hidden h-4 w-px bg-border/50 md:block" />

        <!-- Layout Edit Toggle -->
        <Tooltip>
            <TooltipTrigger as-child>
                <button
                    @click="handleLayoutToggle"
                    class="flex items-center gap-1.5 rounded px-1.5 py-1 text-xs transition-colors sm:px-2"
                    :class="
                        layout.isEditMode.value
                            ? 'bg-blue-500/20 text-blue-400 hover:bg-blue-500/30'
                            : 'bg-muted text-muted-foreground hover:bg-muted/80'
                    "
                >
                    <LayoutGrid class="size-3.5" />
                    <span class="hidden md:inline">Layout</span>
                </button>
            </TooltipTrigger>
            <TooltipContent side="bottom">
                <p class="text-xs">{{ layout.isEditMode.value ? 'Exit Layout Edit Mode' : 'Edit Layout' }}</p>
            </TooltipContent>
        </Tooltip>

        <!-- Settings (authenticated users only) -->
        <Link
            v-if="$page.props.auth.user"
            :href="settingsUrl"
            class="flex items-center gap-1.5 rounded bg-muted px-1.5 py-1 text-xs text-muted-foreground transition-colors hover:bg-muted/80 hover:text-foreground sm:px-2"
            prefetch
        >
            <Settings class="size-3.5" />
            <span class="hidden md:inline">Settings</span>
        </Link>
    </div>

    <!-- Tracking Signature Dialog -->
    <TrackingSignatureDialog
        v-model:open="show_signature_modal"
        :origin-map-solarsystem="origin_map_solarsystem"
        :target-solarsystem-name="targetSolarsystemName"
        :signatures="signatures"
        :suggested-alias="suggested_alias"
        @select-signature="handleSelectSignature"
    />

    <!-- Leave Layout Editing Confirmation -->
    <Dialog v-model:open="showCancelLayoutDialog">
        <DialogContent class="sm:max-w-md">
            <DialogHeader>
                <DialogTitle>Save your layout changes?</DialogTitle>
                <DialogDescription>
                    You have unsaved layout changes. Save them, discard them and revert to your last saved layout, or keep editing.
                </DialogDescription>
            </DialogHeader>
            <DialogFooter class="gap-2 sm:justify-between">
                <Button variant="outline" @click="showCancelLayoutDialog = false">Keep editing</Button>
                <div class="flex gap-2">
                    <Button variant="destructive" @click="discardLayoutAndExit">Discard</Button>
                    <Button @click="saveLayoutAndExit">Save</Button>
                </div>
            </DialogFooter>
        </DialogContent>
    </Dialog>
</template>
