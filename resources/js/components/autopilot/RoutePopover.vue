<script setup lang="ts">
import DestinationContextMenu from '@/components/autopilot/DestinationContextMenu.vue';
import ExtraWormholeIcon from '@/components/icons/ExtraWormholeIcon.vue';
import { CharacterImage } from '@/components/images';
import SolarsystemSearchResult from '@/components/solarsystem/SolarsystemSearchResult.vue';
import { Button } from '@/components/ui/button';
import { DropdownMenu, DropdownMenuContent, DropdownMenuItem, DropdownMenuSeparator, DropdownMenuTrigger } from '@/components/ui/dropdown-menu';
import { Popover, PopoverContent, PopoverTrigger } from '@/components/ui/popover';
import { Tooltip, TooltipContent, TooltipTrigger } from '@/components/ui/tooltip';
import { useIgnoreList } from '@/composables/useIgnoreList';
import { usePath } from '@/composables/usePath';
import { useSolarsystemAliases } from '@/composables/useSolarsystemAliases';
import useUser from '@/composables/useUser';
import { useWaypoint } from '@/composables/useWaypoint';
import { useMapSolarsystems } from '@/map/api';
import type { TResolvedSolarsystem } from '@/pages/maps';
import { vElementHover } from '@vueuse/components';
import { Navigation, Users, X } from 'lucide-vue-next';
import { computed } from 'vue';

interface Props {
    route?: TResolvedSolarsystem[];
}

const props = defineProps<Props>();

const { ignoreSolarsystem, clearIgnoreList, ignored_systems } = useIgnoreList();
const { map_solarsystems } = useMapSolarsystems();
const { getAlias } = useSolarsystemAliases(map_solarsystems);
const { setPath } = usePath();
const { setWaypoint, setWaypointAll, onlineCharacters } = useWaypoint();
const user = useUser();

const hasRoute = computed(() => props.route && props.route.length > 0);
const jumpCount = computed(() => (props.route ? props.route.length - 1 : 0));
const destination = computed(() => (props.route && props.route.length > 0 ? props.route[props.route.length - 1] : null));

function handleSetDestination(characterId: number) {
    if (destination.value) {
        setWaypoint(characterId, destination.value.id);
    }
}

function handleSetDestinationAll() {
    if (!destination.value) return;
    setWaypointAll(destination.value.id);
}

function handleIgnoreSolarsystem(solarsystem_id: number) {
    ignoreSolarsystem(solarsystem_id, {
        onSuccess() {
            setPath(props.route!);
        },
    });
}

function handleClearIgnoreList() {
    clearIgnoreList({
        onSuccess() {
            setPath(props.route!);
        },
    });
}

function onHover(hovered: boolean) {
    if (hovered && hasRoute.value) {
        setPath(props.route!);
    } else {
        setPath(null);
    }
}
</script>

<template>
    <Popover>
        <PopoverTrigger class="justify-self-end">
            <slot />
        </PopoverTrigger>
        <PopoverContent class="w-72 p-0" align="end" side="bottom">
            <div v-element-hover="onHover">
                <!-- Header -->
                <div class="flex items-center justify-between gap-2 border-b border-border/50 bg-muted/30 px-3 py-2">
                    <div class="flex min-w-0 items-baseline gap-2">
                        <span class="font-mono text-[10px] tracking-wider text-muted-foreground uppercase">Route</span>
                        <span v-if="hasRoute" class="font-mono text-xs font-medium whitespace-nowrap">{{ jumpCount }} jumps</span>
                    </div>
                    <DropdownMenu v-if="user && destination">
                        <DropdownMenuTrigger as-child>
                            <Button variant="secondary" size="sm" class="h-6 shrink-0 gap-1 px-2 text-[10px] whitespace-nowrap">
                                <Navigation class="size-3" />
                                Set Destination
                            </Button>
                        </DropdownMenuTrigger>
                        <DropdownMenuContent align="end" class="w-48">
                            <template v-if="onlineCharacters.length">
                                <DropdownMenuItem
                                    v-for="character in onlineCharacters"
                                    :key="character.id"
                                    @select="handleSetDestination(character.id)"
                                    class="gap-2 text-xs"
                                >
                                    <CharacterImage :character_id="character.id" :character_name="character.name" class="size-5 rounded" />
                                    {{ character.name }}
                                </DropdownMenuItem>
                                <DropdownMenuSeparator v-if="onlineCharacters.length > 1" />
                                <DropdownMenuItem v-if="onlineCharacters.length > 1" @select="handleSetDestinationAll" class="gap-2 text-xs">
                                    <Users class="size-4" />
                                    All Characters
                                </DropdownMenuItem>
                            </template>
                            <DropdownMenuItem v-else disabled class="text-xs">No characters online</DropdownMenuItem>
                        </DropdownMenuContent>
                    </DropdownMenu>
                </div>

                <!-- Route List -->
                <div v-if="hasRoute" class="grid max-h-64 grid-cols-[auto_minmax(0,1fr)_minmax(0,1fr)_auto_auto_auto] overflow-y-auto">
                    <DestinationContextMenu v-for="(solarsystem, index) in route" :key="index" :solarsystem_id="solarsystem.id">
                        <div
                            class="col-span-6 grid grid-cols-subgrid items-center gap-x-1.5 border-b border-border/30 px-3 py-0.5 last:border-0 hover:bg-muted/30"
                        >
                            <SolarsystemSearchResult :solarsystem="solarsystem" :alias="getAlias(solarsystem.id)" />
                            <Tooltip
                                v-if="
                                    route &&
                                    index < route.length - 1 &&
                                    (route[index].connection_type === 'wormhole' || route[index].connection_type === 'evescout')
                                "
                            >
                                <TooltipTrigger as-child>
                                    <ExtraWormholeIcon
                                        class="size-3.5"
                                        :class="route[index].connection_type === 'evescout' ? 'text-blue-400' : 'text-amber-500'"
                                    />
                                </TooltipTrigger>
                                <TooltipContent side="left">
                                    {{ route[index].connection_type === 'evescout' ? 'EVE Scout' : 'Wormhole' }}
                                </TooltipContent>
                            </Tooltip>
                            <span v-else class="size-3.5"></span>
                            <button
                                v-if="route && index !== 0 && index !== route.length - 1"
                                class="text-muted-foreground/40 hover:text-destructive"
                                @click.stop="handleIgnoreSolarsystem(solarsystem.id)"
                            >
                                <X class="size-3" />
                            </button>
                            <span v-else class="size-3"></span>
                        </div>
                    </DestinationContextMenu>
                </div>

                <!-- Empty State -->
                <div v-else class="flex items-center justify-center p-4">
                    <span class="font-mono text-[10px] tracking-wider text-muted-foreground/60 uppercase">No route available</span>
                </div>

                <!-- Ignored Systems -->
                <div v-if="ignored_systems.length" class="flex items-center justify-between border-t border-border/50 bg-muted/30 px-3 py-1.5">
                    <span class="font-mono text-[10px] tracking-wider text-muted-foreground uppercase">
                        {{ ignored_systems.length }} {{ ignored_systems.length === 1 ? 'system' : 'systems' }} ignored
                    </span>
                    <button @click="handleClearIgnoreList" class="text-[10px] whitespace-nowrap text-muted-foreground hover:text-foreground">
                        Clear
                    </button>
                </div>
            </div>
        </PopoverContent>
    </Popover>
</template>
