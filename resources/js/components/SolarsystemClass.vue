<script setup lang="ts">
const { security = 0, wormhole_class } = defineProps<{
    wormhole_class: number | null;
    security?: number;
}>();
</script>

<template>
    <span
        v-if="wormhole_class"
        :data-c1="wormhole_class === 1"
        :data-c2="wormhole_class === 2"
        :data-c3="wormhole_class === 3"
        :data-c4="wormhole_class === 4"
        :data-c5="wormhole_class === 5"
        :data-c6="wormhole_class === 6"
        class="data-[c1=true]:text-cyan-400 data-[c2=true]:text-blue-500 data-[c3=true]:text-violet-500 data-[c4=true]:text-purple-500 data-[c5=true]:text-orange-300 data-[c6=true]:text-orange-600"
    >
        C{{ wormhole_class }}
    </span>
    <span class="text-green-500" v-else-if="security >= 0.5"> H </span>
    <span class="text-orange-500" v-else-if="security >= 0.1"> L </span>
    <span class="text-red-500" v-else> N </span>
</template>

<style scoped></style>
