<script setup lang="ts">
import BreakpointManager from '@/components/layout/BreakpointManager.vue';
import { Badge } from '@/components/ui/badge';
import { Button } from '@/components/ui/button';
import { Dialog, DialogContent, DialogDescription, DialogFooter, DialogHeader, DialogTitle } from '@/components/ui/dialog';
import { DropdownMenu, DropdownMenuContent, DropdownMenuItem, DropdownMenuSeparator, DropdownMenuTrigger } from '@/components/ui/dropdown-menu';
import { Popover, PopoverContent, PopoverTrigger } from '@/components/ui/popover';
import { Separator } from '@/components/ui/separator';
import { Tooltip, TooltipContent, TooltipTrigger } from '@/components/ui/tooltip';
import { UseMapLayoutReturn } from '@/composables/useMapLayout';
import { isProtectedBreakpoint, REMOVABLE_CARD_DESCRIPTIONS, REMOVABLE_CARD_LABELS, REMOVABLE_CARDS } from '@/const/layoutDefaults';
import {
    ClipboardCopy,
    ClipboardPaste,
    Laptop,
    LayoutGrid,
    Monitor,
    MonitorUp,
    MoreHorizontal,
    Plus,
    RotateCcw,
    Save,
    Smartphone,
    Tablet,
    X,
} from 'lucide-vue-next';
import { computed, ref } from 'vue';
import { toast } from 'vue-sonner';

const props = defineProps<{
    layout: UseMapLayoutReturn;
}>();

const hiddenCardIds = computed(() => {
    return REMOVABLE_CARDS.filter((cardId) => props.layout.isCardHidden(cardId));
});

// Pick a device icon by min width so custom breakpoints get a sensible glyph too.
function getBreakpointIcon(minWidth: number) {
    if (minWidth < 640) return Smartphone;
    if (minWidth < 1024) return Tablet;
    if (minWidth < 1536) return Laptop;
    if (minWidth < 1920) return Monitor;
    return MonitorUp;
}

const hasUnsavedChanges = computed(() => props.layout.hasUnsavedChanges.value);

const showDiscardDialog = ref(false);

function handleExitEditMode() {
    if (hasUnsavedChanges.value) {
        showDiscardDialog.value = true;
        return;
    }
    props.layout.toggleEditMode();
}

function confirmDiscard() {
    showDiscardDialog.value = false;
    props.layout.revertChanges();
    props.layout.toggleEditMode();
}

function handleSave() {
    props.layout.saveLayout();
}

function copyLayout() {
    const data = {
        breakpoints: props.layout.breakpoints.value,
        hiddenCards: props.layout.hiddenCards.value,
    };
    const encoded = btoa(JSON.stringify(data));
    navigator.clipboard.writeText(encoded);
    toast.success('Layout copied to clipboard.');
}

function pasteLayout() {
    navigator.clipboard.readText().then((text) => {
        try {
            const data = JSON.parse(atob(text.trim()));
            if (!data.breakpoints || typeof data.breakpoints !== 'object') {
                toast.error('Invalid layout data.');
                return;
            }
            props.layout.importLayout(data);
            toast.success('Layout pasted successfully.');
        } catch {
            toast.error('Invalid layout data. Make sure you copied a valid layout string.');
        }
    });
}

const canResetCurrentBreakpoint = computed(() => {
    return isProtectedBreakpoint(props.layout.selectedBreakpointKey.value);
});
</script>

