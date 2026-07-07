<script setup lang="ts">
import SolarsystemSovereignty from '@/components/map/SolarsystemSovereignty.vue';
import SolarsystemClass from '@/components/solarsystem/SolarsystemClass.vue';
import SolarsystemEffect from '@/components/solarsystem/SolarsystemEffect.vue';
import type { TResolvedSolarsystem } from '@/pages/maps';

const {
    solarsystem,
    alias = null,
    occupier_alias = null,
} = defineProps<{
    solarsystem: TResolvedSolarsystem;
    alias?: string | null;
    occupier_alias?: string | null;
}>();
</script>

<!-- Renders its cells into the parent's subgrid (expects a `grid-cols-[auto_1fr_1fr_auto]` ancestor) so columns align across rows. -->
<template>
    <div class="contents text-xs">
        <span class="justify-self-center">
            <SolarsystemClass :solarsystem_class="solarsystem.class" />
        </span>
        <span class="min-w-0 truncate">
            <span v-if="alias" class="mr-1 font-medium text-foreground">{{ alias }}</span>
            <span class="font-medium" :class="alias ? 'text-muted-foreground' : 'text-foreground'">{{ solarsystem.name }}</span>
            <span v-if="occupier_alias" class="text-muted-foreground"> ({{ occupier_alias }})</span>
        </span>
        <span class="min-w-0 truncate text-muted-foreground">{{ solarsystem.region?.name }}</span>
        <!-- Optional extra grid cell before the sovereignty cell. -->
        <slot />
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
