<script setup lang="ts">
import SolarsystemEffectBadge from '@/components/solarsystem/SolarsystemEffectBadge.vue';
import { CardTitle } from '@/components/ui/card';
import { Popover, PopoverContent, PopoverTrigger } from '@/components/ui/popover';
import { TEffect } from '@/pages/maps';
import { ArrowDown, ArrowUp } from 'lucide-vue-next';

const { effect } = defineProps<{
    effect: TEffect;
}>();
</script>

<template>
    <span>
        <Popover>
            <PopoverTrigger class="pointer-events-auto" @drag.prevent>
                <div class="grid size-[14px] cursor-pointer place-items-center">
                    <SolarsystemEffectBadge :name="effect.name" />
                </div>
            </PopoverTrigger>
            <PopoverContent>
                <CardTitle class="flex items-center gap-2"><SolarsystemEffectBadge :name="effect.name" /> {{ effect.name }}</CardTitle>
                <ul class="mt-4 grid grid-cols-[auto_1fr] divide-y text-xs">
                    <li class="col-span-full grid grid-cols-subgrid items-center gap-2 py-1" v-for="buff in effect.effects" :key="buff.name">
                        <span class="text-muted-foreground">{{ buff.name }}</span>
                        <span
                            :data-type="buff.type"
                            class="inline-flex items-center justify-end gap-1 text-right text-foreground data-[type=Buff]:text-green-500 data-[type=Debuff]:text-red-500"
                        >
                            <ArrowUp v-if="buff.type === 'Buff'" class="size-3" aria-label="Buff" />
                            <ArrowDown v-else class="size-3" aria-label="Debuff" />
                            {{ buff.strength }}
                        </span>
                    </li>
                </ul>
            </PopoverContent>
        </Popover>
    </span>
</template>

<style scoped></style>
