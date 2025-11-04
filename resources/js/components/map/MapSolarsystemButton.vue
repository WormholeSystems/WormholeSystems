<script setup lang="ts">
import LockIcon from '@/components/icons/LockIcon.vue';
import SatelliteDish from '@/components/icons/SatelliteDish.vue';
import HasExtraConnections from '@/components/map/HasExtraConnections.vue';
import SolarsystemEffect from '@/components/map/SolarsystemEffect.vue';
import SolarsystemSovereignty from '@/components/map/SolarsystemSovereignty.vue';
import SolarsystemName from '@/components/map/solarsystem/SolarsystemName.vue';
import SolarsystemPilots from '@/components/map/solarsystem/SolarsystemPilots.vue';
import SolarsystemRegion from '@/components/map/solarsystem/SolarsystemRegion.vue';
import SolarsystemStatics from '@/components/map/solarsystem/SolarsystemStatics.vue';
import SolarsystemClass from '@/components/solarsystem/SolarsystemClass.vue';
import { Button } from '@/components/ui/button';
import { Input } from '@/components/ui/input';
import { Popover, PopoverAnchor, PopoverContent } from '@/components/ui/popover';
import { TDataMapSolarSystem } from '@/composables/map';
import MapSolarsystems from '@/routes/map-solarsystems';
import { TCharacter } from '@/types/models';
import { useForm } from '@inertiajs/vue3';
import { computed, ref } from 'vue';

const { map_solarsystem } = defineProps<{
    map_solarsystem: TDataMapSolarSystem;
    pilots: TCharacter[];
    is_active?: boolean;
}>();

const form = useForm<{
    alias: string;
    occupier_alias: string;
}>({
    alias: map_solarsystem.alias ?? '',
    occupier_alias: map_solarsystem.occupier_alias ?? '',
});

const open = ref(false);

const extra_connections_count = computed(() => {
    const connections_count = map_solarsystem.wormhole_signatures_count ?? 0;
    const mapped_connections_count = map_solarsystem.map_connections_count ?? 0;
    return Math.max(0, connections_count - mapped_connections_count);
});

function handleSubmit() {
    form.put(MapSolarsystems.update(map_solarsystem.id).url, {
        onSuccess: () => {
            open.value = false;
        },
        preserveScroll: true,
        preserveState: true,
        only: ['map', 'selected_map_solarsystem'],
    });
}
</script>

<template>
    <div
        :data-selected="map_solarsystem.is_selected"
        :data-hovered="map_solarsystem.is_hovered"
        :data-status="map_solarsystem.status"
        :data-has-pilots="pilots.length > 0"
        :data-is-active="is_active"
        class="grid h-[40px] rounded border border-neutral-300 bg-white text-left text-xs ring-offset-2 ring-offset-neutral-50 transition-colors duration-200 ease-in-out select-none hover:bg-white focus:bg-white data-[has-pilots=true]:h-[60px] data-[hovered=true]:outline-2 data-[hovered=true]:outline-yellow-500 data-[is-active=true]:ring-2 data-[is-active=true]:ring-amber-500 data-[selected=true]:bg-amber-100 data-[status=active]:border-active data-[status=empty]:border-empty data-[status=friendly]:border-friendly data-[status=hostile]:border-hostile data-[status=unknown]:border-unknown dark:border-neutral-700 dark:bg-neutral-900 dark:ring-offset-neutral-900 dark:hover:bg-neutral-800 dark:focus:bg-neutral-800 dark:data-[is-active=true]:ring-2 dark:data-[is-active=true]:ring-amber-500 dark:data-[selected=true]:bg-amber-900 dark:data-[status=active]:border-active dark:data-[status=empty]:border-empty dark:data-[status=friendly]:border-friendly dark:data-[status=hostile]:border-hostile dark:data-[status=unscanned]:border-unscanned"
        @dblclick="open = true"
        @drag.prevent
    >
        <div class="row-start-1 grid grid-cols-[auto_1fr_auto] items-center justify-center gap-x-1 px-2">
            <SolarsystemClass :security="map_solarsystem.solarsystem!.security" :wormhole_class="map_solarsystem.solarsystem.class" />
            <Popover :open="open" @update:open="(value) => open && (open = value)">
                <PopoverAnchor>
                    <SolarsystemName :map_solarsystem="map_solarsystem" />
                </PopoverAnchor>
                <PopoverContent>
                    <form @submit.prevent="handleSubmit" class="grid gap-2">
                        <Input v-model="form.alias" type="text" placeholder="Alias" class="w-full" />
                        <Input v-model="form.occupier_alias" type="text" placeholder="Occupier Alias" class="w-full" />
                        <Button type="submit"> Save</Button>
                    </form>
                </PopoverContent>
            </Popover>
            <div class="col-start-3 row-start-1 flex items-center gap-1">
                <LockIcon v-if="map_solarsystem.pinned" class="size-[14px] text-muted-foreground" />
                <SatelliteDish v-if="map_solarsystem.signatures_count" class="size-[14px] text-amber-500" />
                <HasExtraConnections v-if="extra_connections_count" :extra_connections_count="extra_connections_count" />
                <SolarsystemSovereignty v-if="map_solarsystem.solarsystem?.sovereignty" :sovereignty="map_solarsystem.solarsystem.sovereignty" />
                <SolarsystemEffect :effect="map_solarsystem.solarsystem.effect" v-if="map_solarsystem.solarsystem.effect" />
            </div>
            <SolarsystemRegion
                :region="map_solarsystem.solarsystem?.region"
                v-if="map_solarsystem.solarsystem?.region && !map_solarsystem.solarsystem.class"
            />
            <SolarsystemStatics v-else-if="map_solarsystem.solarsystem.statics" :statics="map_solarsystem.solarsystem.statics" />
        </div>
        <SolarsystemPilots v-if="pilots.length" :pilots />
    </div>
</template>

<style scoped></style>
