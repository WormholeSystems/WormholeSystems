<script setup lang="ts">
import CopyConnectionNameMenu from '@/components/map/CopyConnectionNameMenu.vue';
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
import { deleteMapConnection, TProcessedConnection, updateMapConnection } from '@/composables/map';
import { formatDateToISO } from '@/lib/utils';
import { TLifetimeStatus, TMassStatus, TShipSize } from '@/types/models';
import { UTCDate } from '@date-fns/utc';

const { map_connection } = defineProps<{
    map_connection: TProcessedConnection;
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

function handleLifetimeChange(lifetime: TLifetimeStatus | string) {
    updateMapConnection(map_connection, {
        lifetime: lifetime as TLifetimeStatus,
        lifetime_updated_at: formatDateToISO(new UTCDate()),
    });
}
</script>

<template>
    <ContextMenuContent>
        <ContextMenuSub>
            <ContextMenuSubTrigger>Lifetime</ContextMenuSubTrigger>
            <ContextMenuSubContent>
                <ContextMenuRadioGroup :model-value="map_connection.lifetime_status" @update:model-value="handleLifetimeChange">
                    <ContextMenuRadioItem value="healthy" class="flex items-center justify-between gap-2">
                        <span class="flex items-center gap-2">
                            <span class="inline-block size-2 rounded-full bg-neutral-500" />
                            Healthy
                        </span>
                    </ContextMenuRadioItem>
                    <ContextMenuRadioItem value="eol" class="flex items-center justify-between gap-2">
                        <span class="flex items-center gap-2">
                            <span class="inline-block size-2 rounded-full bg-purple-500" />
                            End of Life
                        </span>
                        <span class="text-muted-foreground">&lt; 4h</span>
                    </ContextMenuRadioItem>
                    <ContextMenuRadioItem value="critical" class="flex items-center justify-between gap-2">
                        <span class="flex items-center gap-2">
                            <span class="inline-block size-2 rounded-full bg-red-500" />
                            Critical
                        </span>
                        <span class="text-muted-foreground">&lt; 1h</span>
                    </ContextMenuRadioItem>
                </ContextMenuRadioGroup>
            </ContextMenuSubContent>
        </ContextMenuSub>
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
                    <ContextMenuRadioItem value="fresh" class="flex items-center justify-between gap-2">
                        Fresh
                        <span class="text-muted-foreground">&ge; 50%</span>
                    </ContextMenuRadioItem>
                    <ContextMenuRadioItem value="reduced" class="flex items-center justify-between gap-2">
                        Reduced
                        <span class="text-muted-foreground">&lt; 50%</span>
                    </ContextMenuRadioItem>
                    <ContextMenuRadioItem value="critical" class="flex items-center justify-between gap-2">
                        Critical
                        <span class="text-muted-foreground">&le; 15%</span>
                    </ContextMenuRadioItem>
                </ContextMenuRadioGroup>
            </ContextMenuSubContent>
        </ContextMenuSub>
        <ContextMenuSeparator />
        <CopyConnectionNameMenu :map_connection="map_connection" />
        <ContextMenuSeparator />
        <ContextMenuItem @click="handleRemoveFromMap"> Remove connection</ContextMenuItem>
    </ContextMenuContent>
</template>

<style scoped></style>
