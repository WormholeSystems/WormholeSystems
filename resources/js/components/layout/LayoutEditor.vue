<script setup lang="ts">
import { Button } from '@/components/ui/button';
import { Tooltip, TooltipContent, TooltipTrigger } from '@/components/ui/tooltip';
import { UseMapLayoutReturn } from '@/composables/useMapLayout';
import { Edit3, Save, RotateCcw, Monitor, Tablet, Smartphone } from 'lucide-vue-next';
import { computed } from 'vue';

const props = defineProps<{
    layout: UseMapLayoutReturn;
}>();

const breakpointIcons = {
    sm: Smartphone,
    md: Tablet,
    lg: Monitor,
};

const hasChanges = computed(() => {
    // Could implement change detection here if needed
    return props.layout.isEditMode.value;
});
</script>

<template>
    <div class="flex items-center gap-2">
        <!-- Edit Mode Toggle -->
        <Tooltip>
            <TooltipTrigger as-child>
                <Button
                    :variant="layout.isEditMode.value ? 'default' : 'outline'"
                    size="icon"
                    @click="layout.toggleEditMode()"
                >
                    <Edit3 class="h-4 w-4" />
                </Button>
            </TooltipTrigger>
            <TooltipContent side="bottom">
                <p class="text-sm">{{ layout.isEditMode.value ? 'Exit Edit Mode' : 'Edit Layout' }}</p>
            </TooltipContent>
        </Tooltip>

        <!-- Breakpoint Selector (shown in edit mode) -->
        <template v-if="layout.isEditMode.value">
            <div class="flex items-center gap-1 rounded-md border p-1">
                <Tooltip v-for="option in layout.breakpointOptions" :key="option.key">
                    <TooltipTrigger as-child>
                        <Button
                            :variant="layout.selectedBreakpoint.value === option.key ? 'default' : 'ghost'"
                            size="icon"
                            class="h-8 w-8"
                            @click="layout.setBreakpoint(option.key)"
                        >
                            <component :is="breakpointIcons[option.key]" class="h-4 w-4" />
                        </Button>
                    </TooltipTrigger>
                    <TooltipContent side="bottom">
                        <div class="text-sm">
                            <p class="font-semibold">{{ option.label }}</p>
                            <p class="text-xs text-muted-foreground">{{ option.description }}</p>
                        </div>
                    </TooltipContent>
                </Tooltip>
            </div>

            <!-- Save Button -->
            <Tooltip>
                <TooltipTrigger as-child>
                    <Button variant="outline" size="icon" @click="layout.saveLayout()" :disabled="!hasChanges">
                        <Save class="h-4 w-4" />
                    </Button>
                </TooltipTrigger>
                <TooltipContent side="bottom">
                    <p class="text-sm">Save Layout</p>
                </TooltipContent>
            </Tooltip>

            <!-- Reset Button -->
            <Tooltip>
                <TooltipTrigger as-child>
                    <Button variant="outline" size="icon" @click="layout.resetLayout()">
                        <RotateCcw class="h-4 w-4" />
                    </Button>
                </TooltipTrigger>
                <TooltipContent side="bottom">
                    <p class="text-sm">Reset to Default</p>
                </TooltipContent>
            </Tooltip>
        </template>

        <!-- Info Badge (shown in edit mode) -->
        <div
            v-if="layout.isEditMode.value"
            class="ml-2 rounded-md bg-blue-100 px-3 py-1 text-xs font-medium text-blue-800 dark:bg-blue-900/30 dark:text-blue-300"
        >
            Editing: {{ layout.breakpointOptions.find((o) => o.key === layout.selectedBreakpoint.value)?.label }}
        </div>
    </div>
</template>

