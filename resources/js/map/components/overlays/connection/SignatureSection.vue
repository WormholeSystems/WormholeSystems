<script setup lang="ts">
import SolarsystemClass from '@/components/solarsystem/SolarsystemClass.vue';
import { TTailoredSignature } from '@/pages/maps';

defineProps<{
    signature: TTailoredSignature;
    title: string;
}>();
</script>

<template>
    <div class="space-y-1">
        <div class="border-b pb-1 text-xs font-medium text-foreground">{{ title }}</div>
        <div class="grid grid-cols-2 divide-y truncate text-xs text-muted-foreground *:py-1">
            <div class="col-span-full grid grid-cols-subgrid">
                <span>Type</span>
                <span class="text-right">{{
                    signature.wormhole?.name || signature.signature_type?.name || signature.raw_type_name || 'Unknown'
                }}</span>
            </div>
            <div class="col-span-full grid grid-cols-subgrid">
                <span>Signature ID</span>
                <span class="text-right">{{ signature.signature_id || 'Unknown' }}</span>
            </div>
            <div class="col-span-full grid grid-cols-subgrid" v-if="signature.target_class">
                <span>Leads To</span>
                <span class="flex justify-end text-right">
                    <SolarsystemClass :solarsystem_class="signature.target_class" />
                </span>
            </div>
        </div>
    </div>
</template>
