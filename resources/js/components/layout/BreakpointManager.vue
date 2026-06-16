<script setup lang="ts">
import { Badge } from '@/components/ui/badge';
import { Button } from '@/components/ui/button';
import { Dialog, DialogContent, DialogDescription, DialogHeader, DialogTitle, DialogTrigger } from '@/components/ui/dialog';
import { Input } from '@/components/ui/input';
import { Label } from '@/components/ui/label';
import { ScrollArea } from '@/components/ui/scroll-area';
import { Slider } from '@/components/ui/slider';
import { Tabs, TabsContent, TabsList, TabsTrigger } from '@/components/ui/tabs';
import { Tooltip, TooltipContent, TooltipTrigger } from '@/components/ui/tooltip';
import { UseMapLayoutReturn } from '@/composables/useMapLayout';
import { isProtectedBreakpoint } from '@/const/layoutDefaults';
import { BreakpointDefinition } from '@/types/layout';
import { AlertCircle, Check, Plus, Settings2, Trash2, X } from 'lucide-vue-next';
import { computed, ref, watch } from 'vue';

const props = defineProps<{
    layout: UseMapLayoutReturn;
}>();

const isOpen = ref(false);
const activeTab = ref('manage');
const confirmingKey = ref<string | null>(null);

const newBreakpointKey = ref('');
const newBreakpointLabel = ref('');
const newBreakpointWidth = ref(1920);
const newBreakpointCols = ref(12);
const newBreakpointRowHeight = ref(100);
const newBreakpointDescription = ref('');

const MIN_BREAKPOINT_WIDTH = 0;
const MIN_COLUMN_WIDTH = 160;
const WIDTH_PRESETS = [768, 1280, 1920, 2560, 3440];

const maxColumns = computed(() => {
    if (newBreakpointWidth.value === 0) return 1;
    return Math.max(1, Math.floor(newBreakpointWidth.value / MIN_COLUMN_WIDTH));
});

watch(newBreakpointWidth, (width) => {
    const max = width === 0 ? 1 : Math.max(1, Math.floor(width / MIN_COLUMN_WIDTH));
    if (newBreakpointCols.value > max) {
        newBreakpointCols.value = max;
    }
});

// Reset any pending delete confirmation when the dialog or tab changes.
watch([isOpen, activeTab], () => {
    confirmingKey.value = null;
});

const validationError = computed(() => {
    const key = newBreakpointKey.value.trim();

    if (!key) return 'Key is required';
    if (props.layout.breakpoints.value[key]) return 'A breakpoint with this key already exists';
    if (!newBreakpointLabel.value.trim()) return 'Label is required';
    if (newBreakpointWidth.value < MIN_BREAKPOINT_WIDTH) return `Width must be at least ${MIN_BREAKPOINT_WIDTH}px`;
    if (newBreakpointCols.value < 1) return 'Must have at least 1 column';
    if (newBreakpointCols.value > maxColumns.value) return `Max ${maxColumns.value} columns at ${newBreakpointWidth.value}px wide`;

    return null;
});

const canAddBreakpoint = computed(() => !validationError.value);

function setCols(value: number[] | undefined) {
    if (value?.length) newBreakpointCols.value = value[0];
}

function setRowHeight(value: number[] | undefined) {
    if (value?.length) newBreakpointRowHeight.value = value[0];
}

function handleSubmit() {
    if (!canAddBreakpoint.value) return;

    const key = newBreakpointKey.value.trim();
    const closestBreakpoint = findClosestBreakpoint(newBreakpointWidth.value);

    const newBreakpoint: BreakpointDefinition = {
        key,
        label: newBreakpointLabel.value.trim(),
        minWidth: newBreakpointWidth.value,
        description: newBreakpointDescription.value.trim() || undefined,
        cols: newBreakpointCols.value,
        rowHeight: newBreakpointRowHeight.value,
        items: closestBreakpoint?.items || [],
    };

    props.layout.addBreakpoint(newBreakpoint);

    resetForm();
    activeTab.value = 'manage';
}

