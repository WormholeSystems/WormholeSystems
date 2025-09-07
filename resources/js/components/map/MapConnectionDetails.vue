<script setup lang="ts">
import { PopoverContent } from '@/components/ui/popover';
import { TMapConnection, TMapSolarSystem } from '@/types/models';
import { computed } from 'vue';
import ConnectionStatus from './connection/ConnectionStatus.vue';
import SignatureSection from './connection/SignatureSection.vue';
import WormholeProperties from './connection/WormholeProperties.vue';

const { connection } = defineProps<{
    connection: TMapConnection & {
        source: TMapSolarSystem;
        target: TMapSolarSystem;
    };
}>();

const outSignature = computed(() => {
    if (!connection.signatures?.length) return null;
    return connection.signatures.find((sig) => !sig.wormhole?.name.startsWith('K162')) || null;
});

const inSignature = computed(() => {
    if (!connection.signatures?.length) return null;
    return connection.signatures.find((sig) => sig.wormhole?.name.startsWith('K162')) || null;
});

const wormhole = computed(() => {
    return outSignature.value?.wormhole || inSignature.value?.wormhole || null;
});
</script>

<template>
    <PopoverContent class="w-60">
        <div class="space-y-3">
            <SignatureSection v-if="outSignature" :signature="outSignature" title="Out Sig" />
            <SignatureSection v-if="inSignature" :signature="inSignature" title="In Sig" />
            <div v-if="!outSignature && !inSignature" class="space-y-1">
                <div class="py-2 text-center text-xs text-muted-foreground">No signatures assigned</div>
            </div>
            <ConnectionStatus :connection="connection" />
            <WormholeProperties v-if="wormhole" :wormhole="wormhole" />
        </div>
    </PopoverContent>
</template>

<style scoped></style>
