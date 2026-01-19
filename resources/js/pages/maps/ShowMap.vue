<script setup lang="ts">
import MapAccessController from '@/actions/App/Http/Controllers/MapAccessController';
import MapPreferencesController from '@/actions/App/Http/Controllers/MapPreferencesController';
import MapSettingsController from '@/actions/App/Http/Controllers/MapSettingsController';
import Audits from '@/components/audits/Audits.vue';
import NavigationPanel from '@/components/autopilot/NavigationPanel.vue';
import MapCharacters from '@/components/characters/MapCharacters.vue';
import EveScoutConnections from '@/components/eve-scout/EveScoutConnections.vue';
import QuestionIcon from '@/components/icons/QuestionIcon.vue';
import LayoutEditor from '@/components/layout/LayoutEditor.vue';
import LayoutEditorToolbar from '@/components/layout/LayoutEditorToolbar.vue';
import MapKillmails from '@/components/map-killmails/MapKillmails.vue';
import ActiveCharacterWarning from '@/components/map/ActiveCharacterWarning.vue';
import LocationVisibility from '@/components/map/LocationVisibility.vue';
import MapComponent from '@/components/map/MapComponent.vue';
import MapIntroduction from '@/components/map/MapIntroduction.vue';
import MapSearch from '@/components/map/MapSearch.vue';
import Tracker from '@/components/map/Tracker.vue';
import ShipHistory from '@/components/ship-history/ShipHistory.vue';
import Signatures from '@/components/signatures/Signatures.vue';
import SolarsystemDetails from '@/components/solarsystem/SolarsystemDetails.vue';
import { Button } from '@/components/ui/button';
import { Tooltip, TooltipContent, TooltipTrigger } from '@/components/ui/tooltip';
import { useActiveMapCharacter } from '@/composables/useActiveMapCharacter';
import { useDisableTextSelection } from '@/composables/useDisableTextSelection';
import useHasWritePermission from '@/composables/useHasWritePermission';
import useIsMapOwner from '@/composables/useIsMapOwner';
import { useMapLayout } from '@/composables/useMapLayout';
import { useOnClient } from '@/composables/useOnClient';
import { useStaticSolarsystems } from '@/composables/useStaticSolarsystems';
import AppLayout from '@/layouts/AppLayout.vue';
import SeoHead from '@/layouts/SeoHead.vue';
import { TMap, TResolvedMapNavigation, TResolvedSelectedMapSolarsystem, TShowMapProps } from '@/pages/maps/index';
import { Link, router, usePage } from '@inertiajs/vue3';
import { echo } from '@laravel/echo-vue';
import { GridItem, GridLayout } from 'grid-layout-plus';
import { Settings } from 'lucide-vue-next';
import { computed, ref } from 'vue';

const {
    map,
    selected_map_solarsystem,
    map_killmails,

    map_navigation,
    map_user_settings,
    ignored_systems,
    map_characters,
    has_guest_access,
    active_character_has_access,
    eve_scout_connections,
} = defineProps<TShowMapProps>();

const character = useActiveMapCharacter();
const hasWriteAccess = useHasWritePermission();
const isOwner = useIsMapOwner();
const page = usePage();

const { resolveSolarsystem } = useStaticSolarsystems();

// Show warning when the active character is not on the map's access list
const showActiveCharacterWarning = computed(() => !active_character_has_access);

const resolvedMap = computed<TMap>(() => {
    const solarsystems = map.map_solarsystems?.map((mapSolarsystem) => {
        const resolvedSolarsystem = resolveSolarsystem(mapSolarsystem.solarsystem_id);

        return {
            ...mapSolarsystem,
            solarsystem: resolvedSolarsystem,
        };
    });

    return {
        ...map,
        map_solarsystems: solarsystems ?? map.map_solarsystems,
    };
});

