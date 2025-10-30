<script setup lang="ts">
import { Button } from '@/components/ui/button';
import { Tooltip, TooltipContent, TooltipTrigger } from '@/components/ui/tooltip';
import { UseMapLayoutReturn } from '@/composables/useMapLayout';
import { Settings } from 'lucide-vue-next';

defineProps<{
    layout: UseMapLayoutReturn;
}>();
</script>

<template>
    <!-- Edit Mode Toggle Button (floating in bottom right when not in edit mode) -->
    <Transition
        enter-active-class="transition duration-200 ease-out"
        enter-from-class="opacity-0 scale-95"
        enter-to-class="opacity-100 scale-100"
        leave-active-class="transition duration-150 ease-in"
        leave-from-class="opacity-100 scale-100"
        leave-to-class="opacity-0 scale-95"
    >
        <div v-if="!layout.isEditMode.value" class="fixed right-6 bottom-6 z-50">
            <Tooltip>
                <TooltipTrigger as-child>
                    <Button variant="outline" size="icon" class="h-11 w-11 shadow-lg backdrop-blur-sm" @click="layout.toggleEditMode()">
                        <Settings class="h-5 w-5" />
                    </Button>
                </TooltipTrigger>
                <TooltipContent side="left">
                    <p class="text-sm">Edit Layout</p>
                </TooltipContent>
            </Tooltip>
        </div>
    </Transition>
</template>
