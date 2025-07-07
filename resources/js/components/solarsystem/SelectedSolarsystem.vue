<script setup lang="ts">
import ExternalIcon from '@/components/icons/ExternalIcon.vue';
import SolarsystemEffect from '@/components/map/SolarsystemEffect.vue';
import SovereigntyIcon from '@/components/map/SovereigntyIcon.vue';
import SolarsystemSignatures from '@/components/signatures/SolarsystemSignatures.vue';
import SecurityStatus from '@/components/solarsystem/SecurityStatus.vue';
import SolarsystemClass from '@/components/SolarsystemClass.vue';
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card';
import { TMap, TMapSolarSystem } from '@/types/models';

const { map_solarsystem } = defineProps<{
    map_solarsystem: TMapSolarSystem;
    map: TMap;
}>();
</script>

<template>
    <Card>
        <CardHeader>
            <CardTitle class="flex items-center gap-2">
                <SolarsystemClass :wormhole_class="map_solarsystem.class" v-if="map_solarsystem.class" />
                <SecurityStatus :security="map_solarsystem.solarsystem?.security" v-else-if="map_solarsystem.solarsystem?.security !== undefined" />
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
                {{ map_solarsystem.solarsystem?.region?.name }} | {{ map_solarsystem.solarsystem?.constellation?.name }}
            </CardDescription>
        </CardHeader>
        <CardContent>
            <SolarsystemSignatures :map_solarsystem="map_solarsystem" :map />
        </CardContent>
    </Card>
</template>

<style scoped></style>
