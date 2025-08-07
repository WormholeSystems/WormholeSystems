<script setup lang="ts">
import DestinationContextMenu from '@/components/DestinationContextMenu.vue';
import TypeImage from '@/components/images/TypeImage.vue';
import Affiliation from '@/components/killmails/Affiliation.vue';
import AttackerImage from '@/components/killmails/AttackerImage.vue';
import VictimImage from '@/components/killmails/VictimImage.vue';
import SolarsystemSovereignty from '@/components/map/SolarsystemSovereignty.vue';
import SolarsystemClass from '@/components/SolarsystemClass.vue';
import { TableCell, TableRow } from '@/components/ui/table';
import { useMapSolarsystems } from '@/composables/map';
import { formatISK } from '@/lib/utils';
import { TKillmail } from '@/types/models';
import { UTCDate } from '@date-fns/utc';
import { vElementHover } from '@vueuse/components';
import { useNow } from '@vueuse/core';
import { formatDistanceStrict } from 'date-fns';
import { computed } from 'vue';

const { killmail } = defineProps<{
    killmail: TKillmail;
}>();

const { map_solarsystems, setHoveredMapSolarsystem } = useMapSolarsystems();

function onHover(hovered: boolean) {
    if (map_solarsystem.value) {
        setHoveredMapSolarsystem(map_solarsystem.value.id, hovered);
    }
}

const map_solarsystem = computed(() => {
    return map_solarsystems.value.find((solarsystem) => solarsystem.solarsystem_id === killmail.solarsystem.id);
});

const final_blow = computed(() => {
    return killmail.data.attackers.find((attacker) => attacker.final_blow)!;
});

const now = useNow();

const time_ago = computed(() => {
    return formatDistanceStrict(new UTCDate(killmail.time), now.value, {
        addSuffix: true,
    });
});

const total_worth = computed(() => {
    return formatISK(killmail.zkb.totalValue);
});
</script>

<template>
    <DestinationContextMenu :solarsystem_id="killmail.solarsystem.id">
        <TableRow v-element-hover="onHover" class="cursor-pointer">
            <TableCell>
                <div class="flex gap-x-2">
                    <a :href="`https://zkillboard.com/kill/${killmail.id}/`" target="_blank" rel="noopener noreferrer">
                        <TypeImage class="size-8 rounded-lg" :type_id="killmail.ship_type.id" :type_name="killmail.ship_type.name" />
                    </a>
                    <a :href="`https://zkillboard.com/character/${killmail.data.victim.character_id}/`" target="_blank" rel="noopener noreferrer">
                        <VictimImage class="size-8 overflow-hidden rounded-lg" :victim="killmail.data.victim" />
                    </a>
                    <Affiliation class="size-8 overflow-hidden rounded-lg" alt="Victim group" :affiliation="killmail.data.victim" />
                </div>
            </TableCell>
            <TableCell>
                <div class="flex gap-x-2">
                    <a :href="`https://zkillboard.com/character/${final_blow.character_id}/`" target="_blank" rel="noopener noreferrer">
                        <AttackerImage class="size-8 overflow-hidden rounded-lg" :attacker="final_blow" />
                    </a>
                    <Affiliation class="size-8 overflow-hidden rounded-lg" alt="Attacker group" :affiliation="final_blow" />
                </div>
            </TableCell>
            <TableCell>
                <div class="flex items-center gap-2">
                    <div class="flex w-6 flex-shrink-0 justify-center">
                        <SolarsystemClass :wormhole_class="killmail.solarsystem.class" :security="killmail.solarsystem.security" />
                    </div>
                    <div class="min-w-0 flex-1">
                        <div v-if="map_solarsystem?.alias" class="flex gap-1 font-medium hover:text-accent-foreground">
                            {{ map_solarsystem.alias }}<span class="text-muted-foreground">{{ killmail.solarsystem.name }}</span>
                        </div>
                        <div v-else class="font-medium hover:text-accent-foreground">
                            {{ killmail.solarsystem.name }}
                        </div>
                        <p class="text-xs text-muted-foreground">
                            {{ time_ago }}
                        </p>
                    </div>
                    <SolarsystemSovereignty
                        v-if="killmail.solarsystem.sovereignty"
                        :sovereignty="killmail.solarsystem.sovereignty"
                        class="size-6 flex-shrink-0"
                    />
                </div>
            </TableCell>
            <TableCell class="hidden @lg:table-cell">
                <div class="text-right">
                    <div class="font-mono text-sm font-medium text-primary">
                        {{ total_worth }}
                    </div>
                </div>
            </TableCell>
        </TableRow>
    </DestinationContextMenu>
</template>

<style scoped></style>
