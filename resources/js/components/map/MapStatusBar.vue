<script setup lang="ts">
import MapPreferencesController from '@/actions/App/Http/Controllers/MapPreferencesController';
import MapSettingsController from '@/actions/App/Http/Controllers/MapSettingsController';
import TrackingIcon from '@/components/icons/TrackingIcon.vue';
import SolarsystemClass from '@/components/solarsystem/SolarsystemClass.vue';
import { Tooltip, TooltipContent, TooltipTrigger } from '@/components/ui/tooltip';
import { updateMapUserSettings } from '@/composables/map';
import { useTracking } from '@/composables/map/composables/useTracking';
import { useActiveMapCharacter } from '@/composables/useActiveMapCharacter';
import { UseMapLayoutReturn } from '@/composables/useMapLayout';
import usePermission from '@/composables/usePermission';
import { usePing } from '@/composables/usePing';
import { useStaticSolarsystem } from '@/composables/useStaticSolarsystems';
import type { TMap } from '@/pages/maps';
import type { TMapUserSetting } from '@/types/models';
import { Link } from '@inertiajs/vue3';
import { useConnectionStatus } from '@laravel/echo-vue';
import { ConnectionStatus } from 'laravel-echo';
import { Eye, EyeOff, LayoutGrid, Map as MapIcon, Settings, Wifi, WifiOff } from 'lucide-vue-next';
import { computed, ref } from 'vue';
import MapSearch from './MapSearch.vue';
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
} = useTracking();

const targetSolarsystemName = computed(() => target_solarsystem.value?.name || currentSolarsystem.value?.name || null);

// Location visibility toggle
function handleToggleVisibility() {
    updateMapUserSettings(map.slug, {
        tracking_allowed: !map_user_settings.tracking_allowed,
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
    <div class="flex h-10 shrink-0 items-center gap-2 border-b border-border/50 bg-muted/30 px-2 sm:gap-3 sm:px-3">
        <!-- Map Name -->
        <div class="flex items-center gap-2">
            <MapIcon class="size-4 text-muted-foreground" />
            <span class="hidden font-mono text-xs font-medium sm:inline">{{ map.name }}</span>
        </div>

        <div class="hidden h-4 w-px bg-border/50 sm:block" />

        <!-- Search -->
        <div v-if="canEdit" class="hidden flex-1 sm:block">
            <MapSearch :map="map" />
        </div>
        <div v-else class="hidden flex-1 sm:block" />

        <!-- Spacer for mobile -->
        <div class="flex-1 sm:hidden" />

        <!-- Pilot Location -->
        <div v-if="character && currentSolarsystem" class="hidden items-center gap-2 lg:flex">
            <span class="text-[10px] tracking-wider text-muted-foreground uppercase">Location</span>
            <div class="flex items-center gap-1.5">
                <SolarsystemClass
                    :wormhole_class="currentSolarsystem.class"
                    :security="currentSolarsystem.security"
                    :name="currentSolarsystem.name"
                />
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

        <div class="hidden h-4 w-px bg-border/50 md:block" />

        <!-- Layout Edit Toggle -->
        <Tooltip>
            <TooltipTrigger as-child>
                <button
                    @click="layout.toggleEditMode()"
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
        @select-signature="handleSelectSignature"
    />
</template>
