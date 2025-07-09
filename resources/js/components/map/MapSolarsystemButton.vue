<script setup lang="ts">
import SolarsystemClass from '@/components/SolarsystemClass.vue';
import LockIcon from '@/components/icons/LockIcon.vue';
import HasSignatures from '@/components/map/HasSignatures.vue';
import SolarsystemEffect from '@/components/map/SolarsystemEffect.vue';
import SovereigntyIcon from '@/components/map/SovereigntyIcon.vue';
import { usePilotsInMapSolarsystem } from '@/composables/usePilotsInMapSolarsystem';
import { TMapSolarSystem } from '@/types/models';

const { map_solarsystem } = defineProps<{
    map_solarsystem: TMapSolarSystem & { is_selected?: boolean; is_hovered?: boolean };
}>();

const pilots = usePilotsInMapSolarsystem(map_solarsystem);
</script>

<template>
    <button
        :data-selected="map_solarsystem.is_selected"
        :data-hovered="map_solarsystem.is_hovered"
        :data-status="map_solarsystem.status"
        :data-has-pilots="pilots?.length > 0"
        class="h-[40px] rounded border border-neutral-700 bg-neutral-900 text-left text-xs transition-colors duration-200 ease-in-out select-none hover:bg-neutral-800 focus:bg-neutral-800 data-[has-pilots=true]:h-[60px] data-[hovered=true]:bg-amber-900 data-[selected=true]:bg-amber-900 data-[status=active]:border-active data-[status=empty]:border-empty data-[status=friendly]:border-friendly data-[status=hostile]:border-hostile data-[status=unknown]:border-unknown data-[status=unscanned]:border-unscanned"
        @drag.prevent
    >
        <span class="pointer-events-none grid grid-cols-[auto_1fr_auto] items-center justify-center gap-x-1 px-2">
            <SolarsystemClass :security="map_solarsystem.solarsystem!.security"
                              :wormhole_class="map_solarsystem.class" />
            <span class="pointer-events-none">
                <span class="mr-1 inline-block" v-if="map_solarsystem.alias">{{ map_solarsystem.alias }}</span>
                <span :data-has-alias="map_solarsystem.alias !== null"
                      class="data-[has-alias=true]:text-muted-foreground">{{ map_solarsystem.solarsystem?.name }}</span>
                <span v-if="map_solarsystem.occupier_alias"
                      class="text-muted-foreground"> ({{ map_solarsystem.occupier_alias }})</span>
            </span>
            <span class="flex items-center gap-1">
                <SovereigntyIcon v-if="map_solarsystem.solarsystem?.sovereignty"
                                 :sovereignty="map_solarsystem.solarsystem.sovereignty" />
                <LockIcon v-if="map_solarsystem.pinned" class="w-4 text-muted-foreground" />
                <HasSignatures v-if="map_solarsystem.signatures_count" />
                <SolarsystemEffect :effect="map_solarsystem.effect" :effects="map_solarsystem.effects"
                                   v-if="map_solarsystem.effect" />
            </span>
            <span class="col-span-2 row-start-2 block text-xs text-muted-foreground" v-if="!map_solarsystem.class">{{
                    map_solarsystem.solarsystem?.region?.name
                }}</span>
            <span class="col-span-3 row-start-2 flex justify-end gap-1 text-right text-xs" v-else>
                <span
                    :data-leads-to="wh.leads_to"
                    class="uppercase data-[leads-to=c1]:text-c1 data-[leads-to=c2]:text-c2 data-[leads-to=c3]:text-c3 data-[leads-to=c4]:text-c4 data-[leads-to=c5]:text-c5 data-[leads-to=c6]:text-c6 data-[leads-to=hs]:text-hs data-[leads-to=ls]:text-ls data-[leads-to=ns]:text-ns"
                    v-for="wh in map_solarsystem.statics"
                    :key="wh.id"
                >
                    {{ wh.leads_to.replace('s', '') }}
                </span>
            </span>
        </span>
        <span v-if="pilots?.length"
              class="flex h-[20px] items-center gap-1 truncate border-t border-neutral-500 px-2 pt-1 text-[10px] leading-0">
            <span class="size-1 animate-pulse rounded-full bg-green-500"></span>{{ pilots.at(0)?.name }}
            <span v-if="pilots.length > 1">and {{ pilots.length - 1 }} more</span>
        </span>
    </button>
</template>

<style scoped></style>
