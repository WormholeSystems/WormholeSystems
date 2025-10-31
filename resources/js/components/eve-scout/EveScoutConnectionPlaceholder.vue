<script setup lang="ts">
import { Skeleton } from '@/components/ui/skeleton';
</script>

<template>
    <div class="rounded-lg border bg-card">
        <div class="border-b bg-muted/50 px-3 py-2">
            <div class="flex items-center justify-between">
                <Skeleton class="h-4 w-24" />
                <Skeleton class="h-3 w-16" />
            </div>
        </div>
        <div class="space-y-3 p-3">
            <div v-for="i in 2" :key="i" class="space-y-2">
                <div class="grid grid-cols-[1fr_auto_1fr] items-center gap-2">
                    <div class="flex flex-col gap-1">
                        <Skeleton class="h-3 w-20" />
                        <Skeleton class="h-3 w-16" />
                    </div>
                    <Skeleton class="h-3 w-8" />
                    <div class="flex flex-col gap-1">
                        <Skeleton class="h-3 w-20" />
                        <Skeleton class="h-3 w-16" />
                    </div>
                </div>
                <div class="flex gap-2">
                    <Skeleton class="h-4 w-16" />
                    <Skeleton class="h-4 w-16" />
                    <Skeleton class="h-4 w-20" />
                </div>
            </div>
        </div>
    </div>
</template>
