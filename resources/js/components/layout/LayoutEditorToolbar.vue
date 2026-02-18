<script setup lang="ts">
import BreakpointManager from '@/components/layout/BreakpointManager.vue';
import { Button } from '@/components/ui/button';
import { Tooltip, TooltipContent, TooltipTrigger } from '@/components/ui/tooltip';
import { UseMapLayoutReturn } from '@/composables/useMapLayout';
import { isProtectedBreakpoint, REMOVABLE_CARD_LABELS, REMOVABLE_CARDS } from '@/const/layoutDefaults';
import { Laptop, Monitor, MonitorUp, Plus, RotateCcw, Save, Smartphone, Tablet, X } from 'lucide-vue-next';
import { computed, onMounted, ref, watch } from 'vue';

const props = defineProps<{
    layout: UseMapLayoutReturn;
}>();

// Track initial state to detect changes
const initialState = ref<string>('');
const initialHiddenState = ref<string>('');

onMounted(() => {
    // Store initial state as JSON for comparison
    initialState.value = JSON.stringify(props.layout.breakpoints.value);
    initialHiddenState.value = JSON.stringify(props.layout.hiddenCards.value);
});

// Watch for entering edit mode to capture initial state
watch(
    () => props.layout.isEditMode.value,
    (isEditMode) => {
        if (isEditMode) {
            // Capture state when entering edit mode
            initialState.value = JSON.stringify(props.layout.breakpoints.value);
            initialHiddenState.value = JSON.stringify(props.layout.hiddenCards.value);
        }
    },
);

// Hidden cards that can be re-added
const hiddenCardIds = computed(() => {
    return REMOVABLE_CARDS.filter((cardId) => props.layout.isCardHidden(cardId));
});

// Map breakpoint keys to icons (with fallbacks for custom breakpoints)
function getBreakpointIcon(key: string) {
    const icons: Record<string, any> = {
        sm: Smartphone,
        md: Tablet,
        lg: Monitor,
        xl: Laptop,
        '2xl': MonitorUp,
    };
    return icons[key] || Monitor;
}

const hasUnsavedChanges = computed(() => {
    if (!initialState.value) return false;
    const currentState = JSON.stringify(props.layout.breakpoints.value);
    const currentHiddenState = JSON.stringify(props.layout.hiddenCards.value);
    return initialState.value !== currentState || initialHiddenState.value !== currentHiddenState;
});

const selectedBreakpoint = computed(() => {
    return props.layout.breakpoints.value[props.layout.selectedBreakpointKey.value];
});

function handleExitEditMode() {
    if (hasUnsavedChanges.value) {
        if (!confirm('You have unsaved changes. Are you sure you want to exit without saving?')) {
            return;
        }
        // Revert changes to the last saved state
        props.layout.revertChanges();
        // Reset initial state to match reverted state
        initialState.value = JSON.stringify(props.layout.breakpoints.value);
        initialHiddenState.value = JSON.stringify(props.layout.hiddenCards.value);
    }
    props.layout.toggleEditMode();
}

function handleSave() {
    props.layout.saveLayout();
    // Update the initial state after saving
    initialState.value = JSON.stringify(props.layout.breakpoints.value);
    initialHiddenState.value = JSON.stringify(props.layout.hiddenCards.value);
}

const canResetCurrentBreakpoint = computed(() => {
    return isProtectedBreakpoint(props.layout.selectedBreakpointKey.value);
});
</script>

