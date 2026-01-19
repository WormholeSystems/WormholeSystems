<script setup lang="ts">
import SolarsystemSovereignty from '@/components/map/SolarsystemSovereignty.vue';
import SolarsystemClass from '@/components/solarsystem/SolarsystemClass.vue';
import { Label } from '@/components/ui/label';
import { TSolarsystem } from '@/pages/maps';

const { solarsystem } = defineProps<{ solarsystem: TSolarsystem | null }>();
</script>

<template>
    <div class="h-20 rounded-lg border p-3">
        <Label class="text-xs text-muted-foreground"><slot name="label" /></Label>
        <div v-if="solarsystem" class="grid grid-cols-[1fr_auto]">
            <div class="font-medium">{{ solarsystem.name }}</div>
            <div class="place-self-center">
                <SolarsystemClass :wormhole_class="solarsystem.class" :security="solarsystem.security" />
            </div>
            <div class="text-sm text-muted-foreground">{{ solarsystem.region?.name }}</div>
            <div class="place-self-center">
                <SolarsystemSovereignty :sovereignty="solarsystem.sovereignty" :solarsystem-id="solarsystem.id" />
            </div>
        </div>
        <div v-else class="mt-1 text-sm text-muted-foreground">No system selected</div>
    </div>
</template>

<style scoped></style>
