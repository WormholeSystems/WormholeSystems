<script setup lang="ts">
import SolarsystemEffectBadge from '@/components/solarsystem/SolarsystemEffectBadge.vue';
import { TWormholeEffectName } from '@/types/models';

const { effect } = defineProps<{
    effect: TWormholeEffectName;
}>();
</script>

<template>
    <SolarsystemEffectBadge :name="effect" size="sm" :title="effect" />
</template>

<style scoped></style>
