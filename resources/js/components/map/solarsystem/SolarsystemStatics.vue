<script setup lang="ts">
import { TWormhole } from '@/types/models';

defineProps<{
    statics: TWormhole[];
}>();
</script>

<template>
    <span class="col-span-3 flex justify-end gap-1 text-right text-xs">
        <span
            v-for="wh in statics"
            :key="wh.id"
            :data-leads-to="wh.leads_to"
            class="uppercase data-[leads-to=c1]:text-c1 data-[leads-to=c2]:text-c2 data-[leads-to=c3]:text-c3 data-[leads-to=c4]:text-c4 data-[leads-to=c5]:text-c5 data-[leads-to=c6]:text-c6 data-[leads-to=hs]:text-hs data-[leads-to=ls]:text-ls data-[leads-to=ns]:text-ns"
        >
            {{ wh.leads_to.replace('s', '') }}
        </span>
    </span>
</template>

<style scoped></style>
