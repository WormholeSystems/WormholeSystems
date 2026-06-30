<script setup lang="ts">
import {
    ContextMenuItem,
    ContextMenuLabel,
    ContextMenuSeparator,
    ContextMenuSub,
    ContextMenuSubContent,
    ContextMenuSubTrigger,
} from '@/components/ui/context-menu';
import { isWormholeClass } from '@/const/solarsystemClasses';
import type { TSolarsystem } from '@/pages/maps';
import { Circle, Compass, ExternalLink, Globe, Map } from 'lucide-vue-next';
import { computed } from 'vue';

const { solarsystem } = defineProps<{
    solarsystem: TSolarsystem;
}>();

const systemName = computed(() => solarsystem.name.replaceAll(' ', '_'));
const regionName = computed(() => solarsystem.region?.name.replaceAll(' ', '_'));
const isWormhole = computed(() => isWormholeClass(solarsystem.class));
</script>

<template>
    <ContextMenuSub>
        <ContextMenuSubTrigger>
            <ExternalLink class="size-4" />
            External
        </ContextMenuSubTrigger>
        <ContextMenuSubContent>
            <ContextMenuLabel class="flex items-center gap-2 text-[0.65rem] font-semibold tracking-wider text-muted-foreground uppercase">
                <img src="https://evemaps.dotlan.net/favicon.ico" alt="" class="size-3.5 rounded-sm" />
                Dotlan
            </ContextMenuLabel>
            <ContextMenuItem as-child>
                <a :href="`https://evemaps.dotlan.net/system/${systemName}`" target="_blank" rel="noopener">
                    <Globe class="size-4" />
                    System
                </a>
            </ContextMenuItem>
            <ContextMenuItem as-child>
                <a :href="`https://evemaps.dotlan.net/map/${regionName}/${systemName}`" target="_blank" rel="noopener">
                    <Map class="size-4" />
                    Region Map
                </a>
            </ContextMenuItem>
            <ContextMenuItem as-child v-if="!isWormhole">
                <a :href="`https://evemaps.dotlan.net/range/Revelation,5/${systemName}`" target="_blank" rel="noopener">
                    <Circle class="size-4" />
                    Jump Range
                </a>
            </ContextMenuItem>

            <ContextMenuSeparator />

            <ContextMenuLabel class="flex items-center gap-2 text-[0.65rem] font-semibold tracking-wider text-muted-foreground uppercase">
                <img src="https://zkillboard.com/favicon.ico" alt="" class="size-3.5 rounded-sm" />
                zKillboard
            </ContextMenuLabel>
            <ContextMenuItem as-child>
                <a :href="`https://zkillboard.com/system/${solarsystem.id}/`" target="_blank" rel="noopener">
                    <Globe class="size-4" />
                    System
                </a>
            </ContextMenuItem>
            <ContextMenuItem as-child>
                <a :href="`https://zkillboard.com/constellation/${solarsystem.constellation_id}/`" target="_blank" rel="noopener">
                    <Compass class="size-4" />
                    Constellation
                </a>
            </ContextMenuItem>
            <ContextMenuItem as-child>
                <a :href="`https://zkillboard.com/region/${solarsystem.region_id}/`" target="_blank" rel="noopener">
                    <Map class="size-4" />
                    Region
                </a>
            </ContextMenuItem>
        </ContextMenuSubContent>
    </ContextMenuSub>
</template>
