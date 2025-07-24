<script setup lang="ts">
import LockIcon from '@/components/icons/LockIcon.vue';
import TrashIcon from '@/components/icons/TrashIcon.vue';
import { CharacterImage } from '@/components/images';
import SolarsystemClass from '@/components/SolarsystemClass.vue';
import { Button } from '@/components/ui/button';
import {
    ContextMenu,
    ContextMenuContent,
    ContextMenuItem,
    ContextMenuSeparator,
    ContextMenuSub,
    ContextMenuSubContent,
    ContextMenuSubTrigger,
    ContextMenuTrigger,
} from '@/components/ui/context-menu';
import { TableCell, TableRow } from '@/components/ui/table';
import { useHasWritePermission } from '@/composables/useHasPermission';
import { usePath } from '@/composables/usePath';
import useUser from '@/composables/useUser';
import { useWaypoint } from '@/composables/useWaypoint';
import { TMapRouteSolarsystem } from '@/types/models';
import { router } from '@inertiajs/vue3';
import { vElementHover } from '@vueuse/components';

const { map_route } = defineProps<{
    map_route: TMapRouteSolarsystem;
}>();

const { setPath } = usePath();

const setWaypoint = useWaypoint();

const user = useUser();

const can_write = useHasWritePermission();

function onHover(route: TMapRouteSolarsystem, hovered: boolean) {
    if (hovered) {
        setPath(route.route);
    } else {
        setPath(null);
    }
}

function togglePinned() {
    router.put(
        route('map-route-solarsystems.update', map_route.id),
        {
            is_pinned: !map_route.is_pinned,
        },
        {
            preserveScroll: true,
            preserveState: true,
            only: ['map_route_solarsystems'],
        },
    );
}

function removeRoute() {
    router.delete(route('map-route-solarsystems.destroy', map_route.id), {
        preserveScroll: true,
        preserveState: true,
        only: ['map_route_solarsystems'],
    });
}
</script>

<template>
    <ContextMenu>
        <ContextMenuTrigger as-child>
            <TableRow v-element-hover="(hovered) => onHover(map_route, hovered)" class="group">
                <TableCell>
                    <SolarsystemClass :wormhole_class="map_route.solarsystem.class" :security="map_route.solarsystem.security" />
                    {{ map_route.solarsystem.name }}
                </TableCell>
                <TableCell>
                    <span class="text-xs text-muted-foreground">
                        {{ map_route.route.length ? map_route.route.length - 1 : 'N/A' }}
                    </span>
                </TableCell>
                <TableCell v-if="can_write">
                    <Button
                        variant="ghost"
                        size="icon"
                        @click="togglePinned"
                        :data-pinned="map_route.is_pinned"
                        class="data-[pinned=false]:opacity-0 group-hover:data-[pinned=false]:opacity-100"
                    >
                        <span class="sr-only">Toggle Pin</span>
                        <LockIcon />
                    </Button>
                    <Button variant="ghost" size="icon" @click="removeRoute" class="opacity-0 group-hover:opacity-100">
                        <span class="sr-only">Remove Route</span>
                        <TrashIcon />
                    </Button>
                </TableCell>
            </TableRow>
        </ContextMenuTrigger>
        <ContextMenuContent>
            <ContextMenuSub>
                <ContextMenuSubTrigger>Set destination</ContextMenuSubTrigger>
                <ContextMenuSubContent>
                    <ContextMenuItem
                        v-for="character in user.characters"
                        :key="character.id"
                        @select="setWaypoint(character.id, map_route.solarsystem.id)"
                    >
                        <CharacterImage :character_id="character.id" :character_name="character.name" class="size-5 rounded-lg" />
                        {{ character.name }}
                    </ContextMenuItem>
                </ContextMenuSubContent>
            </ContextMenuSub>

            <ContextMenuSub>
                <ContextMenuSubTrigger>Add waypoint</ContextMenuSubTrigger>
                <ContextMenuSubContent>
                    <ContextMenuItem
                        v-for="character in user.characters"
                        :key="character.id"
                        @select="setWaypoint(character.id, map_route.solarsystem.id, false)"
                    >
                        <CharacterImage :character_id="character.id" :character_name="character.name" class="size-5 rounded-lg" />
                        {{ character.name }}
                    </ContextMenuItem>
                </ContextMenuSubContent>
            </ContextMenuSub>
            <ContextMenuSeparator />
        </ContextMenuContent>
    </ContextMenu>
</template>

<style scoped></style>
