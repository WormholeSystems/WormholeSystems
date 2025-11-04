<script setup lang="ts">
import { CardTitle } from '@/components/ui/card';
import { Popover, PopoverContent, PopoverTrigger } from '@/components/ui/popover';
import { TEffect } from '@/pages/maps';

const { effect } = defineProps<{
    effect: TEffect;
}>();
</script>

<template>
    <span>
        <Popover>
            <PopoverTrigger class="pointer-events-auto" @drag.prevent>
                <div class="grid size-[14px] cursor-pointer place-items-center">
                    <span v-if="effect.name === 'Pulsar'" class="block size-2 rounded-full bg-pulsar" />
                    <span v-else-if="effect.name === 'Magnetar'" class="block size-2 rounded-full bg-magnetar" />
                    <span v-else-if="effect.name === 'Black Hole'" class="block size-2 rounded-full bg-black-hole" />
                    <span v-else-if="effect.name === 'Red Giant'" class="block size-2 rounded-full bg-red-giant" />
                    <span v-else-if="effect.name === 'Cataclysmic Variable'" class="block size-2 rounded-full bg-catalysmic-variable" />
                    <span v-else-if="effect.name === 'Wolf-Rayet Star'" class="block size-2 rounded-full bg-wolf-rayet-star" />
                </div>
            </PopoverTrigger>
            <PopoverContent>
                <CardTitle>{{ effect.name }}</CardTitle>
                <ul class="mt-4 grid grid-cols-[auto_1fr] divide-y text-xs">
                    <li class="col-span-full grid grid-cols-subgrid items-center gap-2 py-1" v-for="buff in effect.effects" :key="buff.name">
                        <span class="text-muted-foreground">{{ buff.name }}</span>
                        <span
                            :data-type="buff.type"
                            class="text-right text-foreground data-[type=Buff]:text-green-500 data-[type=Debuff]:text-red-500"
                            >{{ buff.strength }}</span
                        >
                    </li>
                </ul>
            </PopoverContent>
        </Popover>
    </span>
</template>

<style scoped></style>
