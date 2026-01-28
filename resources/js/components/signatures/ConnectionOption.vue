<script setup lang="ts">
import SolarsystemClass from '@/components/solarsystem/SolarsystemClass.vue';
import { SelectItem } from '@/components/ui/select';
import { TProcessedConnection } from '@/composables/map';

defineProps<{
    connection: TProcessedConnection;
}>();
</script>

<template>
    <SelectItem :key="connection.id" :value="connection.id" :text-value="connection.target.solarsystem?.name" class="text-xs">
        <span class="flex items-center gap-1">
            <SolarsystemClass :wormhole_class="connection?.target!.solarsystem.class" :security="connection?.target!.solarsystem?.security" />
            <span class="truncate" v-if="!connection.target!.alias">{{ connection?.target!.solarsystem.name }}</span>
            <span class="truncate" v-else>{{ connection?.target!.alias }}</span>
        </span>
    </SelectItem>
</template>

<style scoped></style>
