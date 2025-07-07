<script setup lang="ts">
import SolarsystemClass from '@/components/SolarsystemClass.vue';
import LockIcon from '@/components/icons/LockIcon.vue';
import SolarsystemEffect from '@/components/map/SolarsystemEffect.vue';
import { TMapSolarSystem } from '@/types/models';
import SovereigntyIcon from '@/components/map/SovereigntyIcon.vue';

const { map_solarsystem } = defineProps<{
    map_solarsystem: TMapSolarSystem & { is_selected?: boolean; is_hovered?: boolean };
}>();
</script>

<template>
    <button
        :data-selected="map_solarsystem.is_selected"
        :data-hovered="map_solarsystem.is_hovered"
        class="grid h-[40px] grid-cols-[auto_1fr_auto] items-center justify-center gap-x-1 rounded border border-neutral-700 bg-neutral-900 px-2 text-left text-xs transition-colors duration-200 ease-in-out select-none hover:bg-neutral-800 focus:bg-neutral-800 data-[hovered=true]:bg-amber-900 data-[selected=true]:bg-amber-900"
        @drag.prevent
    >
        <span class="pointer-events-none contents">
            <SolarsystemClass :security="map_solarsystem.solarsystem!.security" :wormhole_class="map_solarsystem.class" />
            <span class="pointer-events-none">
                <span class="mr-1 inline-block text-muted-foreground" v-if="map_solarsystem.alias">{{ map_solarsystem.alias }}</span>
                <span>{{ map_solarsystem.solarsystem?.name }}</span>
                <span v-if="map_solarsystem.occupier_alias" class="text-muted-foreground"> ({{ map_solarsystem.occupier_alias }})</span>
            </span>
            <span class="flex items-center gap-2">
                <SovereigntyIcon v-if="map_solarsystem.solarsystem?.sovereignty" :sovereignty="map_solarsystem.solarsystem.sovereignty" />
                <LockIcon v-if="map_solarsystem.pinned" class="text-muted-foreground" />
                <SolarsystemEffect :effect="map_solarsystem.effect" :effects="map_solarsystem.effects" v-if="map_solarsystem.effect" />
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
    </button>
</template>

<style scoped></style>
