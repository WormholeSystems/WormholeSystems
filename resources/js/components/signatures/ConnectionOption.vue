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
        <span class="inline-flex items-center gap-1">
            <SolarsystemClass
                :wormhole_class="connection.target.solarsystem.class"
                :security="connection.target.solarsystem?.security"
                class="w-5 shrink-0 text-center"
            />
            <span v-if="connection.target.alias" class="shrink-0 font-medium">{{ connection.target.alias }}</span>
            <span class="truncate text-muted-foreground" :class="{ '!text-foreground': !connection.target.alias }">
                {{ connection.target.solarsystem.name }}
            </span>
            <span class="shrink-0 text-muted-foreground/60">{{ connection.target.solarsystem.region?.name }}</span>
        </span>
    </SelectItem>
</template>
