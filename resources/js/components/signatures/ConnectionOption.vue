<script setup lang="ts">
import { SelectItem } from '@/components/ui/select';
import SolarsystemClass from '@/components/SolarsystemClass.vue';
import { TProcessedConnection } from '@/composables/map';

defineProps<{
    connection: TProcessedConnection;
}>();
</script>

<template>
    <SelectItem :key="connection.id" :value="connection.id" :text-value="connection.target.solarsystem?.name" class="*:[span]:flex *:[span]:gap-2">
        <div class="w-[2ch] text-center">
            <SolarsystemClass :wormhole_class="connection?.target!.class" :security="connection?.target!.solarsystem?.security" />
        </div>
        <span class="mr-auto truncate" v-if="!connection.target!.alias">{{ connection?.target!.name }}</span>
        <span class="mr-auto truncate" v-else>
            <span class="mr-1">{{ connection?.target!.alias }}</span>
            <span class="-foreground text-muted-foreground">{{ connection?.target!.name }}</span>
        </span>
    </SelectItem>
</template>

<style scoped></style>
