<script setup lang="ts">
import { classMeta } from '@/const/solarsystemClasses';
import { TStringedSolarsystemClass } from '@/types/models';
import { computed } from 'vue';

const { solarsystem_class } = defineProps<{
    solarsystem_class: TStringedSolarsystemClass;
}>();

const meta = computed(() => classMeta(solarsystem_class));
</script>

<template>
    <span :class="`text-${meta.color_token}`">{{ meta.short_label }}</span>
</template>

<style scoped></style>
