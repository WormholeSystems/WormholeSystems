<script setup lang="ts">
import Character from '@/components/characters/Character.vue';
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card';
import { Table, TableCell, TableHead, TableHeader, TableRow } from '@/components/ui/table';
import { TCharacter } from '@/types/models';
import { computed } from 'vue';

const { map_characters } = defineProps<{
    map_characters: TCharacter[];
}>();

const sorted_characters = computed(() => map_characters.toSorted(sortCharacters));

function sortCharacters(a: TCharacter, b: TCharacter) {
    if (a.status?.ship_type?.name === 'Capsule' && b.status?.ship_type?.name !== 'Capsule') {
        return 1;
    }
    if (b.status?.ship_type?.name === 'Capsule' && a.status?.ship_type?.name !== 'Capsule') {
        return -1;
    }

    if ((a.status?.station_id || a.status?.structure_id) && !(b.status?.station_id || b.status?.structure_id)) {
        return 1;
    }

    if (!(a.status?.station_id || a.status?.structure_id) && (b.status?.station_id || b.status?.structure_id)) {
        return -1;
    }

    if (isCovertOps(a) && !isCovertOps(b)) {
        return 1;
    }

    if (!isCovertOps(a) && isCovertOps(b)) {
        return -1;
    }

    return a.name.localeCompare(b.name);
}

function isCovertOps(character: TCharacter) {
    return character.status?.ship_type?.group_id === 830;
}
</script>

<template>
    <Card class="pb-0">
        <CardHeader>
            <CardTitle>Characters</CardTitle>
            <CardDescription> See what characters are flying and where they are located in the map solarsystems. </CardDescription>
        </CardHeader>
        <CardContent class="px-1 pb-1">
            <div class="rounded-lg border bg-neutral-900/40">
                <Table>
                    <TableHeader>
                        <TableRow>
                            <TableHead> Pilot</TableHead>
                            <TableHead> Ship</TableHead>
                            <TableHead> Location</TableHead>
                        </TableRow>
                    </TableHeader>
                    <TransitionGroup name="list">
                        <Character v-for="character in sorted_characters" :key="character.id" :character />
                    </TransitionGroup>
                    <TableRow v-if="!sorted_characters?.length">
                        <TableCell colspan="3" class="text-center text-muted-foreground"> No characters found </TableCell>
                    </TableRow>
                </Table>
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
