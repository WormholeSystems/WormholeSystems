import { updateMapUserSettings } from '@/composables/map/actions/updateMapUserSettings';
import { Breakpoint, BreakpointConfig, BREAKPOINT_OPTIONS, BREAKPOINT_WIDTHS, DEFAULT_LAYOUT_CONFIGS, LayoutConfig, LayoutConfigs, LayoutItem } from '@/types/layout';
import { TMapUserSetting } from '@/types/models';
import { useBreakpoints } from '@vueuse/core';
import { computed, ComputedRef, ref, Ref, watch } from 'vue';

export interface UseMapLayoutReturn {
    currentBreakpoint: ComputedRef<Breakpoint>;
    selectedBreakpoint: Ref<Breakpoint>;
    isEditMode: Ref<boolean>;
    currentLayout: ComputedRef<LayoutConfig>;
    breakpointOptions: BreakpointConfig[];
    updateLayout: (items: LayoutItem[]) => void;
    saveLayout: () => Promise<void>;
    resetLayout: () => void;
    toggleEditMode: () => void;
    setBreakpoint: (breakpoint: Breakpoint) => void;
}

export function useMapLayout(mapUserSettings: TMapUserSetting): UseMapLayoutReturn {
    // Initialize breakpoints with VueUse
    const breakpoints = useBreakpoints({
        sm: BREAKPOINT_WIDTHS.sm,
        md: BREAKPOINT_WIDTHS.md,
        lg: BREAKPOINT_WIDTHS.lg,
    });

    // Current breakpoint based on screen size
    const currentBreakpoint = computed<Breakpoint>(() => {
        if (breakpoints.isGreater('lg')) {
            return 'lg';
        }
        if (breakpoints.isGreater('md')) {
            return 'md';
        }
        return 'sm';
    });

    // Selected breakpoint for editing (user can manually choose which to edit)
    const selectedBreakpoint = ref<Breakpoint>(currentBreakpoint.value);
    const isEditMode = ref(false);

    // Load saved layouts or use defaults
    const layoutConfigs = ref<LayoutConfigs>(loadLayoutConfigs(mapUserSettings));

    // Current layout based on selected breakpoint (in edit mode) or current breakpoint
    const currentLayout = computed<LayoutConfig>(() => {
        const breakpoint = isEditMode.value ? selectedBreakpoint.value : currentBreakpoint.value;
        return layoutConfigs.value[breakpoint];
    });

    // Update selected breakpoint when current breakpoint changes (if not in edit mode)
    watch(currentBreakpoint, (newBreakpoint) => {
        if (!isEditMode.value) {
            selectedBreakpoint.value = newBreakpoint;
        }
    });

    // Update the current layout items
    function updateLayout(items: LayoutItem[]) {
        const breakpoint = isEditMode.value ? selectedBreakpoint.value : currentBreakpoint.value;
        layoutConfigs.value[breakpoint] = {
            ...layoutConfigs.value[breakpoint],
            items,
        };
    }

    // Save layout to backend
    async function saveLayout() {
        const data: Partial<TMapUserSetting> = {
            layout_config_sm: layoutConfigs.value.sm,
            layout_config_md: layoutConfigs.value.md,
            layout_config_lg: layoutConfigs.value.lg,
        };

        await updateMapUserSettings(mapUserSettings, data);
    }

    // Reset layout to defaults
    function resetLayout() {
        const breakpoint = isEditMode.value ? selectedBreakpoint.value : currentBreakpoint.value;
        layoutConfigs.value[breakpoint] = JSON.parse(JSON.stringify(DEFAULT_LAYOUT_CONFIGS[breakpoint]));
    }

    // Toggle edit mode
    function toggleEditMode() {
        isEditMode.value = !isEditMode.value;
        if (isEditMode.value) {
            // When entering edit mode, select the current breakpoint
            selectedBreakpoint.value = currentBreakpoint.value;
        }
    }

    // Manually set the selected breakpoint
    function setBreakpoint(breakpoint: Breakpoint) {
        selectedBreakpoint.value = breakpoint;
    }

    return {
        currentBreakpoint,
        selectedBreakpoint,
        isEditMode,
        currentLayout,
        breakpointOptions: BREAKPOINT_OPTIONS,
        updateLayout,
        saveLayout,
        resetLayout,
        toggleEditMode,
        setBreakpoint,
    };
}

// Helper function to load layout configs from map user settings
function loadLayoutConfigs(mapUserSettings: TMapUserSetting): LayoutConfigs {
    const configs: LayoutConfigs = {
        sm: mapUserSettings.layout_config_sm || JSON.parse(JSON.stringify(DEFAULT_LAYOUT_CONFIGS.sm)),
        md: mapUserSettings.layout_config_md || JSON.parse(JSON.stringify(DEFAULT_LAYOUT_CONFIGS.md)),
        lg: mapUserSettings.layout_config_lg || JSON.parse(JSON.stringify(DEFAULT_LAYOUT_CONFIGS.lg)),
    };

    // Ensure all required fields exist
    Object.keys(configs).forEach((key) => {
        const breakpoint = key as Breakpoint;
        if (!configs[breakpoint].cols) {
            configs[breakpoint].cols = DEFAULT_LAYOUT_CONFIGS[breakpoint].cols;
        }
        if (!configs[breakpoint].rowHeight) {
            configs[breakpoint].rowHeight = DEFAULT_LAYOUT_CONFIGS[breakpoint].rowHeight;
        }
        if (!configs[breakpoint].items || !Array.isArray(configs[breakpoint].items)) {
            configs[breakpoint].items = DEFAULT_LAYOUT_CONFIGS[breakpoint].items;
        }
    });

    return configs;
}

