<script setup lang="ts">
import { Button } from '@/components/ui/button';
import { Minus, Plus, RotateCcw } from 'lucide-vue-next';
import { computed } from 'vue';

const emit = defineEmits<{
    'zoom-in': [];
    'zoom-out': [];
    reset: [];
}>();

const scale = defineModel<number>('scale', { required: true });

const scalePercentage = computed(() => Math.round(scale.value * 100));
</script>

<template>
    <div class="flex items-center gap-2">
        <div class="flex items-center rounded-lg border border-neutral-200 bg-white dark:border-neutral-800 dark:bg-neutral-900">
            <Button variant="ghost" size="icon" class="h-8 w-8 rounded-r-none" @click="emit('zoom-out')" :disabled="scale <= 0.02">
                <Minus class="h-4 w-4" />
            </Button>
            <div class="flex w-14 items-center justify-center border-x border-neutral-200 text-sm font-medium dark:border-neutral-800">
                {{ scalePercentage }}%
            </div>
            <Button variant="ghost" size="icon" class="h-8 w-8 rounded-l-none" @click="emit('zoom-in')" :disabled="scale >= 5">
                <Plus class="h-4 w-4" />
            </Button>
        </div>
        <Button variant="outline" size="icon" class="h-8 w-8" @click="emit('reset')" title="Reset view">
            <RotateCcw class="h-4 w-4" />
        </Button>
    </div>
</template>
