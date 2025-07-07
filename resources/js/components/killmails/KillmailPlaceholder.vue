<script setup lang="ts">
import { Skeleton } from '@/components/ui/skeleton';
</script>

<template>
    <div class="col-span-full grid h-12 grid-cols-subgrid items-center gap-2 border-b border-neutral-700 py-2 text-xs last:border-b-0">
        <div class="grid grid-cols-[auto_auto_auto] gap-0.5 gap-x-2">
            <Skeleton class="size-8 rounded-lg" />
            <Skeleton class="size-8 rounded-lg" />
            <Skeleton class="size-8 rounded-lg" />
        </div>

        <div class="">
            <Skeleton class="h-3 w-20 rounded-md" />
            <Skeleton class="h-2 w-16 rounded-md" />
        </div>

        <div class="">
            <Skeleton class="h-3 w-20 rounded-md" />
            <Skeleton class="h-2 w-16 rounded-md" />
        </div>
        <div class="grid grid-cols-[auto_auto_auto] gap-0.5 gap-x-2">
            <Skeleton class="size-8 rounded-lg" />
            <Skeleton class="size-8 rounded-lg" />
            <Skeleton class="size-8 rounded-lg" />
        </div>
        <div class="">
            <Skeleton class="h-3 w-20 rounded-md" />
            <Skeleton class="h-2 w-16 rounded-md" />
        </div>
    </div>
</template>

<style scoped></style>
