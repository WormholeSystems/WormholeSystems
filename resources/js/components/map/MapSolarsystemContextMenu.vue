<script setup lang="ts">
import { CharacterImage } from '@/components/images';
import CopyNameMenu from '@/components/map/CopyNameMenu.vue';
import {
    ContextMenu,
    ContextMenuContent,
    ContextMenuItem,
    ContextMenuRadioGroup,
    ContextMenuRadioItem,
    ContextMenuSeparator,
    ContextMenuSub,
    ContextMenuSubContent,
    ContextMenuSubTrigger,
    ContextMenuTrigger,
} from '@/components/ui/context-menu';
import { deleteMapSolarsystem, updateMapSolarsystem } from '@/composables/map';
import useHasWritePermission from '@/composables/useHasWritePermission';
import { useNavigationSystems } from '@/composables/useNavigationSystems';
import useUser from '@/composables/useUser';
import { useWaypoint } from '@/composables/useWaypoint';
import { TMapSolarsystem } from '@/pages/maps';
import { TMapSolarsystemStatus } from '@/types/models';

const { map_solarsystem } = defineProps<{
    map_solarsystem: TMapSolarsystem;
}>();

const user = useUser();

const can_write = useHasWritePermission();

const setWaypoint = useWaypoint();

const { setFromSystem, setToSystem } = useNavigationSystems();

function handleTogglePin() {
    updateMapSolarsystem(map_solarsystem, { pinned: !map_solarsystem.pinned });
}

function handleRemoveFromMap() {
    deleteMapSolarsystem(map_solarsystem);
}

function handleStatusChange(status: string) {
    updateMapSolarsystem(map_solarsystem, { status });
}

const options: TMapSolarsystemStatus[] = ['unknown', 'friendly', 'hostile', 'active', 'unscanned', 'empty'];
</script>

<template>
    <ContextMenu>
        <ContextMenuTrigger>
            <slot />
        </ContextMenuTrigger>
        <ContextMenuContent>
            <ContextMenuItem @select="handleTogglePin" v-if="can_write">
                {{ map_solarsystem.pinned ? 'Unpin' : 'Pin' }}
            </ContextMenuItem>
            <ContextMenuSub v-if="can_write">
                <ContextMenuSubTrigger> Status</ContextMenuSubTrigger>
                <ContextMenuSubContent>
                    <ContextMenuRadioGroup :model-value="map_solarsystem.status ?? undefined" @update:model-value="handleStatusChange">
                        <ContextMenuRadioItem v-for="option in options" :key="option" :value="option">
                            {{ option.charAt(0).toUpperCase() + option.slice(1) }}
                            <span
                                :data-status="option"
                                class="ml-auto inline-block size-2 rounded-full data-[status=active]:bg-active data-[status=empty]:bg-empty data-[status=friendly]:bg-friendly data-[status=hostile]:bg-hostile data-[status=unknown]:bg-unknown data-[status=unscanned]:bg-unscanned"
                            />
                        </ContextMenuRadioItem>
                    </ContextMenuRadioGroup>
                </ContextMenuSubContent>
            </ContextMenuSub>

            <ContextMenuSeparator />

            <ContextMenuSub>
                <ContextMenuSubTrigger>External</ContextMenuSubTrigger>
                <ContextMenuSubContent>
                    <ContextMenuItem as-child>
                        <a
                            :href="`https://evemaps.dotlan.net/map/${map_solarsystem.solarsystem?.region?.name.replaceAll(' ', '_')}/${map_solarsystem.solarsystem?.name.replaceAll(' ', '_')}`"
                            target="_blank"
                            rel="noopener noreferrer"
                        >
                            Dotlan
                        </a>
                    </ContextMenuItem>
                    <ContextMenuItem as-child>
                        <a :href="`https://zkillboard.com/system/${map_solarsystem.solarsystem?.id}/`" target="_blank" rel="noopener noreferrer">
                            zKillboard
                        </a>
                    </ContextMenuItem>
                </ContextMenuSubContent>
            </ContextMenuSub>
            <ContextMenuSub v-if="!map_solarsystem.solarsystem.class">
                <ContextMenuSubTrigger>Set destination</ContextMenuSubTrigger>
                <ContextMenuSubContent>
                    <ContextMenuItem
                        v-for="character in user.characters"
                        :key="character.id"
                        @select="setWaypoint(character.id, map_solarsystem.solarsystem_id)"
                    >
                        <CharacterImage :character_id="character.id" :character_name="character.name" class="size-5 rounded-lg" />
                        {{ character.name }}
                    </ContextMenuItem>
                </ContextMenuSubContent>
            </ContextMenuSub>

            <ContextMenuSub v-if="!map_solarsystem.solarsystem.class">
                <ContextMenuSubTrigger>Add waypoint</ContextMenuSubTrigger>
                <ContextMenuSubContent>
                    <ContextMenuItem
                        v-for="character in user.characters"
                        :key="character.id"
                        @select="setWaypoint(character.id, map_solarsystem.solarsystem_id, false)"
                    >
                        <CharacterImage :character_id="character.id" :character_name="character.name" class="size-5 rounded-lg" />
                        {{ character.name }}
                    </ContextMenuItem>
                </ContextMenuSubContent>
            </ContextMenuSub>
            <ContextMenuSeparator />
            <CopyNameMenu :map_solarsystem="map_solarsystem" />
            <template v-if="!map_solarsystem.pinned && can_write">
                <ContextMenuSeparator />
                <ContextMenuItem @select="handleRemoveFromMap"> Remove </ContextMenuItem>
            </template>

            <ContextMenuSeparator />

            <ContextMenuSub>
                <ContextMenuSubTrigger>Route planner</ContextMenuSubTrigger>
                <ContextMenuSubContent>
                    <ContextMenuItem @select="setFromSystem(map_solarsystem.solarsystem_id)">Set as origin</ContextMenuItem>
                    <ContextMenuItem @select="setToSystem(map_solarsystem.solarsystem_id)">Set as destination</ContextMenuItem>
                </ContextMenuSubContent>
            </ContextMenuSub>
        </ContextMenuContent>
    </ContextMenu>
</template>

<style scoped></style>
