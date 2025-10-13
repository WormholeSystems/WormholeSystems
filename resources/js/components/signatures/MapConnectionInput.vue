<script setup lang="ts">
import ConnectionOption from '@/components/signatures/ConnectionOption.vue';
import SolarsystemClass from '@/components/solarsystem/SolarsystemClass.vue';
import { Select, SelectContent, SelectGroup, SelectItem, SelectLabel, SelectSeparator, SelectTrigger, SelectValue } from '@/components/ui/select';
import { getSolarsystemClass, TProcessedConnection } from '@/composables/map';
import { TSignatureType } from '@/types/models';
import { computed, ref } from 'vue';

const { type, unconnected_connections, connected_connections } = defineProps<{
    type: TSignatureType | null;
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

function isNotFilterable(type: TSignatureType | null): type is null {
    if (type === null) return true;
    if (type.target_class === null) return true;
    return type.target_class === 'unknown';
}
</script>

<template>
    <Select v-model:model-value="model" v-model:open="open">
        <SelectTrigger class="w-full text-xs">
            <SelectValue as-child>
                <template v-if="selected">
                    <SolarsystemClass :wormhole_class="selected.target.class" :security="selected.target.solarsystem?.security" />
                    <span class="mr-auto truncate" v-if="!selected.target!.alias">{{ selected.target.name }}</span>
                    <span class="mr-auto truncate" v-else>
                        <span class="mr-1">{{ selected.target?.alias }}</span>
                        <span class="text-muted-foreground">{{ selected.target.name }}</span>
                    </span>
                </template>
                <template v-else>
                    <span class="truncate">Connection</span>
                </template>
            </SelectValue>
        </SelectTrigger>
        <SelectContent class="max-h-80">
            <template v-if="open">
                <SelectItem :value="null" text-value="Unknown connection"> Unknown connection</SelectItem>
                <SelectGroup v-if="filtered_unconnected_connections.length > 0">
                    <SelectSeparator />
                    <SelectLabel class="text-muted-foreground"> Unconnected solarsystems</SelectLabel>
                    <ConnectionOption v-for="connection in filtered_unconnected_connections" :key="connection.id" :connection="connection" />
                </SelectGroup>
                <SelectGroup v-if="filtered_connected_connections.length > 0">
                    <SelectSeparator />
                    <SelectLabel class="text-muted-foreground"> Connected solarsystems</SelectLabel>
                    <ConnectionOption v-for="connection in filtered_connected_connections" :key="connection.id" :connection="connection" />
                </SelectGroup>
            </template>
        </SelectContent>
    </Select>
</template>

<style scoped></style>
