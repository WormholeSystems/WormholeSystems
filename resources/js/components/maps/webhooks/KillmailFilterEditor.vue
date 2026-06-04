<script setup lang="ts">
import FilterValueInput from '@/components/maps/webhooks/FilterValueInput.vue';
import { Button } from '@/components/ui/button';
import { Label } from '@/components/ui/label';
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from '@/components/ui/select';
import { TKillmailFilterMode, TKillmailFilterRule, TKillmailFilterSide, TKillmailFilterSubject } from '@/types/models';
import { Plus, Trash2 } from 'lucide-vue-next';

const filters = defineModel<TKillmailFilterRule[]>('filters', { default: () => [] });

const subjectOptions: { value: TKillmailFilterSubject; label: string }[] = [
    { value: 'ship_type', label: 'Type' },
    { value: 'ship_group', label: 'Group' },
    { value: 'character', label: 'Character' },
    { value: 'corporation', label: 'Corporation' },
    { value: 'alliance', label: 'Alliance' },
];

const sideOptions: { value: TKillmailFilterSide; label: string }[] = [
    { value: 'either', label: 'Either side' },
    { value: 'victim', label: 'Victim' },
    { value: 'attacker', label: 'Attacker' },
];

const modeOptions: { value: TKillmailFilterMode; label: string }[] = [
    { value: 'include', label: 'Must match' },
    { value: 'exclude', label: 'Must not match' },
];

function patch(index: number, changes: Partial<TKillmailFilterRule>) {
    filters.value = filters.value.map((rule, i) => (i === index ? { ...rule, ...changes } : rule));
}

function changeSubject(index: number, subject: TKillmailFilterSubject) {
    // Ship ids and entity ids aren't interchangeable, so reset the values on a subject switch.
    patch(index, { subject, ids: [] });
}

function addFilter() {
    filters.value = [...filters.value, { subject: 'ship_group', side: 'either', mode: 'include', ids: [] }];
}

function removeFilter(index: number) {
    filters.value = filters.value.filter((_, i) => i !== index);
}
</script>

<template>
    <div class="space-y-3">
        <div class="flex items-center justify-between">
            <Label>Filters</Label>
            <Button type="button" variant="outline" size="sm" @click="addFilter">
                <Plus class="mr-1 h-4 w-4" />
                Add filter
            </Button>
        </div>

        <p v-if="filters.length === 0" class="text-xs text-muted-foreground">
            No filters — every killmail within range fires this alert. Add filters to narrow it down.
        </p>

        <div v-for="(filter, index) in filters" :key="index" class="space-y-3 rounded-lg border bg-muted/20 p-3">
            <div class="flex items-start gap-2">
                <div class="grid flex-1 grid-cols-1 gap-2 sm:grid-cols-3">
                    <div class="space-y-1">
                        <Label class="text-xs text-muted-foreground">Filter by</Label>
                        <Select :model-value="filter.subject" @update:model-value="(v) => changeSubject(index, v as TKillmailFilterSubject)">
                            <SelectTrigger class="w-full"><SelectValue /></SelectTrigger>
                            <SelectContent>
                                <SelectItem v-for="option in subjectOptions" :key="option.value" :value="option.value">{{ option.label }}</SelectItem>
                            </SelectContent>
                        </Select>
                    </div>
                    <div class="space-y-1">
                        <Label class="text-xs text-muted-foreground">Side</Label>
                        <Select :model-value="filter.side" @update:model-value="(v) => patch(index, { side: v as TKillmailFilterSide })">
                            <SelectTrigger class="w-full"><SelectValue /></SelectTrigger>
                            <SelectContent>
                                <SelectItem v-for="option in sideOptions" :key="option.value" :value="option.value">{{ option.label }}</SelectItem>
                            </SelectContent>
                        </Select>
                    </div>
                    <div class="space-y-1">
                        <Label class="text-xs text-muted-foreground">Rule</Label>
                        <Select :model-value="filter.mode" @update:model-value="(v) => patch(index, { mode: v as TKillmailFilterMode })">
                            <SelectTrigger class="w-full"><SelectValue /></SelectTrigger>
                            <SelectContent>
                                <SelectItem v-for="option in modeOptions" :key="option.value" :value="option.value">{{ option.label }}</SelectItem>
                            </SelectContent>
                        </Select>
                    </div>
                </div>
                <Button
                    type="button"
                    variant="ghost"
                    size="icon"
                    class="mt-5 shrink-0 text-muted-foreground hover:text-destructive"
                    @click="removeFilter(index)"
                >
                    <Trash2 class="h-4 w-4" />
                </Button>
            </div>

            <FilterValueInput :subject="filter.subject" :ids="filter.ids" @update:ids="(value) => patch(index, { ids: value })" />
        </div>
    </div>
</template>
