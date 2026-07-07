<script setup lang="ts">
import SolarsystemSovereignty from '@/components/map/SolarsystemSovereignty.vue';
import SolarsystemClass from '@/components/solarsystem/SolarsystemClass.vue';
import { Label } from '@/components/ui/label';
import { useSolarsystemAliases } from '@/composables/useSolarsystemAliases';
import { useMapSolarsystems } from '@/map/api';
import { TSolarsystem } from '@/pages/maps';
import { computed } from 'vue';

const { solarsystem } = defineProps<{ solarsystem: TSolarsystem | null }>();

const { map_solarsystems } = useMapSolarsystems();
const { getAlias } = useSolarsystemAliases(map_solarsystems);
const alias = computed(() => (solarsystem ? getAlias(solarsystem.id) : null));
</script>

<template>
    <div class="h-20 rounded-lg border p-3">
        <Label class="text-xs text-muted-foreground"><slot name="label" /></Label>
        <div v-if="solarsystem" class="grid grid-cols-[1fr_auto]">
            <div class="font-medium">
                <span v-if="alias" class="mr-1">{{ alias }}</span>
                <span :class="{ 'text-muted-foreground': alias }">{{ solarsystem.name }}</span>
            </div>
            <div class="place-self-center">
                <SolarsystemClass :solarsystem_class="solarsystem.class" />
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