const resolvedSelectedSolarsystem = computed<TResolvedSelectedMapSolarsystem | null>(() => {
    if (!selected_map_solarsystem) {
        return null;
    }

    const resolvedSolarsystem = resolveSolarsystem(selected_map_solarsystem.solarsystem_id);

    return {
        ...selected_map_solarsystem,
        solarsystem: resolvedSolarsystem,
    };
});

const resolvedMapNavigation = computed<TResolvedMapNavigation>(() => {
    const destinations = map_navigation.destinations.map((destination) => {
        const resolvedSolarsystem = resolveSolarsystem(destination.solarsystem_id);

        return {
            ...destination,
            solarsystem: resolvedSolarsystem,
        };
    });

    return {
        destinations,
    };
});

// Initialize layout management with reactive getter
const layout = useMapLayout(() => map_user_settings);

// Helper function to get layout item props
const getLayoutItem = (id: string) => {
    return computed(() => {
        const item = layout.currentLayoutItems.value.find((i) => i.i === id);
        return item || { x: 0, y: 0, w: 1, h: 1, i: id };
    });
};

// Get user scopes from auth data
const userScopes = computed(() => {
    const user = page.props.auth?.user;
    if (!user?.active_character?.esi_scopes) return [];
    return user.active_character.esi_scopes;
});

const settingsUrl = computed(() => {
    if (isOwner.value) {
        return MapSettingsController.show(map.slug).url;
    }

    if (hasWriteAccess.value) {
        return MapAccessController.show(map.slug).url;
    }

    return MapPreferencesController.show(map.slug).url;
});

useOnClient(() =>
    router.on('before', (event) => {
        const id = echo().socketId();
        if (!id) return;
        event.detail.visit.headers['X-Socket-ID'] = id;
    }),
);

// Prevent text selection during drag/resize operations
const isDragging = ref(false);
const isResizing = ref(false);

// Prevent text selection when in edit mode AND (dragging OR resizing)
const shouldPreventSelection = computed(() => {
    return layout.isEditMode.value && (isDragging.value || isResizing.value);
});

// Use text selection disabling for drag/resize operations
useDisableTextSelection(shouldPreventSelection);

const handleDragStart = () => {
    isDragging.value = true;
};

const handleDragEnd = () => {
    isDragging.value = false;
};

const handleResizeStart = () => {
    isResizing.value = true;
};

const handleResizeEnd = () => {
    isResizing.value = false;
};
</script>

