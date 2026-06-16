<script setup lang="ts">
import SolarsystemSovereignty from '@/components/map/SolarsystemSovereignty.vue';
import SolarsystemClass from '@/components/solarsystem/SolarsystemClass.vue';
import SolarsystemEffect from '@/components/solarsystem/SolarsystemEffect.vue';
import type { TStaticSolarsystem } from '@/types/static-data';

const { solarsystem, alias = null } = defineProps<{
    solarsystem: TStaticSolarsystem;
    alias?: string | null;
}>();
</script>

<!-- Renders its cells into the parent's subgrid (expects a `grid-cols-[auto_1fr_1fr_auto]` ancestor) so columns align across rows. -->
<template>
    <div class="contents text-xs">
        <span class="justify-self-center">
            <SolarsystemClass :wormhole_class="solarsystem.class" :security="solarsystem.security" />
        </span>
        <span class="min-w-0 truncate">
            <span class="font-medium text-foreground">{{ solarsystem.name }}</span>
            <span v-if="alias" class="text-muted-foreground"> ({{ alias }})</span>
        </span>
        <span class="min-w-0 truncate text-muted-foreground">{{ solarsystem.region?.name }}</span>
        <span class="flex justify-center">
            <SolarsystemSovereignty :sovereignty="solarsystem.sovereignty" :solarsystem-id="solarsystem.id">
                <template #fallback>
                    <SolarsystemEffect v-if="solarsystem.effect" :effect="solarsystem.effect.name" />
                </template>
            </SolarsystemSovereignty>
        </span>
    </div>
</template>

<style scoped></style>
