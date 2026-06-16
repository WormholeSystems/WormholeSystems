<script setup lang="ts">
import CharacterView from '@/components/characters/CharacterView.vue';
import { TCharacter } from '@/types/models';
import type { TStaticSolarsystem } from '@/types/static-data';

export type TCharacterViewModel = TCharacter & {
    static_solarsystem: TStaticSolarsystem | null;
    alias?: string | null;
};

const { characters } = defineProps<{
    characters: TCharacterViewModel[];
}>();
</script>

<template>
    <template v-if="characters.length">
        <div class="grid grid-cols-[auto_minmax(0,1fr)_minmax(0,1fr)_auto_minmax(0,1fr)_auto_minmax(0,1fr)] text-sm">
            <div
                class="col-span-full grid grid-cols-subgrid items-center gap-2 border-b border-border/50 bg-muted/50 px-3 py-1.5 font-mono text-[10px] tracking-wider text-muted-foreground uppercase"
            >
                <div></div>
                <div>Pilot</div>
                <div>Ship</div>
                <div></div>
                <div>Location</div>
                <div></div>
                <div class="text-right">Jumps</div>
            </div>
            <TransitionGroup name="list">
                <CharacterView
                    v-for="character in characters"
                    :key="character.id"
                    :character="character"
                    :static_solarsystem="character.static_solarsystem"
                    :alias="character.alias ?? null"
                />
            </TransitionGroup>
        </div>
    </template>
    <div v-else class="flex h-full flex-col items-center justify-center gap-2 p-4">
        <p class="font-mono text-[10px] tracking-wider text-muted-foreground/60 uppercase">No pilots online</p>
    </div>
</template>

<style scoped>
.list-move,
.list-enter-active,
.list-leave-active {
    transition: all 0.5s ease;
}

.list-enter-from,
.list-leave-to {
    opacity: 0;
    transform: translateX(30px);
}

.list-leave-active {
    position: absolute;
    opacity: 0;
    transition-duration: 0ms;
}
</style>