<template>
    <Transition
        enter-active-class="transition duration-200 ease-out"
        enter-from-class="translate-y-full opacity-0"
        enter-to-class="translate-y-0 opacity-100"
        leave-active-class="transition duration-150 ease-in"
        leave-from-class="translate-y-0 opacity-100"
        leave-to-class="translate-y-full opacity-0"
    >
        <div v-if="layout.isEditMode.value" class="fixed bottom-6 left-1/2 z-50 -translate-x-1/2 transform">
            <div class="flex items-center gap-3 rounded-xl border bg-card p-3 shadow-2xl backdrop-blur-sm">
                <!-- Exit Edit Mode Button -->
                <Tooltip>
                    <TooltipTrigger as-child>
                        <Button
                            variant="outline"
                            size="icon"
                            @click="handleExitEditMode"
                            :class="
                                hasUnsavedChanges ? 'border-destructive text-destructive hover:bg-destructive hover:text-destructive-foreground' : ''
                            "
                        >
                            <X class="h-4 w-4" />
                        </Button>
                    </TooltipTrigger>
                    <TooltipContent side="top" class="z-[60]">
                        <p class="text-sm">{{ hasUnsavedChanges ? 'Exit (Unsaved Changes)' : 'Exit Edit Mode' }}</p>
                    </TooltipContent>
                </Tooltip>

                <div class="h-8 w-px bg-border"></div>

                <!-- Breakpoint Selector -->
                <div class="flex items-center gap-1 rounded-lg border bg-muted/50 p-1">
                    <Tooltip v-for="bp in layout.sortedBreakpoints.value" :key="bp.key">
                        <TooltipTrigger as-child>
                            <Button
                                :variant="layout.selectedBreakpointKey.value === bp.key ? 'default' : 'ghost'"
                                size="icon"
                                class="h-9 w-9"
                                @click="layout.setBreakpoint(bp.key)"
                            >
                                <component :is="getBreakpointIcon(bp.key)" class="h-4 w-4" />
                            </Button>
                        </TooltipTrigger>
                        <TooltipContent side="top" class="z-[60]">
                            <div class="text-sm">
                                <p class="font-semibold">{{ bp.label }}</p>
                                <p class="text-xs text-muted-foreground">≥ {{ bp.minWidth }}px</p>
                                <p v-if="bp.description" class="text-xs text-muted-foreground">{{ bp.description }}</p>
                            </div>
                        </TooltipContent>
                    </Tooltip>
                </div>

                <!-- Breakpoint Manager -->
                <BreakpointManager :layout="layout" />

                <!-- Current Breakpoint Badge -->
                <div
                    v-if="selectedBreakpoint"
                    class="flex items-center gap-2 rounded-lg bg-blue-100 px-3 py-2 text-xs font-medium text-blue-800 dark:bg-blue-900/30 dark:text-blue-300"
                >
                    <component :is="getBreakpointIcon(selectedBreakpoint.key)" class="h-4 w-4" />
                    <span>{{ selectedBreakpoint.label }} ({{ selectedBreakpoint.minWidth }}px)</span>
                </div>

                <!-- Hidden Cards Re-add Buttons -->
                <template v-if="hiddenCardIds.length > 0">
                    <div class="h-8 w-px bg-border"></div>
                    <div class="flex items-center gap-1">
                        <Tooltip v-for="cardId in hiddenCardIds" :key="cardId">
                            <TooltipTrigger as-child>
                                <Button variant="outline" size="sm" class="h-8 gap-1 text-xs" @click="layout.showCard(cardId)">
                                    <Plus class="h-3 w-3" />
                                    {{ REMOVABLE_CARD_LABELS[cardId] }}
                                </Button>
                            </TooltipTrigger>
                            <TooltipContent side="top" class="z-[60]">
                                <p class="text-sm">Show {{ REMOVABLE_CARD_LABELS[cardId] }}</p>
                            </TooltipContent>
                        </Tooltip>
                    </div>
                </template>

                <div class="h-8 w-px bg-border"></div>

                <!-- Action Buttons -->
                <div class="flex items-center gap-1">
                    <!-- Reset Button (only for default breakpoints) -->
                    <Tooltip v-if="canResetCurrentBreakpoint">
                        <TooltipTrigger as-child>
                            <Button variant="ghost" size="icon" @click="layout.resetLayout()">
                                <RotateCcw class="h-4 w-4" />
                            </Button>
                        </TooltipTrigger>
                        <TooltipContent side="top" class="z-[60]">
                            <p class="text-sm">Reset to Default</p>
                        </TooltipContent>
                    </Tooltip>

                    <!-- Save Button -->
                    <Tooltip>
                        <TooltipTrigger as-child>
                            <Button variant="default" size="icon" @click="handleSave" :disabled="!hasUnsavedChanges">
                                <Save class="h-4 w-4" />
                            </Button>
                        </TooltipTrigger>
                        <TooltipContent side="top" class="z-[60]">
                            <p class="text-sm">{{ hasUnsavedChanges ? 'Save Changes' : 'No Changes' }}</p>
                        </TooltipContent>
                    </Tooltip>
                </div>
            </div>
        </div>
    </Transition>
</template>
