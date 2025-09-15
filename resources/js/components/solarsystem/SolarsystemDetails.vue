<script setup lang="ts">
import StatisticsController from '@/actions/App/Http/Controllers/StatisticsController';
import ExternalIcon from '@/components/icons/ExternalIcon.vue';
import Spinner from '@/components/icons/Spinner.vue';
import WormholeProperties from '@/components/map/connection/WormholeProperties.vue';
import SolarsystemEffect from '@/components/map/SolarsystemEffect.vue';
import SolarsystemSovereignty from '@/components/map/SolarsystemSovereignty.vue';
import Destination from '@/components/solarsystem/Destination.vue';
import SecurityStatus from '@/components/solarsystem/SecurityStatus.vue';
import SolarsystemClass from '@/components/solarsystem/SolarsystemClass.vue';
import { Button } from '@/components/ui/button';
import { CardHeader, CardTitle } from '@/components/ui/card';
import { Dialog, DialogContent, DialogDescription, DialogFooter, DialogHeader, DialogTitle, DialogTrigger } from '@/components/ui/dialog';
import MapPanel from '@/components/ui/map-panel/MapPanel.vue';
import MapPanelContent from '@/components/ui/map-panel/MapPanelContent.vue';
import { Popover, PopoverContent, PopoverTrigger } from '@/components/ui/popover';
import { Textarea } from '@/components/ui/textarea';
import useHasWritePermission from '@/composables/useHasWritePermission';
import MapSolarsystems from '@/routes/map-solarsystems';
import { TMap, TMapRouteSolarsystem, TMapSolarSystem } from '@/types/models';
import { Deferred, Link, useForm } from '@inertiajs/vue3';
import markdownit from 'markdown-it';
import attr from 'markdown-it-link-attributes';
import { computed, ref, watch } from 'vue';

const { map_solarsystem, map_route_solarsystems, hide_notes } = defineProps<{
    map_solarsystem: TMapSolarSystem;
    map_route_solarsystems?: TMapRouteSolarsystem[];
    map: TMap;
    hide_notes?: boolean;
}>();

const can_write = useHasWritePermission();

const pinned = computed(() => map_route_solarsystems?.filter((m) => m.is_pinned));

const editing = ref(false);
const statistics = ref(false);

const form = useForm({
    notes: map_solarsystem.notes || '',
});

const md = markdownit({
    html: true,
    linkify: true,
    typographer: true,
    breaks: true,
});

const dotlan_link = computed(() => {
    const region = map_solarsystem.solarsystem?.region?.name.replace(' ', '_');
    const name = map_solarsystem.name.replace(' ', '_');
    if (map_solarsystem.class) {
        return `https://evemaps.dotlan.net/system/${name}`;
    }

    return `https://evemaps.dotlan.net/map/${region}/${name}`;
});

md.use(attr, {
    attrs: {
        target: '_blank',
        rel: 'noopener',
    },
});

const description = computed(() => {
    if (map_solarsystem.notes) {
        return md.render(map_solarsystem.notes);
    }
    return '';
});

function handleSubmit() {
    form.submit(MapSolarsystems.update(map_solarsystem.id), {
        onSuccess: () => {
            editing.value = false;
        },
        preserveScroll: true,
        preserveState: true,
        only: ['selected_map_solarsystem'],
    });
}

watch(
    () => map_solarsystem,
    (newValue) => {
        editing.value = false;
        form.notes = newValue.notes || '';
    },
);
</script>

