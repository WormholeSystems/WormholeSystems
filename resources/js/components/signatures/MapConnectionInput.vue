<script setup lang="ts">
import ConnectionOption from '@/components/signatures/ConnectionOption.vue';
import SolarsystemClass from '@/components/solarsystem/SolarsystemClass.vue';
import { Select, SelectContent, SelectGroup, SelectItem, SelectLabel, SelectSeparator, SelectTrigger, SelectValue } from '@/components/ui/select';
import { getSolarsystemClass, TProcessedConnection } from '@/composables/map';
import { TSignatureType } from '@/types/models';
import { computed, ref } from 'vue';

const { type, unconnected_connections, connected_connections, disabled } = defineProps<{
    type: TSignatureType | null | undefined;
    selected: TProcessedConnection | null;
    unconnected_connections: TProcessedConnection[];
    connected_connections: TProcessedConnection[];
    disabled?: boolean;
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
    <Select v-model:model-value="model" v-model:open="open" :disabled="disabled">
        <SelectTrigger class="h-6 w-full text-xs">
            <SelectValue as-child>
                <span>
                    <span v-if="selected" class="inline-flex items-center gap-1">
                        <SolarsystemClass
                            :wormhole_class="selected.target.solarsystem.class"
                            :security="selected.target.solarsystem?.security"
                            class="w-5 shrink-0 text-center"
                        />
                        <span v-if="selected.target.alias" class="shrink-0 font-medium">{{ selected.target.alias }}</span>
                        <span class="truncate text-muted-foreground" :class="{ '!text-foreground': !selected.target.alias }">
                            {{ selected.target.solarsystem.name }}
                        </span>
                        <span class="shrink-0 text-muted-foreground/60">{{ selected.target.solarsystem.region?.name }}</span>
                    </span>
                    <span v-else class="truncate text-muted-foreground">Connection</span>
                </span>
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
