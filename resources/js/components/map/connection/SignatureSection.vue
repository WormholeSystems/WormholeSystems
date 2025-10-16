<script setup lang="ts">
import { TSignature } from '@/types/models';

defineProps<{
    signature: TSignature;
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
            <div class="col-span-full grid grid-cols-subgrid" v-if="signature.wormhole?.leads_to">
                <span>Leads To</span>
                <span
                    :data-leads-to="signature.wormhole?.leads_to"
                    class="text-right uppercase data-[leads-to=c1]:text-c1 data-[leads-to=c2]:text-c2 data-[leads-to=c3]:text-c3 data-[leads-to=c4]:text-c4 data-[leads-to=c5]:text-c5 data-[leads-to=c6]:text-c6 data-[leads-to=h]:text-hs data-[leads-to=l]:text-ls data-[leads-to=n]:text-ns"
                    >{{ signature.wormhole?.leads_to }}</span
                >
            </div>
        </div>
    </div>
</template>
