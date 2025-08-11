<script setup lang="ts">
import DestinationContextMenu from '@/components/DestinationContextMenu.vue';
import TimesIcon from '@/components/icons/TimesIcon.vue';
import SolarsystemSovereignty from '@/components/map/SolarsystemSovereignty.vue';
import SolarsystemClass from '@/components/SolarsystemClass.vue';
import SolarsystemEffect from '@/components/SolarsystemEffect.vue';
import { Button } from '@/components/ui/button';
import { Popover, PopoverContent, PopoverTrigger } from '@/components/ui/popover';
import { Table, TableBody, TableCell, TableHead, TableHeader, TableRow } from '@/components/ui/table';
import { Tooltip, TooltipContent, TooltipTrigger } from '@/components/ui/tooltip';
import { useIgnoreList } from '@/composables/useIgnoreList';
import { usePath } from '@/composables/usePath';
import { TSolarsystem } from '@/types/models';
import { vElementHover } from '@vueuse/components';
import { PopoverClose } from 'reka-ui';
import { computed } from 'vue';

interface Props {
    route?: TSolarsystem[];
}

const props = defineProps<Props>();

const { ignoreSolarsystem, clearIgnoreList, ignored_systems } = useIgnoreList();

const { setPath } = usePath();

const hasRoute = computed(() => props.route && props.route.length > 0);

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
        <PopoverTrigger as-child>
            <slot />
        </PopoverTrigger>
        <PopoverContent class="w-96 p-1">
            <div class="" v-element-hover="onHover">
                <div class="grid gap-2 p-3">
                    <div class="flex justify-between">
                        <span class="">Route</span>

                        <PopoverClose as-child>
                            <Button variant="ghost">
                                <TimesIcon />
                            </Button>
                        </PopoverClose>
                    </div>
                    <div class="text-xs text-muted-foreground">
                        This route is calculated based on the shortest path through the available systems.
                    </div>
                    <template v-if="ignored_systems.length">
                        <div class="text-xs text-muted-foreground">
                            You are ignoring
                            <span class="font-medium">{{ ignored_systems.length }}</span>
                            systems in this route.
                        </div>
                        <Button variant="secondary" size="sm" @click="handleClearIgnoreList">Clear ignored systems </Button>
                    </template>
                </div>
                <div class="m-2 max-h-96 overflow-y-auto rounded-md border bg-muted/20" v-if="hasRoute">
                    <Table>
                        <TableHeader>
                            <TableRow class="text-xs">
                                <TableHead class="h-auto w-8 p-1"></TableHead>
                                <TableHead class="h-auto p-1">Name</TableHead>
                                <TableHead class="h-auto p-1">Region</TableHead>
                                <TableHead class="h-auto w-8 p-1"></TableHead>
                                <TableHead class="h-auto w-8 p-1"></TableHead>
                            </TableRow>
                        </TableHeader>
                        <TableBody>
                            <DestinationContextMenu v-for="(solarsystem, index) in route" :key="index" :solarsystem_id="solarsystem.id">
                                <TableRow class="text-xs">
                                    <TableCell class="h-auto p-1">
                                        <div class="flex justify-center">
                                            <SolarsystemClass :wormhole_class="solarsystem.class" :security="solarsystem.security" />
                                        </div>
                                    </TableCell>

                                    <TableCell class="h-auto p-1">
                                        <DestinationContextMenu :solarsystem_id="solarsystem.id">
                                            <span class="cursor-pointer font-medium hover:text-accent-foreground">
                                                {{ solarsystem.name }}
                                            </span>
                                        </DestinationContextMenu>
                                    </TableCell>
                                    <TableCell class="h-auto p-1 text-muted-foreground">
                                        {{ solarsystem.region?.name }}
                                    </TableCell>
                                    <TableCell class="h-auto p-1">
                                        <div class="flex justify-center">
                                            <SolarsystemSovereignty v-if="solarsystem.sovereignty" :sovereignty="solarsystem.sovereignty" />
                                            <SolarsystemEffect v-else-if="solarsystem.effect" :effect="solarsystem.effect" />
                                        </div>
                                    </TableCell>
                                    <TableCell class="h-auto p-1">
                                        <template v-if="route && index !== 0 && index !== route.length - 1">
                                            <Tooltip>
                                                <TooltipTrigger as-child>
                                                    <Button
                                                        variant="secondary"
                                                        size="icon"
                                                        class="h-6 w-6"
                                                        @click="handleIgnoreSolarsystem(solarsystem.id)"
                                                    >
                                                        <TimesIcon class="h-2.5 w-2.5" />
                                                    </Button>
                                                </TooltipTrigger>
                                                <TooltipContent> Ignore this system</TooltipContent>
                                            </Tooltip>
                                        </template>
                                    </TableCell>
                                </TableRow>
                            </DestinationContextMenu>
                        </TableBody>
                    </Table>
                </div>
                <div v-else class="p-3 text-sm text-muted-foreground">
                    <span class="text-xs">No route available</span>
                </div>
            </div>
        </PopoverContent>
    </Popover>
</template>

<style scoped></style>
