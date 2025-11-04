<script setup lang="ts">
import SolarsystemClass from '@/components/solarsystem/SolarsystemClass.vue';
import { SelectItem } from '@/components/ui/select';
import { TProcessedConnection } from '@/composables/map';

defineProps<{
    connection: TProcessedConnection;
}>();
</script>

<template>
    <SelectItem :key="connection.id" :value="connection.id" :text-value="connection.target.solarsystem?.name" class="*:[span]:flex *:[span]:gap-2">
        <div class="w-[2ch] text-center">
            <SolarsystemClass :wormhole_class="connection?.target!.solarsystem.class" :security="connection?.target!.solarsystem?.security" />
        </div>
        <span class="mr-auto truncate" v-if="!connection.target!.alias">{{ connection?.target!.solarsystem.name }}</span>
        <span class="mr-auto truncate" v-else>
            <span class="mr-1">{{ connection?.target!.alias }}</span>
            <span class="-foreground text-muted-foreground">{{ connection?.target!.solarsystem.name }}</span>
        </span>
    </SelectItem>
</template>

<style scoped></style>
