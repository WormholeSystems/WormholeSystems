<script setup lang="ts">
const { security = 0, wormhole_class } = defineProps<{
    wormhole_class: number | null;
    security?: number;
}>();
</script>

<template>
    <span
        v-if="wormhole_class"
        :data-class="wormhole_class"
        class="col-start-1 row-start-1 data-[class=1]:text-c1 data-[class=2]:text-c2 data-[class=3]:text-c3 data-[class=4]:text-c4 data-[class=5]:text-c5 data-[class=6]:text-c6"
    >
        C{{ wormhole_class }}
    </span>
    <span class="text-green-500" v-else-if="security >= 0.5"> H </span>
    <span class="text-orange-500" v-else-if="security >= 0.1"> L </span>
    <span class="text-red-500" v-else> N </span>
</template>

<style scoped></style>
