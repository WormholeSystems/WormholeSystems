<script setup lang="ts">
import ExternalIcon from '@/components/icons/ExternalIcon.vue';
import Spinner from '@/components/icons/Spinner.vue';
import SolarsystemEffect from '@/components/map/SolarsystemEffect.vue';
import SovereigntyIcon from '@/components/map/SovereigntyIcon.vue';
import Destination from '@/components/solarsystem/Destination.vue';
import SecurityStatus from '@/components/solarsystem/SecurityStatus.vue';
import SolarsystemClass from '@/components/SolarsystemClass.vue';
import { Card, CardAction, CardDescription, CardHeader, CardTitle } from '@/components/ui/card';
import { TMap, TMapRouteSolarsystem, TMapSolarSystem } from '@/types/models';
import { Deferred } from '@inertiajs/vue3';
import { computed } from 'vue';

const { map_solarsystem, map_route_solarsystems } = defineProps<{
    map_solarsystem: TMapSolarSystem;
    map_route_solarsystems?: TMapRouteSolarsystem[];
    map: TMap;
}>();

const pinned = computed(() => map_route_solarsystems?.filter((m) => m.is_pinned));
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
                {{ map_solarsystem.solarsystem?.region?.name }} |
                {{ map_solarsystem.solarsystem?.constellation?.name }}
            </CardDescription>
            <CardAction>
                <Deferred data="map_route_solarsystems">
                    <template #fallback>
                        <span class="flex items-center gap-2 text-xs text-muted-foreground">
                            <Spinner class="animate-spin" />
                            <span class="animate-pulse"> Loading pinned destinations...</span>
                        </span>
                    </template>
                    <div class="flex items-center gap-2">
                        <Destination v-for="jump in pinned" :key="jump.solarsystem.id" :destination="jump" />
                    </div>
                </Deferred>
            </CardAction>
        </CardHeader>
    </Card>
</template>

<style scoped></style>
