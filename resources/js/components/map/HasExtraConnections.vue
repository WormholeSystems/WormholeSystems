<script setup lang="ts">
import ExtraWormholeIcon from '@/components/icons/ExtraWormholeIcon.vue';
import { Tooltip, TooltipContent, TooltipTrigger } from '@/components/ui/tooltip';

defineProps<{
    extra_connections_count: number;
}>();
</script>

<template>
    <Tooltip :delay-duration="500">
        <TooltipTrigger>
            <ExtraWormholeIcon class="size-[14px] text-sky-500" />
        </TooltipTrigger>
        <TooltipContent> Has {{ extra_connections_count }} unmapped wormholes </TooltipContent>
    </Tooltip>
</template>

<style scoped></style>
