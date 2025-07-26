<script setup lang="ts">
import {
    ContextMenuContent,
    ContextMenuItem,
    ContextMenuRadioGroup,
    ContextMenuRadioItem,
    ContextMenuSeparator,
    ContextMenuSub,
    ContextMenuSubContent,
    ContextMenuSubTrigger,
} from '@/components/ui/context-menu';
import { TMapConnection, TMassStatus, TShipSize } from '@/types/models';
import { router } from '@inertiajs/vue3';

const { map_connection } = defineProps<{
    map_connection: TMapConnection;
}>();

function handleRemoveFromMap() {
    router.delete(route('map-connections.destroy', map_connection.id), {
        preserveScroll: true,
        preserveState: true,
        only: ['map', 'map_route_solarsystems'],
    });
}

function handleStatusChange(mass_status: TMassStatus | string) {
    router.put(
        route('map-connections.update', map_connection.id),
        { mass_status },
        {
            preserveScroll: true,
            preserveState: true,
            only: ['map', 'map_route_solarsystems'],
        },
    );
}

function handleShipSizeChange(ship_size: TShipSize | string) {
    router.put(
        route('map-connections.update', map_connection.id),
        { ship_size },
        {
            preserveScroll: true,
            preserveState: true,
            only: ['map', 'map_route_solarsystems'],
        },
    );
}

function handleToggleEol() {
    router.put(
        route('map-connections.update', map_connection.id),
        { is_eol: !map_connection.is_eol },
        {
            preserveScroll: true,
            preserveState: true,
            only: ['map', 'map_route_solarsystems'],
        },
    );
}
</script>

<template>
    <ContextMenuContent>
        <ContextMenuItem @click="handleToggleEol"> Toggle EOL</ContextMenuItem>
        <ContextMenuSub>
            <ContextMenuSubTrigger>Ship Size</ContextMenuSubTrigger>
            <ContextMenuSubContent>
                <ContextMenuRadioGroup :model-value="map_connection.ship_size" @update:model-value="handleShipSizeChange">
                    <ContextMenuRadioItem value="frigate"> Frigate</ContextMenuRadioItem>
                    <ContextMenuRadioItem value="medium"> Medium</ContextMenuRadioItem>
                    <ContextMenuRadioItem value="large"> Large</ContextMenuRadioItem>
                </ContextMenuRadioGroup>
            </ContextMenuSubContent>
        </ContextMenuSub>
        <ContextMenuSub>
            <ContextMenuSubTrigger> Mass Status</ContextMenuSubTrigger>
            <ContextMenuSubContent>
                <ContextMenuRadioGroup :model-value="map_connection.mass_status" @update:model-value="handleStatusChange">
                    <ContextMenuRadioItem value="fresh"> Fresh</ContextMenuRadioItem>
                    <ContextMenuRadioItem value="reduced">Reduced</ContextMenuRadioItem>
                    <ContextMenuRadioItem value="critical"> Critical</ContextMenuRadioItem>
                </ContextMenuRadioGroup>
            </ContextMenuSubContent>
        </ContextMenuSub>
        <ContextMenuSeparator />
        <ContextMenuItem @click="handleRemoveFromMap"> Remove connection</ContextMenuItem>
    </ContextMenuContent>
</template>

<style scoped></style>
