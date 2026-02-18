<script setup lang="ts">
import type { HTMLAttributes } from 'vue';
import { inject } from 'vue';
import MapPanelHeaderActionButton from '@/components/ui/map-panel/MapPanelHeaderActionButton.vue';
import { Tooltip, TooltipContent, TooltipTrigger } from '@/components/ui/tooltip';
import { REMOVABLE_CARD_LABELS, type RemovableCardId } from '@/const/layoutDefaults';
import { cn } from '@/lib/utils';
import { X } from 'lucide-vue-next';

const props = defineProps<{
    class?: HTMLAttributes['class'];
    cardId?: RemovableCardId;
}>();

const layoutEditMode = inject<() => boolean>('layoutEditMode', () => false);
const layoutHideCard = inject<(cardId: string) => void>('layoutHideCard');

function handleRemove() {
    if (props.cardId && layoutHideCard) {
        layoutHideCard(props.cardId);
    }
}
</script>

<template>
    <div :class="cn('flex h-9 shrink-0 items-center justify-between border-b border-border/50 bg-muted/30 px-3', props.class)">
        <h3 class="font-mono text-[10px] uppercase tracking-wider text-muted-foreground">
            <slot />
        </h3>
        <div v-if="$slots.actions || (cardId && layoutEditMode())" class="flex items-center gap-2">
            <slot name="actions" />
            <Tooltip v-if="cardId && layoutEditMode()">
                <TooltipTrigger as-child>
                    <MapPanelHeaderActionButton size="icon" variant="ghost" class="text-destructive hover:bg-destructive hover:text-destructive-foreground" @click="handleRemove">
                        <X class="size-3" />
                    </MapPanelHeaderActionButton>
                </TooltipTrigger>
                <TooltipContent side="left" class="z-[60]">
                    <p class="text-sm">Hide {{ REMOVABLE_CARD_LABELS[cardId] }}</p>
                </TooltipContent>
            </Tooltip>
        </div>
    </div>
</template>
