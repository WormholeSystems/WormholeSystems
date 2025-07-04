<script setup lang="ts">
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card';
import { Popover, PopoverContent, PopoverTrigger } from '@/components/ui/popover';
import { TWormholeEffect, TWormholeEffectName } from '@/types/models';

const { effects = [], effect } = defineProps<{
    effects?: TWormholeEffect[];
    effect: TWormholeEffectName;
}>();
</script>

<template>
    <Popover>
        <PopoverTrigger class="pointer-events-auto" @drag.prevent>
            <span v-if="effect === 'Pulsar'" class="block size-2 rounded-full bg-pulsar" />
            <span v-else-if="effect === 'Magnetar'" class="block size-2 rounded-full bg-magnetar" />
            <span v-else-if="effect === 'Black Hole'" class="block size-2 rounded-full bg-black-hole" />
            <span v-else-if="effect === 'Red Giant'" class="block size-2 rounded-full bg-red-giant" />
            <span v-else-if="effect === 'Cataclysmic Variable'" class="block size-2 rounded-full bg-catalysmic-variable" />
            <span v-else-if="effect === 'Wolf-Rayet Star'" class="block size-2 rounded-full bg-wolf-rayet-star" />
        </PopoverTrigger>
        <PopoverContent as-child>
            <Card>
                <CardHeader>
                    <CardTitle>{{ effect }}</CardTitle>
                </CardHeader>
                <CardContent>
                    <ul class="grid grid-cols-[auto_1fr] divide-y text-xs">
                        <li class="col-span-full grid grid-cols-subgrid items-center gap-2 py-1" v-for="effect in effects" :key="effect.name">
                            <span class="text-muted-foreground">{{ effect.name }}</span>
                            <span
                                :data-type="effect.type"
                                class="text-right text-foreground data-[type=Buff]:text-green-500 data-[type=Debuff]:text-red-500"
                                >{{ effect.strength }}</span
                            >
                        </li>
                    </ul>
                </CardContent>
            </Card>
        </PopoverContent>
    </Popover>
</template>

<style scoped></style>
