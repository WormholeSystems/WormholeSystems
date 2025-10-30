import { updateMapUserSettings } from '@/composables/map/actions/updateMapUserSettings';
import { DEFAULT_BREAKPOINTS } from '@/const/layoutDefaults';
import { BreakpointDefinition, BreakpointsConfig, LayoutItem } from '@/types/layout';
import { TMapUserSetting } from '@/types/models';
import { useWindowSize } from '@vueuse/core';
import { computed, type ComputedRef, type MaybeRefOrGetter, nextTick, type Ref, ref, toValue, watch } from 'vue';

export interface UseMapLayoutReturn {
    // Atomic refs
    isEditMode: Ref<boolean>;
    selectedBreakpointKey: Ref<string>;
    breakpoints: Ref<BreakpointsConfig>;
    gridLayoutRef: Ref<any>;

    // Computed properties
    currentBreakpointKey: ComputedRef<string>;
    sortedBreakpoints: ComputedRef<BreakpointDefinition[]>;
    currentLayoutItems: ComputedRef<LayoutItem[]>;
    currentLayoutCols: ComputedRef<number>;
    currentLayoutRowHeight: ComputedRef<number>;

    // Methods
    updateLayout: (items: LayoutItem[]) => void;
    saveLayout: () => void;
    resetLayout: () => void;
    revertChanges: () => void;
    toggleEditMode: () => void;
    setBreakpoint: (key: string) => void;
    addBreakpoint: (breakpoint: BreakpointDefinition) => void;
    removeBreakpoint: (key: string) => void;
    refreshLayout: () => void;
}

export function useMapLayout(mapUserSettings: MaybeRefOrGetter<TMapUserSetting>): UseMapLayoutReturn {
    const { width: windowWidth } = useWindowSize();

    // Atomic refs
    const isEditMode = ref(false);
    const breakpoints = ref<BreakpointsConfig>(loadBreakpoints(toValue(mapUserSettings)));
    const gridLayoutRef = ref<any>(null);

    // Sorted breakpoints
    const sortedBreakpoints = computed(() => {
        return Object.values(breakpoints.value).sort((a, b) => a.minWidth - b.minWidth);
    });

    // Detect current breakpoint based on window width
    const currentBreakpointKey = computed(() => {
        const sorted = sortedBreakpoints.value;
        for (let i = sorted.length - 1; i >= 0; i--) {
            if (windowWidth.value >= sorted[i].minWidth) {
                return sorted[i].key;
            }
        }
        return sorted[0]?.key || 'sm';
    });

    // Initialize selected breakpoint to current one
    const selectedBreakpointKey = ref(currentBreakpointKey.value);

    // Active breakpoint (selected in edit mode, otherwise current)
    const activeBreakpointKey = computed(() => {
        return isEditMode.value ? selectedBreakpointKey.value : currentBreakpointKey.value;
    });

    // Current layout properties from active breakpoint
    const currentLayoutItems = computed(() => {
        const breakpoint = breakpoints.value[activeBreakpointKey.value];
        return breakpoint?.items || [];
    });

    const currentLayoutCols = computed(() => {
        return breakpoints.value[activeBreakpointKey.value]?.cols || 1;
    });

    const currentLayoutRowHeight = computed(() => {
        return breakpoints.value[activeBreakpointKey.value]?.rowHeight || 100;
    });

    // Watch for window resize and update selected breakpoint when not editing
    watch(
        currentBreakpointKey,
        (newKey) => {
            if (!isEditMode.value) {
                selectedBreakpointKey.value = newKey;
            }
        },
        { flush: 'post' },
    );

    function refreshLayout() {
        nextTick(() => {
            if (gridLayoutRef.value?.layoutUpdate) {
                gridLayoutRef.value.layoutUpdate();
            }
        });
    }

    function updateLayout(items: LayoutItem[]) {
        const key = activeBreakpointKey.value;
        breakpoints.value[key] = {
            ...breakpoints.value[key],
            items,
        };
    }

    function saveLayout() {
        updateMapUserSettings(
            toValue(mapUserSettings),
            {
                layout_breakpoints: breakpoints.value,
            },
            ['map_user_settings'],
        );

        isEditMode.value = false;
    }

    function resetLayout() {
        const key = activeBreakpointKey.value;
        const defaultBreakpoint = DEFAULT_BREAKPOINTS[key];
        if (defaultBreakpoint) {
            breakpoints.value[key] = structuredClone(defaultBreakpoint);
            refreshLayout();
        }
    }

    function revertChanges() {
        breakpoints.value = loadBreakpoints(toValue(mapUserSettings));
        console.log('Reverted layout changes, current breakpoints:', breakpoints.value);
        refreshLayout();
    }

    function toggleEditMode() {
        isEditMode.value = !isEditMode.value;
        if (isEditMode.value) {
            selectedBreakpointKey.value = currentBreakpointKey.value;
        }
    }

    function setBreakpoint(key: string) {
        selectedBreakpointKey.value = key;
        refreshLayout();
    }

    function addBreakpoint(breakpoint: BreakpointDefinition) {
        breakpoints.value[breakpoint.key] = breakpoint;
    }

    function removeBreakpoint(key: string) {
        if (Object.keys(breakpoints.value).length <= 1) return;

        delete breakpoints.value[key];

        if (selectedBreakpointKey.value === key) {
            selectedBreakpointKey.value = sortedBreakpoints.value[0]?.key || 'sm';
        }
    }

    return {
        isEditMode,
        selectedBreakpointKey,
        breakpoints,
        gridLayoutRef,
        currentBreakpointKey,
        sortedBreakpoints,
        currentLayoutItems,
        currentLayoutCols,
        currentLayoutRowHeight,
        updateLayout,
        saveLayout,
        resetLayout,
        revertChanges,
        toggleEditMode,
        setBreakpoint,
        addBreakpoint,
        removeBreakpoint,
        refreshLayout,
    };
}

// Helper function to load breakpoints from map user settings
function loadBreakpoints(mapUserSettings: TMapUserSetting) {
    // If saved breakpoints exist, use them
    if (mapUserSettings.layout_breakpoints && typeof mapUserSettings.layout_breakpoints === 'object') {
        const saved = mapUserSettings.layout_breakpoints as Record<string, any>;
        const breakpoints: BreakpointsConfig = {};

        // Load each saved breakpoint
        for (const [key, data] of Object.entries(saved)) {
            breakpoints[key] = {
                key: data.key || key,
                label: data.label || key,
                minWidth: data.minWidth || 0,
                description: data.description,
                cols: data.cols || 1,
                rowHeight: data.rowHeight || 100,
                items: Array.isArray(data.items) ? data.items : [],
            };
        }

        // Ensure we have at least the default breakpoints
        for (const [key, defaultBp] of Object.entries(DEFAULT_BREAKPOINTS)) {
            if (!breakpoints[key]) {
                breakpoints[key] = structuredClone(defaultBp);
            }
        }

        return breakpoints;
    }

    // Fallback to defaults
    return structuredClone(DEFAULT_BREAKPOINTS);
}
