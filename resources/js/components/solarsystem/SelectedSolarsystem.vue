<script setup lang="ts">
import ExternalIcon from '@/components/icons/ExternalIcon.vue';
import Spinner from '@/components/icons/Spinner.vue';
import SolarsystemEffect from '@/components/map/SolarsystemEffect.vue';
import SovereigntyIcon from '@/components/map/SovereigntyIcon.vue';
import SolarsystemSignatures from '@/components/signatures/SolarsystemSignatures.vue';
import Destination from '@/components/solarsystem/Destination.vue';
import SecurityStatus from '@/components/solarsystem/SecurityStatus.vue';
import SolarsystemClass from '@/components/SolarsystemClass.vue';
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card';
import { TDestination, TMap, TMapSolarSystem } from '@/types/models';
import { Deferred } from '@inertiajs/vue3';

const { map_solarsystem } = defineProps<{
    map_solarsystem: TMapSolarSystem;
    jumps?: TDestination[];
    map: TMap;
}>();
</script>

<template>
    <Card>
        <CardHeader class="flex justify-between">
            <div class="">
                <CardTitle class="flex items-center gap-2">
                    <SolarsystemClass :wormhole_class="map_solarsystem.class" v-if="map_solarsystem.class" />
                    <SecurityStatus
                        :security="map_solarsystem.solarsystem?.security"
                        v-else-if="map_solarsystem.solarsystem?.security !== undefined"
                    />
                    {{ map_solarsystem.name }}
                    <SolarsystemEffect :effect="map_solarsystem.effect" :effects="map_solarsystem.effects" v-if="map_solarsystem.effect" />
                    <SovereigntyIcon v-if="map_solarsystem.solarsystem?.sovereignty" :sovereignty="map_solarsystem.solarsystem.sovereignty" />
                    <a
                        :href="`https://zkillboard.com/system/${map_solarsystem.solarsystem?.id}/`"
                        target="_blank"
                        class="text-xs text-muted-foreground hover:text-foreground"
                        rel="noopener noreferrer"
                    >
                        <ExternalIcon />
                    </a>
                </CardTitle>
                <CardDescription>
                    {{ map_solarsystem.solarsystem?.region?.name }} |
                    {{ map_solarsystem.solarsystem?.constellation?.name }}
                </CardDescription>
            </div>
            <Deferred data="jumps">
                <template #fallback>
                    <span class="flex items-center gap-2 text-xs text-muted-foreground">
                        <Spinner class="animate-spin" />
                        <span class="animate-pulse"> Loading jump distances </span>
                    </span>
                </template>
                <div class="flex items-center gap-2">
                    <Destination v-for="jump in jumps" :key="jump.destination.id" :destination="jump" />
                </div>
            </Deferred>
        </CardHeader>
        <CardContent>
            <SolarsystemSignatures :map_solarsystem="map_solarsystem" />
        </CardContent>
    </Card>
</template>

<style scoped></style>
