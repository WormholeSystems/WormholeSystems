<script setup lang="ts">
import DestinationContextMenu from '@/components/DestinationContextMenu.vue';
import SovereigntyIcon from '@/components/map/SovereigntyIcon.vue';
import { Popover, PopoverContent, PopoverTrigger } from '@/components/ui/popover';
import SolarsystemClass from '@/components/SolarsystemClass.vue';
import { TSolarsystem } from '@/types/models';

defineProps<{
    route: TSolarsystem[];
}>();
</script>

<template>
    <Popover>
        <PopoverTrigger as-child>
            <slot />
        </PopoverTrigger>
        <PopoverContent>
            <div class="flex flex-col gap-2">
                <span class="text-sm text-muted-foreground">Route</span>
                <ul class="grid divide-y text-xs">
                    <DestinationContextMenu v-for="(solarsystem, index) in route" :key="index" :solarsystem_id="solarsystem.id">
                        <li class="col-span-4 grid grid-cols-subgrid gap-x-2 py-1 hover:bg-accent">
                            <div class="flex justify-center">
                                <SolarsystemClass :wormhole_class="solarsystem.class" :security="solarsystem.security" />
                            </div>
                            <span class="truncate">
                                {{ solarsystem.name }}
                            </span>
                            <span class="truncate text-muted-foreground">
                                {{ solarsystem.region?.name }}
                            </span>
                            <SovereigntyIcon v-if="solarsystem.sovereignty" :sovereignty="solarsystem.sovereignty" />
                        </li>
                    </DestinationContextMenu>
                </ul>
            </div>
        </PopoverContent>
    </Popover>
</template>

<style scoped></style>
