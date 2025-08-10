<script setup lang="ts">
import LockIcon from '@/components/icons/LockIcon.vue';
import SatelliteDish from '@/components/icons/SatelliteDish.vue';
import MapSolarsystemContextMenu from '@/components/map/MapSolarsystemContextMenu.vue';
import SolarsystemConnectionHandle from '@/components/map/solarsystem/SolarsystemConnectionHandle.vue';
import SolarsystemDragHandle from '@/components/map/solarsystem/SolarsystemDragHandle.vue';
import SolarsystemName from '@/components/map/solarsystem/SolarsystemName.vue';
import SolarsystemPilots from '@/components/map/solarsystem/SolarsystemPilots.vue';
import SolarsystemRegion from '@/components/map/solarsystem/SolarsystemRegion.vue';
import SolarsystemStatics from '@/components/map/solarsystem/SolarsystemStatics.vue';
import SolarsystemEffect from '@/components/map/SolarsystemEffect.vue';
import SolarsystemSovereignty from '@/components/map/SolarsystemSovereignty.vue';
import SolarsystemClass from '@/components/SolarsystemClass.vue';
import { Button } from '@/components/ui/button';
import { Input } from '@/components/ui/input';
import { Popover, PopoverAnchor, PopoverContent } from '@/components/ui/popover';
import { useMapScale, useMapSolarsystem, useNewConnection, usePilotsInMapSolarsystem } from '@/composables/map';
import { useHasWritePermission } from '@/composables/useHasPermission';
import { TShowMapProps } from '@/pages/maps';
import MapSolarsystems from '@/routes/map-solarsystems';
import { AppPageProps } from '@/types';
import { TMapSolarSystem } from '@/types/models';
import { router, useForm, usePage } from '@inertiajs/vue3';
import { computed, ref, useTemplateRef } from 'vue';

const { map_solarsystem } = defineProps<{
    map_solarsystem: TMapSolarSystem & { is_selected?: boolean; is_hovered?: boolean };
}>();

const { scale } = useMapScale();

const element = useTemplateRef('element');
const handle = useTemplateRef('handle');
const new_connection_handle = useTemplateRef('new_connection_handle');

const drag = useMapSolarsystem(
    () => map_solarsystem,
    () => element.value!,
    () => handle.value?.handle ?? null,
);

const open = ref(false);

const page = usePage<AppPageProps<TShowMapProps>>();
const can_write = useHasWritePermission();
const form = useForm<{
    alias: string;
    occupier_alias: string;
}>({
    alias: map_solarsystem.alias ?? '',
    occupier_alias: map_solarsystem.occupier_alias ?? '',
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

useNewConnection(
    () => new_connection_handle.value?.new_connection_handle ?? null,
    () => map_solarsystem,
    () => element.value!,
);

function handleBadgeClick() {
    if (page.props.selected_map_solarsystem?.id === map_solarsystem.id) {
        return;
    }
    router.reload({
        data: {
            map_solarsystem_id: map_solarsystem.id,
        },
        only: ['selected_map_solarsystem', 'map_route_solarsystems', 'map_characters'],
    });
}

function handleBadgeDblClick() {
    open.value = true;
}

const pilots = usePilotsInMapSolarsystem(map_solarsystem);

const is_active = computed(() => {
    return page.props.selected_map_solarsystem?.id === map_solarsystem.id;
});
</script>

<template>
    <div
        ref="element"
        :style="drag.style.value"
        class="absolute hover:z-20 data-[active=true]:z-10"
        :data-active="page.props.selected_map_solarsystem?.id === map_solarsystem.id"
    >
        <div
            :style="{
                '--tw-translate-x': -scale * 40 + 'px',
                '--tw-translate-y': -scale * 20 + 'px',
            }"
            class="origin-top-left translate-[var(--translate-x),var(--translate-y)]"
        >
            <MapSolarsystemContextMenu :map_solarsystem>
                <div
                    class="group relative origin-top-left"
                    :style="{
                        scale: scale,
                    }"
                >
                    <div
                        :data-selected="map_solarsystem.is_selected"
                        :data-hovered="map_solarsystem.is_hovered"
                        :data-status="map_solarsystem.status"
                        :data-has-pilots="pilots?.length > 0"
                        :data-is-active="is_active"
                        class="grid h-[40px] rounded border border-neutral-300 bg-white text-left text-xs ring-offset-2 ring-offset-neutral-50 transition-colors duration-200 ease-in-out select-none hover:bg-white focus:bg-white data-[has-pilots=true]:h-[60px] data-[hovered=true]:outline-2 data-[hovered=true]:outline-yellow-500 data-[is-active=true]:ring-2 data-[is-active=true]:ring-amber-500 data-[selected=true]:bg-amber-100 data-[status=active]:border-active data-[status=empty]:border-empty data-[status=friendly]:border-friendly data-[status=hostile]:border-hostile data-[status=unknown]:border-unknown dark:border-neutral-700 dark:bg-neutral-900 dark:ring-offset-neutral-900 dark:hover:bg-neutral-800 dark:focus:bg-neutral-800 dark:data-[is-active=true]:ring-2 dark:data-[is-active=true]:ring-amber-500 dark:data-[selected=true]:bg-amber-900 dark:data-[status=active]:border-active dark:data-[status=empty]:border-empty dark:data-[status=friendly]:border-friendly dark:data-[status=hostile]:border-hostile dark:data-[status=unscanned]:border-unscanned"
                        @click="handleBadgeClick"
                        @dblclick="handleBadgeDblClick"
                        @drag.prevent
                    >
                        <div class="row-start-1 grid grid-cols-[auto_1fr_auto] items-center justify-center gap-x-1 px-2">
                            <SolarsystemClass :security="map_solarsystem.solarsystem!.security" :wormhole_class="map_solarsystem.class" />
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
                                <LockIcon v-if="map_solarsystem.pinned" class="w-4 text-muted-foreground" />
                                <SatelliteDish v-if="map_solarsystem.signatures_count" class="w-4 text-amber-500" />
                                <SolarsystemSovereignty
                                    v-if="map_solarsystem.solarsystem?.sovereignty"
                                    :sovereignty="map_solarsystem.solarsystem.sovereignty"
                                />
                                <SolarsystemEffect
                                    :effect="map_solarsystem.effect"
                                    :effects="map_solarsystem.effects"
                                    v-if="map_solarsystem.effect"
                                />
                            </div>
                            <SolarsystemRegion
                                :region="map_solarsystem.solarsystem?.region"
                                v-if="map_solarsystem.solarsystem?.region && !map_solarsystem.class"
                            />
                            <SolarsystemStatics v-else-if="map_solarsystem.statics" :statics="map_solarsystem.statics" />
                        </div>
                        <SolarsystemPilots v-if="pilots.length" :pilots />
                    </div>
                    <template v-if="can_write">
                        <SolarsystemDragHandle ref="handle" v-if="!map_solarsystem.pinned" />
                        <SolarsystemConnectionHandle ref="new_connection_handle" />
                    </template>
                </div>
            </MapSolarsystemContextMenu>
        </div>
    </div>
</template>

<style scoped></style>
