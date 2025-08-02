<script setup lang="ts">
import QuestionIcon from '@/components/icons/QuestionIcon.vue';
import SolarsystemClass from '@/components/SolarsystemClass.vue';
import { Select, SelectContent, SelectGroup, SelectItem, SelectLabel, SelectTrigger, SelectValue } from '@/components/ui/select';
import { TProcessedConnection } from '@/composables/map';

defineProps<{
    selected: TProcessedConnection | null;
    unconnected_connections: TProcessedConnection[];
    connected_connections: TProcessedConnection[];
}>();

const model = defineModel<number | null>({
    required: true,
});
</script>

<template>
    <Select v-model:model-value="model">
        <SelectTrigger class="w-full overflow-hidden">
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
            <SelectItem :value="null" text-value="Unknown connection">
                <QuestionIcon />
                Unknown connection
            </SelectItem>
            <SelectGroup v-if="unconnected_connections.length > 0">
                <SelectLabel> Unconnected solarsystems</SelectLabel>
                <SelectItem
                    v-for="connection in unconnected_connections"
                    :key="connection.id"
                    :value="connection.id"
                    :text-value="connection.target.solarsystem?.name"
                >
                    <SolarsystemClass :wormhole_class="connection?.target!.class" :security="connection?.target!.solarsystem?.security" />
                    <span class="mr-auto truncate" v-if="!connection.target!.alias">{{ connection?.target!.name }}</span>
                    <span class="mr-auto truncate" v-else>
                        <span class="mr-1">{{ connection?.target!.alias }}</span>
                        <span class="text-muted-foreground">{{ connection?.target!.name }}</span>
                    </span>
                </SelectItem>
            </SelectGroup>
            <SelectGroup v-if="connected_connections.length > 0">
                <SelectLabel> Connected solarsystems</SelectLabel>
                <SelectItem
                    v-for="connection in connected_connections"
                    :key="connection.id"
                    :value="connection.id"
                    :text-value="connection.target.solarsystem?.name"
                >
                    <SolarsystemClass :wormhole_class="connection?.target!.class" :security="connection?.target!.solarsystem?.security" />
                    <span class="mr-auto truncate" v-if="!connection.target!.alias">{{ connection?.target!.name }}</span>
                    <span class="mr-auto truncate" v-else>
                        <span class="mr-1">{{ connection?.target!.alias }}</span>
                        <span class="-foreground text-muted">{{ connection?.target!.name }}</span>
                    </span>
                </SelectItem>
            </SelectGroup>
        </SelectContent>
    </Select>
</template>

<style scoped></style>