function resetForm() {
    newBreakpointKey.value = '';
    newBreakpointLabel.value = '';
    newBreakpointWidth.value = 1920;
    newBreakpointCols.value = 12;
    newBreakpointRowHeight.value = 100;
    newBreakpointDescription.value = '';
}

function findClosestBreakpoint(targetWidth: number): BreakpointDefinition | null {
    const breakpoints = props.layout.sortedBreakpoints.value;
    let closest: BreakpointDefinition | null = null;
    let minDiff = Infinity;

    for (const bp of breakpoints) {
        const diff = Math.abs(bp.minWidth - targetWidth);
        if (diff < minDiff) {
            minDiff = diff;
            closest = bp;
        }
    }

    return closest;
}

function confirmRemove(key: string) {
    if (props.layout.sortedBreakpoints.value.length <= 1) return;
    props.layout.removeBreakpoint(key);
    confirmingKey.value = null;
}
</script>

<template>
    <Dialog v-model:open="isOpen">
        <Tooltip>
            <TooltipTrigger as-child>
                <DialogTrigger as-child>
                    <Button variant="ghost" size="icon" class="size-9 rounded-xl">
                        <Settings2 class="size-4" />
                    </Button>
                </DialogTrigger>
            </TooltipTrigger>
            <TooltipContent side="top" class="z-[60]">
                <p class="text-sm">Manage breakpoints</p>
            </TooltipContent>
        </Tooltip>

        <DialogContent class="max-w-xl gap-0 p-0">
            <DialogHeader class="border-b px-6 py-4">
                <DialogTitle>Breakpoints</DialogTitle>
                <DialogDescription>Control how your layout adapts to different screen sizes.</DialogDescription>
            </DialogHeader>

            <Tabs v-model="activeTab" class="w-full">
                <div class="px-6 pt-4">
                    <TabsList class="grid w-full grid-cols-2">
                        <TabsTrigger value="manage">Breakpoints</TabsTrigger>
                        <TabsTrigger value="add">Add custom</TabsTrigger>
                    </TabsList>
                </div>

                <!-- Manage existing -->
                <TabsContent value="manage" class="mt-0 p-6">
                    <ScrollArea class="h-[320px] pr-3">
                        <div class="flex flex-col gap-2">
                            <div
                                v-for="bp in layout.sortedBreakpoints.value"
                                :key="bp.key"
                                class="flex items-center justify-between gap-3 rounded-lg border bg-card px-3 py-2.5"
                            >
                                <div class="min-w-0">
                                    <div class="flex items-center gap-2">
                                        <span class="truncate text-sm font-medium">{{ bp.label }}</span>
                                        <Badge variant="outline" class="font-mono text-[10px]">{{ bp.key }}</Badge>
                                        <Badge v-if="isProtectedBreakpoint(bp.key)" variant="secondary" class="text-[10px]">Default</Badge>
                                    </div>
                                    <div class="mt-1.5 flex flex-wrap items-center gap-1.5 text-[11px] text-muted-foreground">
                                        <span class="rounded bg-muted px-1.5 py-0.5">≥ {{ bp.minWidth }}px</span>
                                        <span class="rounded bg-muted px-1.5 py-0.5">{{ bp.cols }} cols</span>
                                        <span class="rounded bg-muted px-1.5 py-0.5">{{ bp.rowHeight }}px rows</span>
                                    </div>
                                    <p v-if="bp.description" class="mt-1 truncate text-xs text-muted-foreground">{{ bp.description }}</p>
                                </div>

                                <div v-if="!isProtectedBreakpoint(bp.key)" class="flex shrink-0 items-center">
                                    <template v-if="confirmingKey === bp.key">
                                        <Button variant="ghost" size="icon" class="size-8 text-muted-foreground" @click="confirmingKey = null">
                                            <X class="size-4" />
                                        </Button>
                                        <Button
                                            variant="ghost"
                                            size="icon"
                                            class="size-8 text-destructive hover:bg-destructive/10"
                                            @click="confirmRemove(bp.key)"
                                        >
                                            <Check class="size-4" />
                                        </Button>
                                    </template>
                                    <Button
                                        v-else
                                        variant="ghost"
                                        size="icon"
                                        class="size-8 text-muted-foreground hover:text-destructive"
                                        @click="confirmingKey = bp.key"
                                    >
                                        <Trash2 class="size-4" />
                                    </Button>
                                </div>
                            </div>
                        </div>
                    </ScrollArea>
                </TabsContent>

                <!-- Add custom -->
                <TabsContent value="add" class="mt-0 p-6">
                    <form class="flex flex-col gap-5" @submit.prevent="handleSubmit">
                        <div class="grid grid-cols-2 gap-4">
                            <div class="flex flex-col gap-1.5">
                                <Label for="bp-key" class="text-xs text-muted-foreground">Key</Label>
                                <Input
                                    id="bp-key"
                                    v-model="newBreakpointKey"
                                    placeholder="xl, 2xl, ultrawide"
                                    pattern="[a-zA-Z0-9_-]+"
                                    maxlength="20"
                                    required
                                />
                            </div>
                            <div class="flex flex-col gap-1.5">
                                <Label for="bp-label" class="text-xs text-muted-foreground">Label</Label>
                                <Input id="bp-label" v-model="newBreakpointLabel" placeholder="Extra Large" required />
                            </div>
                        </div>

                        <div class="flex flex-col gap-2">
                            <div class="flex items-center justify-between">
                                <Label for="bp-width" class="text-xs text-muted-foreground">Min width</Label>
                                <span class="font-mono text-xs text-muted-foreground">{{ newBreakpointWidth }}px</span>
                            </div>
                            <Input id="bp-width" v-model.number="newBreakpointWidth" type="number" :min="MIN_BREAKPOINT_WIDTH" step="1" required />
                            <div class="flex flex-wrap gap-1.5">
                                <button
                                    v-for="preset in WIDTH_PRESETS"
                                    :key="preset"
                                    type="button"
                                    class="rounded-md border px-2 py-0.5 text-xs transition-colors"
                                    :class="
                                        newBreakpointWidth === preset
                                            ? 'border-primary bg-primary/10 text-primary'
                                            : 'text-muted-foreground hover:bg-muted'
                                    "
                                    @click="newBreakpointWidth = preset"
                                >
                                    {{ preset }}px
                                </button>
                            </div>
                        </div>

                        <div class="grid grid-cols-2 gap-5">
                            <div class="flex flex-col gap-2">
                                <div class="flex items-center justify-between">
                                    <Label class="text-xs text-muted-foreground">Columns</Label>
                                    <span class="font-mono text-xs">{{ newBreakpointCols }}</span>
                                </div>
                                <Slider :model-value="[newBreakpointCols]" :min="1" :max="maxColumns" :step="1" @update:model-value="setCols" />
                                <p class="text-[11px] text-muted-foreground">Up to {{ maxColumns }} ({{ MIN_COLUMN_WIDTH }}px each)</p>
                            </div>
                            <div class="flex flex-col gap-2">
                                <div class="flex items-center justify-between">
                                    <Label class="text-xs text-muted-foreground">Row height</Label>
                                    <span class="font-mono text-xs">{{ newBreakpointRowHeight }}px</span>
                                </div>
                                <Slider :model-value="[newBreakpointRowHeight]" :min="50" :max="500" :step="10" @update:model-value="setRowHeight" />
                                <p class="text-[11px] text-muted-foreground">Height of each grid row</p>
                            </div>
                        </div>

                        <div class="flex flex-col gap-1.5">
                            <Label for="bp-description" class="text-xs text-muted-foreground"
                                >Description <span class="text-muted-foreground/60">(optional)</span></Label
                            >
                            <Input id="bp-description" v-model="newBreakpointDescription" placeholder="Ultra-wide monitors" />
                        </div>

                        <div
                            v-if="validationError && (newBreakpointKey || newBreakpointLabel)"
                            class="flex items-center gap-2 text-xs text-destructive"
                        >
                            <AlertCircle class="size-3.5 shrink-0" />
                            <span>{{ validationError }}</span>
                        </div>

                        <Button type="submit" :disabled="!canAddBreakpoint" class="gap-1.5">
                            <Plus class="size-4" />
                            Add breakpoint
                        </Button>
                    </form>
                </TabsContent>
            </Tabs>
        </DialogContent>
    </Dialog>
</template>
