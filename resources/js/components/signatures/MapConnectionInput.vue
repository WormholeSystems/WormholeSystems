<script setup lang="ts">
import ConnectionOption from '@/components/signatures/ConnectionOption.vue';
import SolarsystemClass from '@/components/solarsystem/SolarsystemClass.vue';
import { Select, SelectContent, SelectGroup, SelectItem, SelectLabel, SelectSeparator, SelectTrigger, SelectValue } from '@/components/ui/select';
import { getSolarsystemClass, TProcessedConnection } from '@/composables/map';
import { TSignatureType } from '@/types/models';
import { computed, ref } from 'vue';

const { type, unconnected_connections, connected_connections } = defineProps<{
    type: TSignatureType | null | undefined;
    selected: TProcessedConnection | null;
    unconnected_connections: TProcessedConnection[];
    connected_connections: TProcessedConnection[];
}>();

const model = defineModel<number | null>({
    required: true,
});

const open = ref(false);

const filtered_unconnected_connections = computed(() => {
    if (isNotFilterable(type)) {
        return unconnected_connections;
    }
    return unconnected_connections.filter((connection) => type.target_class === getSolarsystemClass(connection.target).toString());
});

const filtered_connected_connections = computed(() => {
    if (isNotFilterable(type)) {
        return connected_connections;
    }
    return connected_connections.filter((connection) => type.target_class === getSolarsystemClass(connection.target).toString());
});

function isNotFilterable(type: TSignatureType | null | undefined): type is null | undefined {
    if (!type) return true;
    if (type.target_class === null) return true;
    return type.target_class === 'unknown';
}
</script>

<template>
    <Select v-model:model-value="model" v-model:open="open">
        <SelectTrigger class="h-6 w-full text-xs">
            <SelectValue as-child>
                <template v-if="selected">
                    <span class="flex items-center gap-1">
                        <SolarsystemClass :wormhole_class="selected.target.solarsystem.class" :security="selected.target.solarsystem?.security" />
                        <span class="truncate" v-if="!selected.target!.alias">{{ selected.target.solarsystem.name }}</span>
                        <span class="truncate" v-else>{{ selected.target?.alias }}</span>
                    </span>
                </template>
                <template v-else>
                    <span class="truncate text-muted-foreground">Connection</span>
                </template>
            </SelectValue>
        </SelectTrigger>
        <SelectContent class="max-h-72">
            <template v-if="open">
                <SelectItem :value="null" text-value="Unknown" class="text-xs"> Unknown </SelectItem>
                <SelectGroup v-if="filtered_unconnected_connections.length > 0">
                    <SelectSeparator />
                    <SelectLabel class="text-xs text-muted-foreground">Connections</SelectLabel>
                    <ConnectionOption v-for="connection in filtered_unconnected_connections" :key="connection.id" :connection="connection" />
                </SelectGroup>
                <SelectGroup v-if="filtered_connected_connections.length > 0">
                    <SelectSeparator />
                    <SelectLabel class="text-xs text-muted-foreground">Connected</SelectLabel>
                    <ConnectionOption v-for="connection in filtered_connected_connections" :key="connection.id" :connection="connection" />
                </SelectGroup>
            </template>
        </SelectContent>
    </Select>
</template>

<style scoped></style>
