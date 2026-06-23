<script setup lang="ts">
import MapSolarsystemButton from '@/components/map/MapSolarsystemButton.vue';
import MapSolarsystemContextMenu from '@/components/map/MapSolarsystemContextMenu.vue';
import SolarsystemConnectionHandle from '@/components/map/solarsystem/SolarsystemConnectionHandle.vue';
import SolarsystemDragHandle from '@/components/map/solarsystem/SolarsystemDragHandle.vue';
import {
    is_layout_locked,
    item_anchor_offset,
    TDataMapSolarSystem,
    useMapScale,
    useMapSolarsystem,
    useNewConnection,
    usePilotsInMapSolarsystem,
} from '@/composables/map';
import { useMap } from '@/composables/useMap';
import usePermission from '@/composables/usePermission';
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

const { canEdit: can_write } = usePermission();

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

const map = useMap();

const is_active = computed(() => {
    return page.props.selected_map_solarsystem?.solarsystem_id === map_solarsystem.solarsystem_id;
});

const is_home = computed(() => {
    return map.value.home_solarsystem_id === map_solarsystem.solarsystem_id;
});

const is_rally = computed(() => {
    return map.value.rally_solarsystem_id === map_solarsystem.solarsystem_id;
});
</script>

<template>
    <div
        ref="element"
        :style="drag.style.value"
        class="pointer-events-none absolute hover:z-20 data-[active=true]:z-10"
        :data-active="page.props.selected_map_solarsystem?.solarsystem_id === map_solarsystem.solarsystem_id"
    >
        <div
            :style="{
                '--tw-translate-x': -scale * item_anchor_offset.x + 'px',
                '--tw-translate-y': -scale * item_anchor_offset.y + 'px',
            }"
            class="pointer-events-auto origin-top-left translate-[var(--translate-x),var(--translate-y)]"
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
                                show(map.slug, {
                                    mergeQuery: { solarsystem_id: map_solarsystem.solarsystem_id },
                                })
                            "
                            preserve-state
                            preserve-scroll
                            :only="[
                                'map',
                                'selected_map_solarsystem',
                                'map_navigation',
                                'map_characters',
                                'eve_scout_connections',
                                'threat_analysis',
                            ]"
                            prefetch
                            cache-for="2s"
                        >
                            <MapSolarsystemButton :map_solarsystem="map_solarsystem" :pilots="pilots" ref="button" :is_active :is_home :is_rally />
                        </Link>
                        <template v-if="can_write">
                            <SolarsystemDragHandle ref="drag_ref" v-if="!map_solarsystem.pinned && !is_layout_locked" />
                            <SolarsystemConnectionHandle ref="link_ref" :data-connection-source="map_solarsystem.solarsystem_id" />
                        </template>
                    </div>
                </div>
            </MapSolarsystemContextMenu>
        </div>
    </div>
</template>

<style scoped></style>
