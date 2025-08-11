<script setup lang="ts">
import MapSolarsystemButton from '@/components/map/MapSolarsystemButton.vue';
import MapSolarsystemContextMenu from '@/components/map/MapSolarsystemContextMenu.vue';
import SolarsystemConnectionHandle from '@/components/map/solarsystem/SolarsystemConnectionHandle.vue';
import SolarsystemDragHandle from '@/components/map/solarsystem/SolarsystemDragHandle.vue';
import { TDataMapSolarSystem, useMapScale, useMapSolarsystem, useNewConnection, usePilotsInMapSolarsystem } from '@/composables/map';
import { useHasWritePermission } from '@/composables/useHasPermission';
import { TShowMapProps } from '@/pages/maps';
import { show } from '@/routes/maps';
import { AppPageProps } from '@/types';
import { Link, usePage } from '@inertiajs/vue3';
import { computed, useTemplateRef } from 'vue';

const { map_solarsystem } = defineProps<{
    map_solarsystem: TDataMapSolarSystem;
}>();

const { scale } = useMapScale();

const element = useTemplateRef('element');

const can_write = useHasWritePermission();

const drag_ref = useTemplateRef('drag_ref');
const link_ref = useTemplateRef('link_ref');

const drag = useMapSolarsystem(
    () => map_solarsystem,
    () => element.value!,
    () => drag_ref?.value?.handle ?? null,
);

const page = usePage<AppPageProps<TShowMapProps>>();

useNewConnection(
    () => link_ref?.value?.new_connection_handle ?? null,
    () => map_solarsystem,
    () => element.value!,
);

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
                    <div class="">
                        <Link
                            :href="
                                show(map_solarsystem.map_id, {
                                    mergeQuery: { map_solarsystem_id: map_solarsystem.id },
                                })
                            "
                            preserve-state
                            preserve-scroll
                            :only="['map', 'selected_map_solarsystem']"
                            prefetch
                            cache-for="2s"
                        >
                            <MapSolarsystemButton :map_solarsystem="map_solarsystem" :pilots="pilots" ref="button" :is_active />
                        </Link>
                        <template v-if="can_write">
                            <SolarsystemDragHandle ref="drag_ref" v-if="!map_solarsystem.pinned" />
                            <SolarsystemConnectionHandle ref="link_ref" />
                        </template>
                    </div>
                </div>
            </MapSolarsystemContextMenu>
        </div>
    </div>
</template>

<style scoped></style>