<template>
    <MapPanel>
        <CardHeader class="pb-4">
            <div class="flex items-start justify-between gap-4">
                <div class="min-w-0 flex-1">
                    <CardTitle class="flex items-center gap-2 text-xl">
                        <SolarsystemClass :wormhole_class="map_solarsystem.class" v-if="map_solarsystem.class" />
                        <SecurityStatus
                            :security="map_solarsystem.solarsystem?.security"
                            v-else-if="map_solarsystem.solarsystem?.security !== undefined"
                        />
                        <span class="truncate">{{ map_solarsystem.name }}</span>
                        <div v-if="map_solarsystem.wormholes?.length" class="flex gap-1">
                            <Popover v-for="wormhole in map_solarsystem.wormholes" :key="wormhole.id">
                                <PopoverTrigger>
                                    <span
                                        :data-leads-to="wormhole.leads_to"
                                        class="cursor-pointer text-xl uppercase transition-colors hover:opacity-70 data-[leads-to=c1]:text-c1 data-[leads-to=c2]:text-c2 data-[leads-to=c3]:text-c3 data-[leads-to=c4]:text-c4 data-[leads-to=c5]:text-c5 data-[leads-to=c6]:text-c6 data-[leads-to=hs]:text-hs data-[leads-to=ls]:text-ls data-[leads-to=ns]:text-ns"
                                    >
                                        {{ wormhole.leads_to.replace('s', '') }}
                                    </span>
                                </PopoverTrigger>
                                <PopoverContent class="w-60">
                                    <div class="space-y-3">
                                        <div class="space-y-1">
                                            <div class="border-b pb-1 text-xs font-medium text-foreground">{{ wormhole.name }}</div>
                                            <div class="grid grid-cols-2 text-xs text-muted-foreground">
                                                <span>Leads to:</span>
                                                <span
                                                    class="text-right uppercase"
                                                    :data-leads-to="wormhole.leads_to"
                                                    :class="`data-[leads-to=c1]:text-c1 data-[leads-to=c2]:text-c2 data-[leads-to=c3]:text-c3 data-[leads-to=c4]:text-c4 data-[leads-to=c5]:text-c5 data-[leads-to=c6]:text-c6 data-[leads-to=hs]:text-hs data-[leads-to=ls]:text-ls data-[leads-to=ns]:text-ns`"
                                                >
                                                    {{ wormhole.leads_to.replace('s', '') }}
                                                </span>
                                            </div>
                                        </div>
                                        <WormholeProperties :wormhole="wormhole" />
                                    </div>
                                </PopoverContent>
                            </Popover>
                        </div>
                        <SolarsystemEffect :effect="map_solarsystem.effect" :effects="map_solarsystem.effects" v-if="map_solarsystem.effect" />
                    </CardTitle>

                    <div class="mt-1 flex items-center gap-2 text-sm text-muted-foreground">
                        <span>{{ map_solarsystem.solarsystem?.region?.name }}</span>
                        <span class="text-xs">•</span>
                        <span>{{ map_solarsystem.solarsystem?.constellation?.name }}</span>
                        <SolarsystemSovereignty
                            v-if="map_solarsystem.solarsystem?.sovereignty"
                            :sovereignty="map_solarsystem.solarsystem.sovereignty"
                            class="ml-1"
                        />
                    </div>
                </div>

                <div class="flex items-center gap-3 text-xs">
                    <a
                        :href="`https://zkillboard.com/system/${map_solarsystem.solarsystem?.id}/`"
                        target="_blank"
                        class="flex items-center gap-1 text-muted-foreground transition-colors hover:text-foreground"
                        rel="noopener"
                    >
                        <ExternalIcon class="size-3" />
                        <span>zKillboard</span>
                    </a>
                    <a
                        :href="dotlan_link"
                        target="_blank"
                        class="flex items-center gap-1 text-muted-foreground transition-colors hover:text-foreground"
                        rel="noopener"
                    >
                        <ExternalIcon class="size-3" />
                        <span>Dotlan</span>
                    </a>
                    <a
                        v-if="map_solarsystem.class"
                        :href="`https://anoik.is/systems/${map_solarsystem.name}`"
                        target="_blank"
                        class="flex items-center gap-1 text-muted-foreground transition-colors hover:text-foreground"
                        rel="noopener"
                    >
                        <ExternalIcon class="size-3" />
                        <span>Anoik.is</span>
                    </a>
                </div>
            </div>

            <div class="mt-4" v-if="pinned?.length">
                <Deferred data="map_route_solarsystems">
                    <template #fallback>
                        <div class="flex items-center gap-2 text-xs text-muted-foreground">
                            <Spinner class="size-3 animate-spin" />
                            <span class="animate-pulse">Loading destinations...</span>
                        </div>
                    </template>
                    <div class="flex flex-wrap gap-1">
                        <Destination v-for="jump in pinned" :key="jump.solarsystem.id" :destination="jump" />
                    </div>
                </Deferred>
            </div>
        </CardHeader>
        <MapPanelContent inner-class="p-4" v-if="!hide_notes">
            <form @submit.prevent="handleSubmit" class="w-full">
                <div class="mb-4 flex justify-between gap-4">
                    <h3 class="mr-auto text-lg font-semibold">Notes</h3>
                    <Dialog v-model:open="statistics" v-if="can_write">
                        <DialogTrigger as-child>
                            <Button variant="secondary" role="button">Statistics</Button>
                        </DialogTrigger>
                        <DialogContent>
                            <DialogHeader>
                                <DialogTitle> Collect system intel</DialogTitle>
                                <DialogDescription>
                                    Trigger the statistics collection to gather intel about all wormhole solarsystem and who might live there. It will
                                    list the possible occupants, their kill and set the system status based on the amount of kills.
                                </DialogDescription>
                            </DialogHeader>
                            <DialogFooter>
                                <Button variant="secondary" @click="statistics = false"> Cancel</Button>
                                <Button as-child>
                                    <Link
                                        :href="StatisticsController.store()"
                                        :data="{ map_id: map_solarsystem.map_id }"
                                        method="post"
                                        @click="statistics = false"
                                        preserve-state
                                        preserve-scroll
                                        :only="['selected_map_solarsystem']"
                                    >
                                        Add to queue
                                    </Link>
                                </Button>
                            </DialogFooter>
                        </DialogContent>
                    </Dialog>
                    <Button variant="secondary" @click="editing = true" v-if="!editing && can_write"> Edit</Button>
                    <div class="flex gap-4" v-else-if="can_write">
                        <Button variant="destructive" @click="editing = false"> Cancel</Button>
                        <Button variant="secondary" type="submit"> Save</Button>
                    </div>
                </div>
                <div
                    v-html="description"
                    v-if="!editing && map_solarsystem.notes"
                    class="prose-li:ml:0 prose-sm prose-neutral prose-invert prose-a:text-amber-500 prose-a:hover:underline prose-ul:list-disc"
                />
                <Textarea
                    v-else-if="editing"
                    v-model="form.notes"
                    class="h-[200px] w-full resize-none"
                    placeholder="Add notes about this solarsystem..."
                />
                <p class="text-xs text-muted-foreground" v-else-if="!map_solarsystem.notes">No notes found</p>
            </form>
        </MapPanelContent>
    </MapPanel>
</template>

<style scoped></style>