<template>
    <Transition
        enter-active-class="transition duration-200 ease-out"
        enter-from-class="translate-y-4 opacity-0"
        enter-to-class="translate-y-0 opacity-100"
        leave-active-class="transition duration-150 ease-in"
        leave-from-class="translate-y-0 opacity-100"
        leave-to-class="translate-y-4 opacity-0"
    >
        <div v-if="layout.isEditMode.value" class="fixed bottom-6 left-1/2 z-50 -translate-x-1/2">
            <div class="flex items-center gap-2 rounded-2xl border border-border/60 bg-card/95 p-1.5 shadow-xl ring-1 ring-black/5 backdrop-blur-md">
                <!-- Exit -->
                <Tooltip>
                    <TooltipTrigger as-child>
                        <Button
                            variant="ghost"
                            size="icon"
                            class="size-9 rounded-xl"
                            :class="hasUnsavedChanges ? 'text-destructive hover:bg-destructive/10 hover:text-destructive' : ''"
                            @click="handleExitEditMode"
                        >
                            <X class="size-4" />
                        </Button>
                    </TooltipTrigger>
                    <TooltipContent side="top" class="z-[60]">
                        <p class="text-sm">{{ hasUnsavedChanges ? 'Exit (unsaved changes)' : 'Exit edit mode' }}</p>
                    </TooltipContent>
                </Tooltip>

                <Separator orientation="vertical" class="h-7" />

                <!-- Breakpoint selector (segmented, active tab shows its label) -->
                <div class="flex items-center gap-1.5">
                    <div class="flex items-center gap-0.5 rounded-xl border border-border/60 bg-muted/40 p-0.5">
                        <Tooltip v-for="bp in layout.sortedBreakpoints.value" :key="bp.key">
                            <TooltipTrigger as-child>
                                <button
                                    class="flex h-8 items-center gap-1.5 rounded-lg px-2 text-xs transition-colors"
                                    :class="
                                        layout.selectedBreakpointKey.value === bp.key
                                            ? 'bg-background font-medium text-foreground shadow-sm'
                                            : 'text-muted-foreground hover:text-foreground'
                                    "
                                    @click="layout.setBreakpoint(bp.key)"
                                >
                                    <component :is="getBreakpointIcon(bp.minWidth)" class="size-4 shrink-0" />
                                    <span v-if="layout.selectedBreakpointKey.value === bp.key" class="whitespace-nowrap">{{ bp.label }}</span>
                                </button>
                            </TooltipTrigger>
                            <TooltipContent side="top" class="z-[60]">
                                <p class="text-sm font-semibold">{{ bp.label }}</p>
                                <p class="text-xs text-muted-foreground">≥ {{ bp.minWidth }}px · {{ bp.cols }} cols</p>
                                <p v-if="bp.description" class="text-xs text-muted-foreground">{{ bp.description }}</p>
                            </TooltipContent>
                        </Tooltip>
                    </div>

                    <BreakpointManager :layout="layout" />
                </div>

                <Separator orientation="vertical" class="h-7" />

                <!-- Card library -->
                <Popover>
                    <PopoverTrigger as-child>
                        <Button variant="ghost" size="icon" class="relative size-9 rounded-xl">
                            <LayoutGrid class="size-4" />
                            <Badge
                                v-if="hiddenCardIds.length"
                                class="absolute -top-1 -right-1 size-4 justify-center rounded-full p-0 text-[10px] tabular-nums"
                            >
                                {{ hiddenCardIds.length }}
                            </Badge>
                        </Button>
                    </PopoverTrigger>
                    <PopoverContent side="top" align="center" class="z-[60] w-80 p-0">
                        <div class="border-b px-3 py-2.5">
                            <p class="text-sm font-medium">Cards</p>
                            <p class="text-xs text-muted-foreground">Add panels to your layout.</p>
                        </div>
                        <div class="max-h-80 space-y-0.5 overflow-y-auto p-1.5">
                            <template v-for="cardId in REMOVABLE_CARDS" :key="cardId">
                                <button
                                    v-if="layout.isCardHidden(cardId)"
                                    class="flex w-full items-start gap-3 rounded-lg p-2 text-left transition-colors hover:bg-muted"
                                    @click="layout.addCard(cardId)"
                                >
                                    <div class="min-w-0 flex-1">
                                        <p class="text-sm font-medium">{{ REMOVABLE_CARD_LABELS[cardId] }}</p>
                                        <p class="text-xs text-muted-foreground">{{ REMOVABLE_CARD_DESCRIPTIONS[cardId] }}</p>
                                    </div>
                                    <Plus class="mt-0.5 size-4 shrink-0 text-muted-foreground" />
                                </button>
                                <div v-else class="flex items-start gap-3 rounded-lg p-2 opacity-50">
                                    <div class="min-w-0 flex-1">
                                        <p class="text-sm font-medium">{{ REMOVABLE_CARD_LABELS[cardId] }}</p>
                                        <p class="text-xs text-muted-foreground">{{ REMOVABLE_CARD_DESCRIPTIONS[cardId] }}</p>
                                    </div>
                                    <span class="mt-0.5 text-[10px] font-medium tracking-wider text-muted-foreground uppercase">On layout</span>
                                </div>
                            </template>
                        </div>
                    </PopoverContent>
                </Popover>

                <Separator orientation="vertical" class="h-7" />

                <!-- Primary action + overflow -->
                <div class="flex items-center gap-1">
                    <Button class="h-9 gap-1.5 rounded-xl" :disabled="!hasUnsavedChanges" @click="handleSave">
                        <Save class="size-4" />
                        Save
                    </Button>

                    <DropdownMenu>
                        <DropdownMenuTrigger as-child>
                            <Button variant="ghost" size="icon" class="size-9 rounded-xl">
                                <MoreHorizontal class="size-4" />
                            </Button>
                        </DropdownMenuTrigger>
                        <DropdownMenuContent side="top" align="end" class="z-[60] w-48">
                            <template v-if="canResetCurrentBreakpoint">
                                <DropdownMenuItem @select="layout.resetLayout()">
                                    <RotateCcw class="size-4" />
                                    Reset to default
                                </DropdownMenuItem>
                                <DropdownMenuSeparator />
                            </template>
                            <DropdownMenuItem @select="copyLayout">
                                <ClipboardCopy class="size-4" />
                                Copy layout
                            </DropdownMenuItem>
                            <DropdownMenuItem @select="pasteLayout">
                                <ClipboardPaste class="size-4" />
                                Paste layout
                            </DropdownMenuItem>
                        </DropdownMenuContent>
                    </DropdownMenu>
                </div>
            </div>
        </div>
    </Transition>

    <Dialog v-model:open="showDiscardDialog">
        <DialogContent class="sm:max-w-md">
            <DialogHeader>
                <DialogTitle>Discard changes?</DialogTitle>
                <DialogDescription>You have unsaved layout changes. Are you sure you want to exit without saving?</DialogDescription>
            </DialogHeader>
            <DialogFooter>
                <Button variant="outline" @click="showDiscardDialog = false">Keep editing</Button>
                <Button variant="destructive" @click="confirmDiscard">Discard changes</Button>
            </DialogFooter>
        </DialogContent>
    </Dialog>
</template>
