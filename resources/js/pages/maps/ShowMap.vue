<script setup lang="ts">
import MapAccessController from '@/actions/App/Http/Controllers/MapAccessController';
import MapPreferencesController from '@/actions/App/Http/Controllers/MapPreferencesController';
import MapSettingsController from '@/actions/App/Http/Controllers/MapSettingsController';
import Audits from '@/components/audits/Audits.vue';
import Autopilot from '@/components/autopilot/Autopilot.vue';
import MapCharacters from '@/components/characters/MapCharacters.vue';
import QuestionIcon from '@/components/icons/QuestionIcon.vue';
import LayoutEditor from '@/components/layout/LayoutEditor.vue';
import MapKillmails from '@/components/map-killmails/MapKillmails.vue';
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
import { useMapLayout } from '@/composables/useMapLayout';
import useHasWritePermission from '@/composables/useHasWritePermission';
import useIsMapOwner from '@/composables/useIsMapOwner';
import { useOnClient } from '@/composables/useOnClient';
import AppLayout from '@/layouts/AppLayout.vue';
import SeoHead from '@/layouts/SeoHead.vue';
import { TShowMapProps } from '@/pages/maps/index';
import { Link, router, usePage } from '@inertiajs/vue3';
import { echo } from '@laravel/echo-vue';
import { GridItem, GridLayout } from 'grid-layout-plus';
import { Settings } from 'lucide-vue-next';
import { computed, onMounted, onUnmounted } from 'vue';

const {
    map,
    selected_map_solarsystem,
    map_killmails,
    map_route_solarsystems,
    map_user_settings,
    shortest_path,
    ignored_systems,
    map_characters,
    closest_systems,
    has_guest_access,
} = defineProps<TShowMapProps>();

const character = useActiveMapCharacter();
const hasWriteAccess = useHasWritePermission();
const isOwner = useIsMapOwner();
const page = usePage();

// Initialize layout management
const layout = useMapLayout(map_user_settings);

