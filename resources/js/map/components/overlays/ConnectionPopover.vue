<script setup lang="ts">
import { Popover, PopoverAnchor } from '@/components/ui/popover';
import MapConnectionDetails from '@/map/components/overlays/MapConnectionDetails.vue';
import { TMapConnection, TMapSolarsystem } from '@/pages/maps';

/**
 * The connection-details popover, anchored to the click point via a virtual
 * reference. Positioning it off a real element fails when the map (which
 * scrolls, and whose ancestors may form a containing block) shifts it; a
 * virtual reference is read in viewport coordinates, like the context menu.
 */
defineProps<{
    connection: TMapConnection & { source: TMapSolarsystem; target: TMapSolarsystem };
    reference?: { getBoundingClientRect: () => DOMRect };
}>();

const open = defineModel<boolean>('open', { required: true });
</script>

<template>
    <Popover v-model:open="open">
        <PopoverAnchor :reference="reference" />
        <MapConnectionDetails :connection="connection" />
    </Popover>
</template>

<style scoped></style>
