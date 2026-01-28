<script setup lang="ts">
import DestinationContextMenu from '@/components/autopilot/DestinationContextMenu.vue';
import ExtraWormholeIcon from '@/components/icons/ExtraWormholeIcon.vue';
import SolarsystemSovereignty from '@/components/map/SolarsystemSovereignty.vue';
import SolarsystemClass from '@/components/solarsystem/SolarsystemClass.vue';
import SolarsystemEffect from '@/components/solarsystem/SolarsystemEffect.vue';
import { Button } from '@/components/ui/button';
import { Popover, PopoverContent, PopoverTrigger } from '@/components/ui/popover';
import { Tooltip, TooltipContent, TooltipTrigger } from '@/components/ui/tooltip';
import { useActiveMapCharacter } from '@/composables/useActiveMapCharacter';
import { useIgnoreList } from '@/composables/useIgnoreList';
import { usePath } from '@/composables/usePath';
import { useWaypoint } from '@/composables/useWaypoint';
import type { TResolvedSolarsystem } from '@/pages/maps';
import { vElementHover } from '@vueuse/components';
import { Navigation, X } from 'lucide-vue-next';
import { computed } from 'vue';

interface Props {
    route?: TResolvedSolarsystem[];
}

const props = defineProps<Props>();

const { ignoreSolarsystem, clearIgnoreList, ignored_systems } = useIgnoreList();
const { setPath } = usePath();
const setWaypoint = useWaypoint();
const character = useActiveMapCharacter();

const hasRoute = computed(() => props.route && props.route.length > 0);
const jumpCount = computed(() => (props.route ? props.route.length - 1 : 0));
const destination = computed(() => (props.route && props.route.length > 0 ? props.route[props.route.length - 1] : null));

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

function handleSetDestination() {
    if (character.value && destination.value) {
        setWaypoint(character.value.id, destination.value.id);
    }
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
                <div class="flex items-center justify-between border-b border-border/50 bg-muted/30 px-3 py-2">
                    <div class="flex items-center gap-2">
                        <span class="font-mono text-[10px] tracking-wider text-muted-foreground uppercase">Route</span>
                        <span v-if="hasRoute" class="font-mono text-xs font-medium">{{ jumpCount }} jumps</span>
                    </div>
                    <div class="flex items-center gap-2">
                        <button
                            v-if="ignored_systems.length"
                            @click="handleClearIgnoreList"
                            class="text-[10px] text-muted-foreground hover:text-foreground"
                        >
                            Clear {{ ignored_systems.length }} ignored
                        </button>
                        <Tooltip v-if="character && destination">
                            <TooltipTrigger as-child>
                                <Button variant="secondary" size="sm" class="h-6 gap-1 px-2 text-[10px]" @click="handleSetDestination">
                                    <Navigation class="size-3" />
                                    Set Destination
                                </Button>
                            </TooltipTrigger>
                            <TooltipContent side="bottom">Set in-game autopilot destination</TooltipContent>
                        </Tooltip>
                    </div>
                </div>

                <!-- Route List -->
                <div v-if="hasRoute" class="max-h-64 overflow-y-auto">
                    <DestinationContextMenu v-for="(solarsystem, index) in route" :key="index" :solarsystem_id="solarsystem.id">
                        <div class="flex items-center gap-1.5 border-b border-border/30 px-3 py-0.5 last:border-0 hover:bg-muted/30">
                            <SolarsystemClass :wormhole_class="solarsystem.class" :security="solarsystem.security" class="shrink-0" />
                            <span class="min-w-0 flex-1 truncate text-xs">{{ solarsystem.name }}</span>
                            <span class="shrink-0 truncate text-[10px] text-muted-foreground">{{ solarsystem.region?.name }}</span>
                            <SolarsystemSovereignty :sovereignty="solarsystem.sovereignty" :solarsystem-id="solarsystem.id" class="size-4 shrink-0">
                                <template #fallback>
                                    <SolarsystemEffect v-if="solarsystem.effect" :effect="solarsystem.effect.name" />
                                </template>
                            </SolarsystemSovereignty>
                            <Tooltip v-if="route && index < route.length - 1 && route[index + 1].connection_type === 'wormhole'">
                                <TooltipTrigger as-child>
                                    <ExtraWormholeIcon class="size-3.5 shrink-0 text-amber-500" />
                                </TooltipTrigger>
                                <TooltipContent side="left">Wormhole</TooltipContent>
                            </Tooltip>
                            <span v-else class="size-3.5 shrink-0"></span>
                            <button
                                v-if="route && index !== 0 && index !== route.length - 1"
                                class="shrink-0 text-muted-foreground/40 hover:text-destructive"
                                @click.stop="handleIgnoreSolarsystem(solarsystem.id)"
                            >
                                <X class="size-3" />
                            </button>
                            <span v-else class="size-3 shrink-0"></span>
                        </div>
                    </DestinationContextMenu>
                </div>

                <!-- Empty State -->
                <div v-else class="flex items-center justify-center p-4">
                    <span class="font-mono text-[10px] tracking-wider text-muted-foreground/60 uppercase">No route available</span>
                </div>
            </div>
        </PopoverContent>
    </Popover>
</template>