// Helper function to get layout item props
const getLayoutItem = (id: string) => {
    return computed(() => {
        const item = layout.currentLayout.value.items.find((i) => i.i === id);
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
let isDragging = false;
let isResizing = false;

const preventSelection = (e: Event) => {
    if (isDragging || isResizing) {
        e.preventDefault();
        e.stopPropagation();
        return false;
    }
};

const disableSelection = () => {
    const style = document.documentElement.style;
    style.userSelect = 'none';
    style.webkitUserSelect = 'none';
    (style as any)['mozUserSelect'] = 'none';
    (style as any)['msUserSelect'] = 'none';
    document.body.classList.add('no-select');
};

const enableSelection = () => {
    const style = document.documentElement.style;
    style.userSelect = '';
    style.webkitUserSelect = '';
    (style as any)['mozUserSelect'] = '';
    (style as any)['msUserSelect'] = '';
    document.body.classList.remove('no-select');
};

const handleDragStart = () => {
    isDragging = true;
    disableSelection();
};

const handleDragEnd = () => {
    isDragging = false;
    enableSelection();
};

const handleResizeStart = () => {
    isResizing = true;
    disableSelection();
    document.body.style.cursor = 'se-resize';
};

const handleResizeEnd = () => {
    isResizing = false;
    enableSelection();
    document.body.style.cursor = '';
};

onMounted(() => {
    // Aggressive prevention of all selection events
    const forcePreventSelection = (e: Event) => {
        if (layout.isEditMode.value) {
            e.preventDefault();
            e.stopPropagation();
            e.stopImmediatePropagation();
            return false;
        }
        return true;
    };

    document.addEventListener('selectstart', forcePreventSelection, true);
    document.addEventListener('dragstart', forcePreventSelection, true);
    
    document.addEventListener('mousedown', (e) => {
        if (layout.isEditMode.value) {
            const target = e.target as HTMLElement;
            if (target.classList.contains('vue-resizable-handle') || 
                target.closest('.vue-grid-item') ||
                target.closest('.vue-grid-layout')) {
                e.preventDefault();
                disableSelection();
            }
        }
    }, true);
    
    document.addEventListener('mousemove', (e) => {
        if (layout.isEditMode.value && (isDragging || isResizing)) {
            e.preventDefault();
            if (window.getSelection) {
                window.getSelection()?.removeAllRanges();
            }
        }
    }, true);
    
    document.addEventListener('mouseup', () => {
        if (!isDragging && !isResizing) {
            enableSelection();
        }
    }, true);
});

onUnmounted(() => {
    document.removeEventListener('selectstart', preventSelection, true);
    document.removeEventListener('dragstart', preventSelection, true);
    enableSelection();
});
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

        <div class="space-y-4 p-4">
            <!-- Layout Controls -->
            <div class="flex items-center justify-between">
                <div class="flex-1"></div>
                <LayoutEditor :layout="layout" />
            </div>

            <!-- Grid Layout Container -->
            <GridLayout
                v-model:layout="layout.currentLayout.value.items"
                :col-num="layout.currentLayout.value.cols"
                :row-height="layout.currentLayout.value.rowHeight"
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
                <GridItem v-bind="getLayoutItem('map').value">
                    <MapComponent :map :config />
                    <MapSearch :map :search :solarsystems v-if="hasWriteAccess" />
                    <div class="absolute top-4 right-4 z-40 flex gap-2">
                        <LocationVisibility :map_user_settings="map_user_settings" :key="character?.id" v-if="hasWriteAccess" />
                        <Tracker :character :map :key="character?.id" v-if="hasWriteAccess" />
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
                <GridItem v-bind="getLayoutItem('solarsystem').value">
                    <div class="h-full overflow-auto rounded-lg border bg-card">
                        <SolarsystemDetails
                            v-if="selected_map_solarsystem"
                            :map_solarsystem="selected_map_solarsystem"
                            :map
                            :map_route_solarsystems
                            :hide_notes="has_guest_access"
                        />
                        <div class="flex h-full flex-col items-center justify-center gap-8 p-8 text-neutral-700" v-else>
                            <QuestionIcon class="text-4xl" />
                            <p class="text-center">Select a solarsystem to see more details</p>
                        </div>
                    </div>
                </GridItem>

                <!-- Signatures Section -->
                <GridItem v-if="selected_map_solarsystem" v-bind="getLayoutItem('signatures').value">
                    <div class="h-full overflow-auto rounded-lg border bg-card">
                        <Signatures :map :map_solarsystem="selected_map_solarsystem" />
                    </div>
                </GridItem>

                <!-- Audits Section -->
                <GridItem
                    v-if="selected_map_solarsystem && selected_map_solarsystem.audits && !has_guest_access"
                    v-bind="getLayoutItem('audits').value"
                >
                    <div class="h-full overflow-auto rounded-lg border bg-card">
                        <Audits :audits="selected_map_solarsystem?.audits" />
                    </div>
                </GridItem>

                <!-- Ship History Section -->
                <GridItem v-if="!has_guest_access" v-bind="getLayoutItem('ship-history').value">
                    <div class="h-full overflow-auto rounded-lg border bg-card">
                        <ShipHistory />
                    </div>
                </GridItem>

                <!-- Map Characters Section -->
                <GridItem v-if="map_characters" v-bind="getLayoutItem('characters').value">
                    <div class="h-full overflow-auto rounded-lg border bg-card">
                        <MapCharacters :map_characters />
                    </div>
                </GridItem>

                <!-- Killmails Section -->
                <GridItem v-bind="getLayoutItem('killmails').value">
                    <div class="h-full overflow-auto rounded-lg border bg-card">
                        <MapKillmails :map_killmails="map_killmails" :map_id="map.id" :map_user_settings="map_user_settings" />
                    </div>
                </GridItem>

                <!-- Autopilot Section -->
                <GridItem v-bind="getLayoutItem('autopilot').value">
                    <div class="h-full overflow-auto rounded-lg border bg-card">
                        <Autopilot
                            :map_route_solarsystems
                            :map
                            :solarsystems
                            :selected_map_solarsystem
                            :map_characters="map_characters"
                            :map_user_settings
                            :shortest_path
                            :ignored_systems
                            :closest_systems
                        />
                    </div>
                </GridItem>
            </GridLayout>
        </div>
    </AppLayout>
</template>

<style>
/* Global no-select class */
body.no-select,
body.no-select * {
    user-select: none !important;
    -webkit-user-select: none !important;
    -moz-user-select: none !important;
    -ms-user-select: none !important;
}
</style>

<style scoped>
/* Grid Layout Base Styles */
:deep(.vue-grid-layout) {
    position: relative;
    transition: height 0.3s ease;
}

:deep(.vue-grid-item) {
    transition: all 0.2s ease;
    touch-action: none;
    position: absolute;
}

:deep(.vue-grid-item.cssTransforms) {
    transition-property: transform, opacity;
}

:deep(.vue-grid-item.resizing) {
    transition: none;
    z-index: 100;
    will-change: width, height;
}

:deep(.vue-grid-item.vue-draggable-dragging) {
    transition: none;
    z-index: 200;
    will-change: transform;
    opacity: 0.8;
}

:deep(.vue-grid-item.vue-grid-placeholder) {
    background: hsl(var(--primary) / 0.2);
    border: 2px dashed hsl(var(--primary));
    border-radius: 0.5rem;
    opacity: 0.7;
    transition-duration: 0.1s;
    z-index: 2;
}

:deep(.vue-grid-item > .vue-resizable-handle) {
    position: absolute;
    width: 20px;
    height: 20px;
    cursor: se-resize;
}

:deep(.vue-grid-item > .vue-resizable-handle::after) {
    content: '';
    position: absolute;
    right: 3px;
    bottom: 3px;
    width: 8px;
    height: 8px;
    border-right: 3px solid hsl(var(--primary));
    border-bottom: 3px solid hsl(var(--primary));
    opacity: 0;
    transition: opacity 0.2s ease;
}

:deep(.vue-grid-item:hover > .vue-resizable-handle::after) {
    opacity: 0.6;
}

:deep(.vue-grid-item > .vue-resizable-handle:hover::after) {
    opacity: 1;
}

/* Grid item borders when draggable */
:deep(.vue-grid-item.vue-draggable) {
    cursor: move;
}

:deep(.vue-grid-item.vue-draggable:hover) {
    box-shadow: 0 4px 6px -1px rgb(0 0 0 / 0.1), 0 2px 4px -2px rgb(0 0 0 / 0.1);
}

:deep(.vue-grid-item.static) {
    cursor: default;
}

:deep(.vue-grid-item .no-drag) {
    cursor: auto;
}

:deep(.vue-grid-item .minMax) {
    font-size: 12px;
}

:deep(.vue-grid-item .add) {
    cursor: pointer;
}
</style>
