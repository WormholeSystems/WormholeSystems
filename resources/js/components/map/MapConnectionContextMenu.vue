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
import { deleteMapConnection, updateMapConnection } from '@/composables/map';
import { TMapConnection, TMassStatus, TShipSize } from '@/types/models';

const { map_connection } = defineProps<{
    map_connection: TMapConnection;
}>();

function handleRemoveFromMap() {
    deleteMapConnection(map_connection);
}

function handleStatusChange(mass_status: TMassStatus | string) {
    updateMapConnection(map_connection, { mass_status });
}

function handleShipSizeChange(ship_size: TShipSize | string) {
    updateMapConnection(map_connection, { ship_size });
}

function handleToggleEol() {
    updateMapConnection(map_connection, { is_eol: !map_connection.is_eol });
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
