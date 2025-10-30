<script setup lang="ts">
import { Button } from '@/components/ui/button';
import {
    Dialog,
    DialogClose,
    DialogContent,
    DialogDescription,
    DialogFooter,
    DialogHeader,
    DialogTitle,
    DialogTrigger,
} from '@/components/ui/dialog';
import { Input } from '@/components/ui/input';
import { Label } from '@/components/ui/label';
import { ScrollArea } from '@/components/ui/scroll-area';
import { Tooltip, TooltipContent, TooltipTrigger } from '@/components/ui/tooltip';
import { UseMapLayoutReturn } from '@/composables/useMapLayout';
import { isProtectedBreakpoint } from '@/const/layoutDefaults';
import { BreakpointDefinition } from '@/types/layout';
import { AlertCircle, Plus, Settings2, Trash2 } from 'lucide-vue-next';
import { computed, ref, watch } from 'vue';

const props = defineProps<{
    layout: UseMapLayoutReturn;
}>();

const isOpen = ref(false);
const newBreakpointKey = ref('');
const newBreakpointLabel = ref('');
const newBreakpointWidth = ref(1920);
const newBreakpointCols = ref(12);
const newBreakpointRowHeight = ref(100);
const newBreakpointDescription = ref('');

const MIN_BREAKPOINT_WIDTH = 0;
const MIN_COLUMN_WIDTH = 160;

const maxColumns = computed(() => {
    if (newBreakpointWidth.value === 0) return 1;
    return Math.max(1, Math.floor(newBreakpointWidth.value / MIN_COLUMN_WIDTH));
});

// Auto-adjust columns when width changes to stay within limits
watch(newBreakpointWidth, (width) => {
    const max = width === 0 ? 1 : Math.max(1, Math.floor(width / MIN_COLUMN_WIDTH));
    if (newBreakpointCols.value > max) {
        newBreakpointCols.value = max;
    }
});

const validationError = computed(() => {
    const key = newBreakpointKey.value.trim();

    if (!key) return 'Key is required';
    if (props.layout.breakpoints.value[key]) return 'A breakpoint with this key already exists';
    if (!newBreakpointLabel.value.trim()) return 'Label is required';
    if (newBreakpointWidth.value < MIN_BREAKPOINT_WIDTH) return `Width must be at least ${MIN_BREAKPOINT_WIDTH}px`;
    if (newBreakpointCols.value < 1) return 'Must have at least 1 column';
    if (newBreakpointCols.value > maxColumns.value)
        return `Max ${maxColumns.value} columns for ${newBreakpointWidth.value}px width (${MIN_COLUMN_WIDTH}px per column)`;

    return null;
});

const canAddBreakpoint = computed(() => !validationError.value);

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

    // Reset form and close dialog
    resetForm();
    isOpen.value = false;
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

function removeBreakpoint(key: string) {
    if (props.layout.sortedBreakpoints.value.length <= 1) {
        return; // Don't allow removing the last breakpoint
    }
    if (confirm(`Are you sure you want to remove the "${props.layout.breakpoints.value[key].label}" breakpoint?`)) {
        props.layout.removeBreakpoint(key);
    }
}
</script>

