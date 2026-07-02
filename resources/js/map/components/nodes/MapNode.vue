<script setup lang="ts">
import NodeCard from '@/map/components/nodes/NodeCard.vue';
import SolarsystemConnectionHandle from '@/map/components/solarsystem/SolarsystemConnectionHandle.vue';
import SolarsystemDragHandle from '@/map/components/solarsystem/SolarsystemDragHandle.vue';
import { ANCHOR_OFFSET, scalePoint } from '@/map/core/coords';
import { useNodeMeasurement } from '@/map/interactions/measure';
import { useMapStore } from '@/map/store/mapStore';
import { TShowMapProps } from '@/pages/maps';
import { show } from '@/routes/maps';
import { AppPageProps } from '@/types';
import { TCharacter } from '@/types/models';
import { Link, usePage } from '@inertiajs/vue3';
import { computed, onBeforeUnmount, useTemplateRef, watch } from 'vue';

/**
 * Store-connected wrapper for one node: resolves the entity, position, and all
 * interaction/page state, then renders the presentational NodeCard plus the
 * drag/connection handles. Gestures and the context menu are wired in a later
 * phase against the data-node-id / data-drag-handle / data-connect-handle hooks.
 */
const { id } = defineProps<{ id: number }>();

const store = useMapStore();
const page = usePage<AppPageProps<TShowMapProps>>();

const system = computed(() => store.systems.get(id) ?? null);

/**
 * The stored position is the connection anchor; the node's top-left sits one
 * ANCHOR_OFFSET above/left of it. Both are emitted in screen units here (the
 * old tree did the same in two steps: left/top at the scaled anchor, then a
 * -scale*offset translate on an inner div).
 */
const wrapperStyle = computed(() => {
    const position = store.renderPosition(id);
    if (!position) {
        return null;
    }
    const scale = store.scale.value;
    const scaled = scalePoint(position, scale);
    return {
        transform: `translate(${scaled.x - ANCHOR_OFFSET.x * scale}px, ${scaled.y - ANCHOR_OFFSET.y * scale}px)`,
    };
});

const isSelected = computed(() => store.isSelected(id));
const isHovered = computed(() => store.isHovered(id));

function handlePointerEnter(): void {
    store.hoveredSolarsystemId.value = id;
}

function handlePointerLeave(): void {
    if (store.hoveredSolarsystemId.value === id) {
        store.hoveredSolarsystemId.value = null;
    }
}

const pilots = computed<TCharacter[]>(() => {
    const current = system.value;
    if (!current) {
        return [];
    }
    return (page.props.map_characters ?? []).filter((character) => character.status?.solarsystem_id === current.solarsystem_id);
});

const isActive = computed(() => {
    return system.value !== null && page.props.selected_map_solarsystem?.solarsystem_id === system.value.solarsystem_id;
});

const isHome = computed(() => {
    return system.value !== null && store.meta.value?.home_solarsystem_id === system.value.solarsystem_id;
});

const isRally = computed(() => {
    return system.value !== null && store.meta.value?.rally_solarsystem_id === system.value.solarsystem_id;
});

const threatLevel = computed(() => {
    return page.props.map_user_settings?.show_threat_level ? (system.value?.threat_level ?? null) : null;
});

const fixedWidth = computed(() => store.isTreeLayout.value || store.isConstantWidthEnabled.value);

const canWrite = computed(() => page.props.permission === 'member' || page.props.permission === 'manager');

const linkHref = computed(() => {
    const meta = store.meta.value;
    const current = system.value;
    if (!meta || !current) {
        return null;
    }
    return show(meta.slug, {
        mergeQuery: { solarsystem_id: current.solarsystem_id },
    });
});

const card = useTemplateRef('card');
const { observeNode, unobserveNode } = useNodeMeasurement();

watch(
    () => card.value?.root ?? null,
    (element, previous) => {
        if (previous) {
            unobserveNode(previous, id);
        }
        if (element) {
            observeNode(element, id);
        }
    },
    { flush: 'post' },
);

onBeforeUnmount(() => {
    const element = card.value?.root;
    if (element) {
        unobserveNode(element, id);
    }
});
</script>

<template>
    <div
        v-if="system && wrapperStyle && linkHref"
        :data-node-id="id"
        :style="wrapperStyle"
        class="pointer-events-none absolute hover:z-20 data-[active=true]:z-10"
        :data-active="isActive"
    >
        <div class="pointer-events-auto origin-top-left" @pointerenter="handlePointerEnter" @pointerleave="handlePointerLeave">
            <div
                class="group relative origin-top-left"
                :style="{
                    scale: store.scale.value,
                }"
            >
                <div class="">
                    <Link
                        :href="linkHref"
                        preserve-state
                        preserve-scroll
                        :only="['map', 'selected_map_solarsystem', 'map_navigation', 'map_characters', 'eve_scout_connections', 'threat_analysis']"
                        prefetch
                        cache-for="2s"
                    >
                        <NodeCard
                            ref="card"
                            :system="system"
                            :pilots="pilots"
                            :is-selected="isSelected"
                            :is-hovered="isHovered"
                            :is-active="isActive"
                            :is-home="isHome"
                            :is-rally="isRally"
                            :fixed-width="fixedWidth"
                            :threat-level="threatLevel"
                        />
                    </Link>
                    <template v-if="canWrite">
                        <SolarsystemDragHandle data-drag-handle v-if="!system.pinned && !store.isLayoutLocked.value" />
                        <SolarsystemConnectionHandle data-connect-handle :data-connection-source="system.solarsystem_id" />
                    </template>
                </div>
            </div>
        </div>
    </div>
</template>

<style scoped></style>