<template>
    <AppLayout>
        <SeoHead
            :title="map.name"
            :description="`Explore the ${map.name} wormhole mapping network. Navigate dangerous wormhole space with real-time intel, signature tracking, and collaborative mapping tools.`"
            keywords="wormhole map, eve online navigation, wormhole signatures, space exploration, real-time intel"
        />

        <!-- Introduction Modal -->
        <MapIntroduction
            :mapUserSettings="map_user_settings"
            :userScopes="userScopes"
            :isVisible="!map_user_settings.introduction_confirmed_at && !has_guest_access"
            :mapSlug="map.slug"
        />

        <!-- Active Character Access Warning -->
        <ActiveCharacterWarning
            v-if="showActiveCharacterWarning"
            :character-name="page.props.auth?.user?.active_character?.name ?? ''"
            :has-write-access="hasWriteAccess"
            :map-slug="map.slug"
        />
        <!-- Grid Layout Container -->
        <GridLayout
            :ref="layout.gridLayoutRef"
            :layout="layout.currentLayoutItems.value"
            :col-num="layout.currentLayoutCols.value"
            :row-height="layout.currentLayoutRowHeight.value"
            :is-draggable="layout.isEditMode.value"
            :is-resizable="layout.isEditMode.value"
            :vertical-compact="true"
            :use-css-transforms="true"
            :margin="[16, 16]"
            @layout-updated="layout.updateLayout"
            @item-move="handleDragStart"
            @item-moved="handleDragEnd"
            @item-resize="handleResizeStart"
            @item-resized="handleResizeEnd"
        >
            <!-- Map Section -->
            <GridItem v-bind="getLayoutItem('map').value" @resize="handleResizeStart" @resized="handleResizeEnd">
                <MapComponent :map="resolvedMap" :config="config" />
                <MapSearch :map="resolvedMap" v-if="hasWriteAccess" />
                <div class="absolute top-4 right-4 z-40 flex gap-2">
                    <LocationVisibility :map_user_settings="map_user_settings" :key="character?.id" v-if="hasWriteAccess" />
                    <Tracker :character :map="resolvedMap" :key="character?.id" v-if="hasWriteAccess" />
                    <Tooltip>
                        <TooltipTrigger>
                            <Button variant="outline" size="icon" as-child>
                                <Link :href="settingsUrl">
                                    <Settings class="h-4 w-4" />
                                </Link>
                            </Button>
                        </TooltipTrigger>
                        <TooltipContent side="bottom">
                            <p class="text-sm">Settings</p>
                        </TooltipContent>
                    </Tooltip>
                </div>
            </GridItem>

            <!-- Solarsystem Details Section -->
            <GridItem @resize="handleResizeStart" @resized="handleResizeEnd" v-bind="getLayoutItem('solarsystem').value">
                <SolarsystemDetails
                    v-if="resolvedSelectedSolarsystem"
                    :map_solarsystem="resolvedSelectedSolarsystem"
                    :map="resolvedMap"
                    :map_navigation="resolvedMapNavigation.destinations"
                    :hide_notes="has_guest_access"
                />
                <div class="flex h-full flex-col items-center justify-center gap-8 rounded-lg border bg-card p-8 text-neutral-700" v-else>
                    <QuestionIcon class="text-4xl" />
                    <p class="text-center">Select a solarsystem to see more details</p>
                </div>
            </GridItem>

            <!-- Signatures Section -->
            <GridItem @resize="handleResizeStart" @resized="handleResizeEnd" v-bind="getLayoutItem('signatures').value">
                <Signatures :map_solarsystem="resolvedSelectedSolarsystem" />
            </GridItem>

            <!-- Audits Section -->
            <GridItem @resize="handleResizeStart" @resized="handleResizeEnd" v-if="!has_guest_access" v-bind="getLayoutItem('audits').value">
                <Audits :audits="resolvedSelectedSolarsystem?.audits ?? []" />
            </GridItem>

            <!-- Ship History Section -->
            <GridItem @resize="handleResizeStart" @resized="handleResizeEnd" v-if="!has_guest_access" v-bind="getLayoutItem('ship-history').value">
                <ShipHistory />
            </GridItem>

            <!-- Map Characters Section -->
            <GridItem @resize="handleResizeStart" @resized="handleResizeEnd" v-if="map_characters" v-bind="getLayoutItem('characters').value">
                <MapCharacters :map_characters />
            </GridItem>

            <!-- Killmails Section -->
            <GridItem @resize="handleResizeStart" @resized="handleResizeEnd" v-bind="getLayoutItem('killmails').value">
                <MapKillmails :map_killmails="map_killmails" :map_id="map.id" :map_user_settings="map_user_settings" />
            </GridItem>

            <!-- Navigation Section -->
            <GridItem @resize="handleResizeStart" @resized="handleResizeEnd" v-bind="getLayoutItem('autopilot').value">
                <NavigationPanel
                    :map_navigation="resolvedMapNavigation"
                    :map="resolvedMap"
                    :selected_map_solarsystem="resolvedSelectedSolarsystem"
                    :map_characters="map_characters"
                    :ignored_systems
                />
            </GridItem>

            <!-- EVE Scout Connections Section -->
            <GridItem @resize="handleResizeStart" @resized="handleResizeEnd" v-bind="getLayoutItem('eve-scout').value">
                <EveScoutConnections :eve_scout_connections />
            </GridItem>
        </GridLayout>
        <!-- Layout Edit Controls -->
        <LayoutEditor :layout="layout" />
        <LayoutEditorToolbar :layout="layout" />
    </AppLayout>
</template>
