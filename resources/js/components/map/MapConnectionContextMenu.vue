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
import MapConnections from '@/routes/map-connections';
import { TMapConnection, TMassStatus, TShipSize } from '@/types/models';
import { router } from '@inertiajs/vue3';

const { map_connection } = defineProps<{
    map_connection: TMapConnection;
}>();

function handleRemoveFromMap() {
    router.delete(MapConnections.destroy(map_connection.id).url, {
        preserveScroll: true,
        preserveState: true,
        only: ['map', 'map_route_solarsystems'],
    });
}

function handleStatusChange(mass_status: TMassStatus | string) {
    router.put(
        MapConnections.update(map_connection.id).url,
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
        MapConnections.update(map_connection.id).url,
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
        MapConnections.update(map_connection.id).url,
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
