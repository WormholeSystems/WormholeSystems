<script setup lang="ts">
import DestinationContextMenu from '@/components/DestinationContextMenu.vue';
import TimesIcon from '@/components/icons/TimesIcon.vue';
import SovereigntyIcon from '@/components/map/SovereigntyIcon.vue';
import SolarsystemClass from '@/components/SolarsystemClass.vue';
import { Button } from '@/components/ui/button';
import { Popover, PopoverContent, PopoverTrigger } from '@/components/ui/popover';
import { Table, TableBody, TableCell, TableHead, TableHeader, TableRow } from '@/components/ui/table';
import { Tooltip, TooltipContent, TooltipTrigger } from '@/components/ui/tooltip';
import { useIgnoreList } from '@/composables/useIgnoreList';
import { usePath } from '@/composables/usePath';
import { TSolarsystem } from '@/types/models';
import { vElementHover } from '@vueuse/components';
import { computed } from 'vue';

interface Props {
    route?: TSolarsystem[];
}

const props = defineProps<Props>();

const { isIgnored, ignoreSystem, unignoreSystem, clearIgnoreList, ignoredSystems } = useIgnoreList();

const { setPath } = usePath();

const hasRoute = computed(() => props.route && props.route.length > 0);

const handleIgnoreSystem = (systemId: number) => {
    if (isIgnored(systemId)) {
        unignoreSystem(systemId);
    } else {
        ignoreSystem(systemId);
    }
};

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
        <PopoverContent class="w-96 p-0">
            <div class="" v-element-hover="onHover" :key="route">
                <div class="grid gap-2 p-3">
                    <span class="">Route</span>

                    <div class="text-xs text-muted-foreground">
                        This route is calculated based on the shortest path through the available systems.
                    </div>
                    <template v-if="ignoredSystems.length">
                        <div class="text-xs text-muted-foreground">
                            You are ignoring
                            <span class="font-medium">{{ ignoredSystems.length }}</span>
                            systems in this route.
                        </div>
                        <Button variant="outline" size="sm" @click="clearIgnoreList">Clear ignored systems</Button>
                    </template>
                </div>

                <!-- Route Table -->
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
                                <TableRow class="text-xs" :class="{ 'opacity-50': isIgnored(solarsystem.id) }">
                                    <!-- Security/Class Column -->
                                    <TableCell class="h-auto p-1">
                                        <div class="flex justify-center">
                                            <SolarsystemClass :wormhole_class="solarsystem.class" :security="solarsystem.security" />
                                        </div>
                                    </TableCell>

                                    <!-- Name Column -->
                                    <TableCell class="h-auto p-1">
                                        <DestinationContextMenu :solarsystem_id="solarsystem.id">
                                            <span class="cursor-pointer font-medium hover:text-accent-foreground">
                                                {{ solarsystem.name }}
                                            </span>
                                        </DestinationContextMenu>
                                    </TableCell>

                                    <!-- Region Column -->
                                    <TableCell class="h-auto p-1 text-muted-foreground">
                                        {{ solarsystem.region?.name }}
                                    </TableCell>

                                    <!-- Sovereignty/Effect Column -->
                                    <TableCell class="h-auto p-1">
                                        <div class="flex justify-center">
                                            <SovereigntyIcon v-if="solarsystem.sovereignty" :sovereignty="solarsystem.sovereignty" />
                                            <span
                                                v-else-if="solarsystem.effect"
                                                class="rounded bg-muted px-1 py-0.5 text-xs text-muted-foreground"
                                                :title="solarsystem.effect"
                                            >
                                                {{ solarsystem.effect.charAt(0) }}
                                            </span>
                                        </div>
                                    </TableCell>

                                    <!-- Option Column -->
                                    <TableCell class="h-auto p-1">
                                        <Tooltip v-if="index > 0 && index < route!.length - 1">
                                            <TooltipTrigger>
                                                <Button variant="outline" size="icon" class="h-6 w-6" @click="handleIgnoreSystem(solarsystem.id)">
                                                    <TimesIcon v-if="!isIgnored(solarsystem.id)" class="h-2.5 w-2.5" />
                                                    <span v-else class="text-xs">↺</span>
                                                </Button>
                                            </TooltipTrigger>
                                            <TooltipContent>
                                                {{ isIgnored(solarsystem.id) ? 'Stop ignoring this system' : 'Ignore this system' }}
                                            </TooltipContent>
                                        </Tooltip>
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
