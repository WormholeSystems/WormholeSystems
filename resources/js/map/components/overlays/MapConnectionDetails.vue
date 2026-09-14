<script setup lang="ts">
import { PopoverContent } from '@/components/ui/popover';
import { TMapConnection, TMapSolarsystem } from '@/pages/maps';
import { computed } from 'vue';
import ConnectionStatus from './connection/ConnectionStatus.vue';
import MassTracking from './connection/MassTracking.vue';
import SignatureSection from './connection/SignatureSection.vue';
import WormholeProperties from './connection/WormholeProperties.vue';

const { connection } = defineProps<{
    connection: TMapConnection & {
        source: TMapSolarsystem;
        target: TMapSolarsystem;
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

const showMassTracking = computed(() => {
    return connection.type === 'wormhole';
});
</script>

<template>
    <!--
        The jump list grows without bound, so the popup is capped to the space the
        popover actually has and scrolls past it. Mass tracking moves into a second
        column where the viewport is wide enough to carry one.
    -->
    <PopoverContent :class="['max-h-(--reka-popover-content-available-height) w-60 overflow-y-auto', showMassTracking ? 'sm:w-[30rem]' : '']">
        <div class="grid items-start gap-3" :class="showMassTracking ? 'sm:grid-cols-2' : ''">
            <div class="space-y-3">
                <SignatureSection v-if="outSignature" :signature="outSignature" title="Out Sig" />
                <SignatureSection v-if="inSignature" :signature="inSignature" title="In Sig" />
                <div v-if="!outSignature && !inSignature" class="space-y-1">
                    <div class="py-2 text-center text-xs text-muted-foreground">No signatures assigned</div>
                </div>
                <ConnectionStatus :connection="connection" />
                <WormholeProperties v-if="wormhole" :wormhole="wormhole" />
            </div>
            <MassTracking v-if="showMassTracking" :connection="connection" :wormhole="wormhole" />
        </div>
    </PopoverContent>
</template>

<style scoped></style>
