<script setup lang="ts">
import { Button } from '@/components/ui/button';
import { Popover, PopoverContent, PopoverTrigger } from '@/components/ui/popover';
import { useStaleConnections } from '@/composables/signatures/useStaleConnections';
import usePermission from '@/composables/usePermission';
import { Trash2, Unplug } from 'lucide-vue-next';
import { ref } from 'vue';

const { canEdit } = usePermission();
const { stale_connections, cleanMap } = useStaleConnections();

const open = ref(false);

function handleCleanMap() {
    cleanMap();
    open.value = false;
}
</script>

<template>
    <Popover v-if="canEdit && stale_connections.length > 0" v-model:open="open">
        <PopoverTrigger as-child>
            <button
                class="flex items-center gap-1.5 rounded-full border border-amber-500/40 bg-amber-500/15 px-3 py-1 text-xs font-medium text-amber-500 shadow-sm transition-colors hover:bg-amber-500/25"
            >
                <Unplug class="size-3.5 shrink-0" />
                <span class="whitespace-nowrap">
                    {{ stale_connections.length }} stale connection{{ stale_connections.length === 1 ? '' : 's' }}
                </span>
            </button>
        </PopoverTrigger>
        <PopoverContent side="bottom" align="center" class="w-80">
            <div class="flex flex-col gap-3">
                <div class="flex flex-col gap-1">
                    <p class="text-sm font-medium text-amber-500">Stale connections</p>
                    <p class="text-xs text-muted-foreground">
                        These connections have been time-critical for over an hour and have likely collapsed. Cleaning removes them and any systems
                        left disconnected from your pinned or home systems.
                    </p>
                </div>
                <ul class="flex max-h-48 flex-col gap-1 overflow-y-auto text-xs">
                    <li v-for="connection in stale_connections" :key="connection.id" class="flex items-center gap-1.5 rounded bg-muted/50 px-2 py-1">
                        <span class="truncate font-medium">{{ connection.from }}</span>
                        <span class="text-muted-foreground">→</span>
                        <span class="truncate font-medium">{{ connection.to }}</span>
                    </li>
                </ul>
                <Button variant="destructive" size="sm" class="w-full" @click="handleCleanMap">
                    <Trash2 class="size-4" />
                    Clean map ({{ stale_connections.length }})
                </Button>
            </div>
        </PopoverContent>
    </Popover>
</template>
