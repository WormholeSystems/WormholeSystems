<script setup lang="ts">
import {
    ContextMenu,
    ContextMenuContent,
    ContextMenuItem,
    ContextMenuRadioGroup,
    ContextMenuRadioItem,
    ContextMenuSeparator,
    ContextMenuShortcut,
    ContextMenuSub,
    ContextMenuSubContent,
    ContextMenuSubTrigger,
    ContextMenuTrigger,
} from '@/components/ui/context-menu';
import { useMapAction } from '@/composables/map';
import { TMapSolarSystem, TMapSolarsystemStatus } from '@/types/models';

const { map_solarsystem } = defineProps<{
    map_solarsystem: TMapSolarSystem;
}>();

const { removeMapSolarsystem, updateMapSolarsystem } = useMapAction();

function handleTogglePin() {
    updateMapSolarsystem(map_solarsystem, { pinned: !map_solarsystem.pinned });
}

function handleRemoveFromMap() {
    removeMapSolarsystem(map_solarsystem);
}

function handleStatusChange(status: string) {
    updateMapSolarsystem(map_solarsystem, { status });
}

const options: TMapSolarsystemStatus[] = ['unknown', 'friendly', 'hostile', 'active', 'unscanned', 'empty'];
</script>

<template>
    <ContextMenu>
        <ContextMenuTrigger as-child>
            <slot />
        </ContextMenuTrigger>
        <ContextMenuContent>
            <ContextMenuItem @select="handleTogglePin">
                {{ map_solarsystem.pinned ? 'Unpin' : 'Pin' }}
                <ContextMenuShortcut>Ctrl+P</ContextMenuShortcut>
            </ContextMenuItem>
            <ContextMenuSub>
                <ContextMenuSubTrigger> Status</ContextMenuSubTrigger>
                <ContextMenuSubContent>
                    <ContextMenuRadioGroup :model-value="map_solarsystem.status ?? undefined" @update:model-value="handleStatusChange">
                        <ContextMenuRadioItem v-for="option in options" :key="option" :value="option">
                            {{ option.charAt(0).toUpperCase() + option.slice(1) }}
                            <span
                                :data-status="option"
                                class="ml-auto inline-block size-2 rounded-full data-[status=active]:bg-active data-[status=empty]:bg-empty data-[status=friendly]:bg-friendly data-[status=hostile]:bg-hostile data-[status=unknown]:bg-unknown data-[status=unscanned]:bg-unscanned"
                            />
                        </ContextMenuRadioItem>
                    </ContextMenuRadioGroup>
                </ContextMenuSubContent>
            </ContextMenuSub>
            <ContextMenuSeparator />
            <template v-if="!map_solarsystem.pinned">
                <ContextMenuSeparator />
                <ContextMenuItem @select="handleRemoveFromMap">
                    Remove
                    <ContextMenuShortcut>DEL</ContextMenuShortcut>
                </ContextMenuItem>
            </template>
        </ContextMenuContent>
    </ContextMenu>
</template>

<style scoped></style>
