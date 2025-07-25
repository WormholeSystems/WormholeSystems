<script setup lang="ts">
import SolarsystemClass from '@/components/SolarsystemClass.vue';
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from '@/components/ui/select';
import { TProcessedConnection } from '@/composables/map';

defineProps<{
    selected: TProcessedConnection | null;
    outgoing_connections: TProcessedConnection[];
}>();

const model = defineModel<number | null>({
    required: true,
});
</script>

<template>
    <Select v-model:model-value="model">
        <SelectTrigger class="w-full">
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
                    <span class="truncate">Select connection</span>
                </template>
            </SelectValue>
        </SelectTrigger>
        <SelectContent>
            <SelectItem v-for="connection in outgoing_connections" :key="connection.id" :value="connection.id">
                <SolarsystemClass :wormhole_class="connection?.target!.class" :security="connection?.target!.solarsystem?.security" />
                <span class="mr-auto truncate" v-if="!connection.target!.alias">{{ connection?.target!.name }}</span>
                <span class="mr-auto truncate" v-else>
                    <span class="mr-1">{{ connection?.target!.alias }}</span>
                    <span class="text-muted-foreground">{{ connection?.target!.name }}</span>
                </span>
            </SelectItem>
        </SelectContent>
    </Select>
</template>

<style scoped></style>
