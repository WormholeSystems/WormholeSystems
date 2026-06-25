<script setup lang="ts">
import DestinationContextMenu from '@/components/autopilot/DestinationContextMenu.vue';
import RoutePopover from '@/components/autopilot/RoutePopover.vue';
import CharacterView from '@/components/characters/CharacterView.vue';
import SolarsystemSovereignty from '@/components/map/SolarsystemSovereignty.vue';
import SolarsystemEffect from '@/components/solarsystem/SolarsystemEffect.vue';
import { useMapSolarsystems } from '@/composables/map';
import { usePath } from '@/composables/usePath';
import { TCharacter } from '@/types/models';
import type { TStaticSolarsystem } from '@/types/static-data';
import { computed } from 'vue';

const { character, static_solarsystem } = defineProps<{
    character: TCharacter;
    static_solarsystem: TStaticSolarsystem | null;
}>();

const { map_solarsystems, setHoveredMapSolarsystem } = useMapSolarsystems();
const { setPath } = usePath();

const map_solarsystem = computed(() => {
    return map_solarsystems.value.find((solarsystem) => solarsystem.solarsystem_id === character.status?.solarsystem_id);
});

// Hovering the row highlights both the system and the route to it on the map.
function onHover(hovered: boolean) {
    if (map_solarsystem.value) {
        setHoveredMapSolarsystem(map_solarsystem.value.id, hovered);
    }
    setPath(hovered ? (character.route ?? null) : null);
}
</script>

<template>
    <div class="contents">
        <DestinationContextMenu :solarsystem_id="character.status?.solarsystem_id ?? 0">
            <CharacterView :character="character" :static_solarsystem="static_solarsystem" :alias="map_solarsystem?.alias ?? null" @hover="onHover">
                <template #sovereignty>
                    <SolarsystemSovereignty
                        v-if="static_solarsystem"
                        :sovereignty="static_solarsystem.sovereignty"
                        :solarsystem-id="static_solarsystem.id"
                        class="size-4"
                    >
                        <template #fallback>
                            <SolarsystemEffect v-if="static_solarsystem.effect" :effect="static_solarsystem.effect.name" />
                        </template>
                    </SolarsystemSovereignty>
                </template>
                <template #jumps="{ nextHop, jumpCount, jumpClass }">
                    <RoutePopover :route="character.route">
                        <button class="flex min-w-0 cursor-pointer items-center gap-1.5 justify-self-end hover:text-foreground">
                            <span v-if="nextHop" class="truncate text-[10px] text-muted-foreground">{{ nextHop.name }}</span>
                            <span v-if="jumpCount !== null" class="shrink-0 font-mono text-xs font-medium" :class="jumpClass">{{ jumpCount }}j</span>
                            <span v-else class="font-mono text-[10px] text-muted-foreground/60">--</span>
                        </button>
                    </RoutePopover>
                </template>
            </CharacterView>
        </DestinationContextMenu>
    </div>
</template>

<style scoped></style>
