<script setup lang="ts">
import Audits from '@/components/audits/Audits.vue';
import NavigationPanel from '@/components/autopilot/NavigationPanel.vue';
import MapCharacters from '@/components/characters/MapCharacters.vue';
import EveScoutConnections from '@/components/eve-scout/EveScoutConnections.vue';
import LayoutEditorToolbar from '@/components/layout/LayoutEditorToolbar.vue';
import MapKillmails from '@/components/map-killmails/MapKillmails.vue';
import ActiveCharacterWarning from '@/components/map/ActiveCharacterWarning.vue';
import MapComponent from '@/components/map/MapComponent.vue';
import MapIntroduction from '@/components/map/MapIntroduction.vue';
import MapStatusBar from '@/components/map/MapStatusBar.vue';
import ShipHistory from '@/components/ship-history/ShipHistory.vue';
import Signatures from '@/components/signatures/Signatures.vue';
import SignaturesEmptyState from '@/components/signatures/SignaturesEmptyState.vue';
import SolarsystemDetails from '@/components/solarsystem/SolarsystemDetails.vue';
import SystemInfo from '@/components/solarsystem/SystemInfo.vue';
import SystemInfoEmptyState from '@/components/solarsystem/SystemInfoEmptyState.vue';
import { useDisableTextSelection } from '@/composables/useDisableTextSelection';
import useHasWritePermission from '@/composables/useHasWritePermission';
import { useMapLayout } from '@/composables/useMapLayout';
import { useOnClient } from '@/composables/useOnClient';
import { useStaticSolarsystems } from '@/composables/useStaticSolarsystems';
import AppLayout from '@/layouts/AppLayout.vue';
import SeoHead from '@/layouts/SeoHead.vue';
import { TMap, TResolvedMapNavigation, TResolvedSelectedMapSolarsystem, TShowMapProps } from '@/pages/maps/index';
import { router, usePage } from '@inertiajs/vue3';
import { echo } from '@laravel/echo-vue';
import { GridItem, GridLayout } from 'grid-layout-plus';
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

const hasWriteAccess = useHasWritePermission();
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

        <!-- Status Bar -->
        <MapStatusBar :map="resolvedMap" :map_user_settings="map_user_settings" :layout="layout" />

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
            :margin="[0, 0]"
            @layout-updated="layout.updateLayout"
            @item-move="handleDragStart"
            @item-moved="handleDragEnd"
            @item-resize="handleResizeStart"
            @item-resized="handleResizeEnd"
        >
            <!-- Map Section -->
            <GridItem v-bind="getLayoutItem('map').value" @resize="handleResizeStart" @resized="handleResizeEnd">
                <MapComponent :map="resolvedMap" :config="config" />
            </GridItem>

            <!-- System Info Section -->
            <GridItem @resize="handleResizeStart" @resized="handleResizeEnd" v-bind="getLayoutItem('system-info').value">
                <SystemInfo v-if="resolvedSelectedSolarsystem" :map_solarsystem="resolvedSelectedSolarsystem" />
                <SystemInfoEmptyState v-else />
            </GridItem>

            <!-- Notes Section -->
            <GridItem @resize="handleResizeStart" @resized="handleResizeEnd" v-if="!has_guest_access" v-bind="getLayoutItem('solarsystem').value">
                <SolarsystemDetails v-if="resolvedSelectedSolarsystem" :map_solarsystem="resolvedSelectedSolarsystem" />
                <SignaturesEmptyState v-else />
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
                <MapCharacters :map_characters :map="resolvedMap" :selected_map_solarsystem="resolvedSelectedSolarsystem" :ignored_systems />
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
        <LayoutEditorToolbar :layout="layout" />
    </AppLayout>
</template>