<template>
    <Dialog v-model:open="isOpen">
        <Tooltip>
            <TooltipTrigger as-child>
                <DialogTrigger as-child>
                    <Button variant="outline" size="icon">
                        <Settings2 class="h-4 w-4" />
                    </Button>
                </DialogTrigger>
            </TooltipTrigger>
            <TooltipContent side="top" class="z-[60]">
                <p class="text-sm">Manage Breakpoints</p>
            </TooltipContent>
        </Tooltip>
        <DialogContent class="max-w-2xl">
            <DialogHeader>
                <DialogTitle>Manage Breakpoints</DialogTitle>
                <DialogDescription> Add or remove custom breakpoints to control your layout at different screen sizes. </DialogDescription>
            </DialogHeader>

            <div class="grid gap-4 py-4">
                <!-- Existing Breakpoints List -->
                <div>
                    <Label class="mb-2 block text-sm font-medium">Current Breakpoints</Label>
                    <ScrollArea class="h-[200px] rounded-md border">
                        <div class="p-4">
                            <div
                                v-for="bp in layout.sortedBreakpoints.value"
                                :key="bp.key"
                                class="mb-2 flex items-center justify-between rounded-lg border bg-card p-3 last:mb-0"
                            >
                                <div class="flex-1">
                                    <div class="flex items-center gap-2">
                                        <span class="font-semibold">{{ bp.label }}</span>
                                        <span class="rounded bg-muted px-2 py-0.5 font-mono text-xs">{{ bp.key }}</span>
                                        <span
                                            v-if="isProtectedBreakpoint(bp.key)"
                                            class="rounded bg-blue-100 px-2 py-0.5 text-xs font-medium text-blue-800 dark:bg-blue-900/30 dark:text-blue-300"
                                        >
                                            Default
                                        </span>
                                    </div>
                                    <div class="mt-1 flex items-center gap-4 text-xs text-muted-foreground">
                                        <span>≥ {{ bp.minWidth }}px</span>
                                        <span>{{ bp.cols }} cols</span>
                                        <span>{{ bp.rowHeight }}px rows</span>
                                    </div>
                                    <p v-if="bp.description" class="mt-1 text-xs text-muted-foreground">
                                        {{ bp.description }}
                                    </p>
                                </div>
                                <Button
                                    v-if="!isProtectedBreakpoint(bp.key)"
                                    variant="ghost"
                                    size="icon"
                                    class="ml-2"
                                    @click="removeBreakpoint(bp.key)"
                                >
                                    <Trash2 class="h-4 w-4 text-destructive" />
                                </Button>
                            </div>
                        </div>
                    </ScrollArea>
                </div>

                <!-- Add New Breakpoint Form -->
                <form @submit.prevent="handleSubmit" class="rounded-lg border bg-muted/50 p-4">
                    <div class="mb-3 flex items-center gap-2">
                        <Plus class="h-4 w-4" />
                        <Label class="text-sm font-medium">Add Custom Breakpoint</Label>
                    </div>

                    <div class="grid gap-4">
                        <div class="grid grid-cols-2 gap-4">
                            <div>
                                <Label for="bp-key" class="text-xs">Key (unique identifier)</Label>
                                <Input
                                    id="bp-key"
                                    v-model="newBreakpointKey"
                                    placeholder="e.g., xl, 2xl, ultrawide"
                                    pattern="[a-zA-Z0-9_-]+"
                                    maxlength="20"
                                    class="mt-1"
                                    required
                                />
                                <p class="mt-1 text-xs text-muted-foreground">Letters, numbers, dashes, underscores only</p>
                            </div>
                            <div>
                                <Label for="bp-label" class="text-xs">Label (display name)</Label>
                                <Input id="bp-label" v-model="newBreakpointLabel" placeholder="e.g., Extra Large" class="mt-1" required />
                            </div>
                        </div>

                        <div class="grid grid-cols-3 gap-4">
                            <div>
                                <Label for="bp-width" class="text-xs">Min Width (px)</Label>
                                <Input
                                    id="bp-width"
                                    v-model.number="newBreakpointWidth"
                                    type="number"
                                    :min="MIN_BREAKPOINT_WIDTH"
                                    step="1"
                                    class="mt-1"
                                    required
                                />
                                <p class="mt-1 text-xs text-muted-foreground">0 = all sizes, or set min width</p>
                            </div>
                            <div>
                                <Label for="bp-cols" class="text-xs">Columns</Label>
                                <Input
                                    id="bp-cols"
                                    v-model.number="newBreakpointCols"
                                    type="number"
                                    min="1"
                                    :max="maxColumns"
                                    class="mt-1"
                                    required
                                />
                                <p class="mt-1 text-xs text-muted-foreground">Max: {{ maxColumns }} ({{ MIN_COLUMN_WIDTH }}px/col)</p>
                            </div>
                            <div>
                                <Label for="bp-rowheight" class="text-xs">Row Height (px)</Label>
                                <Input
                                    id="bp-rowheight"
                                    v-model.number="newBreakpointRowHeight"
                                    type="number"
                                    min="50"
                                    max="500"
                                    step="10"
                                    class="mt-1"
                                    required
                                />
                                <p class="mt-1 text-xs text-muted-foreground">50-500px</p>
                            </div>
                        </div>

                        <div>
                            <Label for="bp-description" class="text-xs">Description (optional)</Label>
                            <Input id="bp-description" v-model="newBreakpointDescription" placeholder="e.g., Ultra-wide monitors" class="mt-1" />
                        </div>

                        <!-- Validation Error Message -->
                        <div
                            v-if="validationError && (newBreakpointKey || newBreakpointLabel)"
                            class="flex items-center gap-2 text-xs text-destructive"
                        >
                            <AlertCircle class="h-3 w-3" />
                            <span>{{ validationError }}</span>
                        </div>

                        <!-- Success Message -->
                        <div v-else-if="canAddBreakpoint" class="flex items-center gap-2 text-xs text-green-600 dark:text-green-400">
                            <span>✓ Ready to add breakpoint</span>
                        </div>

                        <Button type="submit" :disabled="!canAddBreakpoint" class="w-full">
                            <Plus class="mr-2 h-4 w-4" />
                            Add Breakpoint
                        </Button>
                    </div>
                </form>
            </div>

            <DialogFooter>
                <DialogClose as-child>
                    <Button variant="outline">Close</Button>
                </DialogClose>
            </DialogFooter>
        </DialogContent>
    </Dialog>
</template>
