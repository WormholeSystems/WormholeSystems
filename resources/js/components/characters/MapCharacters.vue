<script setup lang="ts">
import Character from '@/components/characters/Character.vue';
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card';
import { getMapChannelName } from '@/const/channels';
import { CharacterStatusUpdatedEvent } from '@/const/events';
import { TCharacter } from '@/types/models';
import { router } from '@inertiajs/vue3';
import { useEchoPublic } from '@laravel/echo-vue';

const { map_characters, map_id } = defineProps<{
    map_characters: TCharacter[];
    map_id: number;
}>();

useEchoPublic(getMapChannelName(map_id), CharacterStatusUpdatedEvent, () => {
    router.reload({
        only: ['map_characters'],
    });
});
</script>

<template>
    <Card>
        <CardHeader>
            <CardTitle>Characters</CardTitle>
            <CardDescription> See what characters are flying and where they are located in the map solarsystems. </CardDescription>
        </CardHeader>
        <CardContent>
            <div
                class="relative grid h-100 grid-cols-[auto_1fr_auto_auto_auto] content-start overflow-x-hidden overflow-y-scroll mask-b-from-90% mask-alpha pr-4 pb-8"
            >
                <TransitionGroup name="list">
                    <Character v-for="character in map_characters" :key="character.id" :character="character" />
                </TransitionGroup>
                <div v-if="map_characters?.length === 0" class="text-center text-sm text-muted-foreground">
                    No characters found on the map. It might take a while for characters to appear after they have been logged in.
                </div>
            </div>
        </CardContent>
    </Card>
</template>

<style scoped>
.list-move, /* apply transition to moving elements */
.list-enter-active,
.list-leave-active {
    transition: all 0.5s ease;
}

.list-enter-from,
.list-leave-to {
    opacity: 0;
    transform: translateX(30px);
}

/* ensure leaving items are taken out of layout flow so that moving
   animations can be calculated correctly. */
.list-leave-active {
    position: absolute;
    opacity: 0;
    transition-duration: 0ms;
}
</style>
