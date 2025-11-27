<script setup lang="ts">
import { Button } from '@/components/ui/button';
import { Select, SelectContent, SelectGroup, SelectItem, SelectLabel, SelectTrigger, SelectValue } from '@/components/ui/select';
import { TUniverseRegion } from '@/types/universe-map';
import { Globe, X } from 'lucide-vue-next';
import { ref } from 'vue';

const { regions } = defineProps<{
    regions: TUniverseRegion[];
}>();

const emit = defineEmits<{
    'select-region': [regionId: number | null];
}>();

const selectedValue = ref<string>('all');

function handleValueChange(value: string) {
    selectedValue.value = value;
    const regionId = value === 'all' ? null : parseInt(value, 10);
    emit('select-region', regionId);
}

function clearSelection() {
    handleValueChange('all');
}
</script>

<template>
    <div class="flex items-center gap-2">
        <Select v-model="selectedValue" @update:model-value="(value) => handleValueChange(value as string)">
            <SelectTrigger class="w-[220px]">
                <div class="flex items-center gap-2">
                    <Globe class="h-4 w-4 shrink-0 text-muted-foreground" />
                    <SelectValue placeholder="Select a region" />
                </div>
            </SelectTrigger>
            <SelectContent>
                <SelectGroup>
                    <SelectLabel>Regions</SelectLabel>
                    <SelectItem value="all"> All Regions </SelectItem>
                    <SelectItem v-for="region in regions" :key="region.id" :value="region.id.toString()">
                        {{ region.name }}
                    </SelectItem>
                </SelectGroup>
            </SelectContent>
        </Select>

        <!-- Clear button when region is selected -->
        <Button v-if="selectedValue !== 'all'" variant="ghost" size="icon" class="h-8 w-8" @click="clearSelection" title="Clear selection">
            <X class="h-4 w-4" />
        </Button>
    </div>
</template>
