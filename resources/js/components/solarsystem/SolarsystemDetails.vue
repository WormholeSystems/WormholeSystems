<script setup lang="ts">
import ExternalIcon from '@/components/icons/ExternalIcon.vue';
import Spinner from '@/components/icons/Spinner.vue';
import SolarsystemEffect from '@/components/map/SolarsystemEffect.vue';
import SolarsystemSovereignty from '@/components/map/SolarsystemSovereignty.vue';
import Destination from '@/components/solarsystem/Destination.vue';
import SecurityStatus from '@/components/solarsystem/SecurityStatus.vue';
import SolarsystemClass from '@/components/SolarsystemClass.vue';
import { Button } from '@/components/ui/button';
import { Card, CardAction, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card';
import { Dialog, DialogContent, DialogDescription, DialogFooter, DialogHeader, DialogTitle, DialogTrigger } from '@/components/ui/dialog';
import { Textarea } from '@/components/ui/textarea';
import { useHasWritePermission } from '@/composables/useHasPermission';
import MapSolarsystems from '@/routes/map-solarsystems';
import Statistics from '@/routes/statistics';
import { TMap, TMapRouteSolarsystem, TMapSolarSystem } from '@/types/models';
import { Deferred, Link, useForm } from '@inertiajs/vue3';
import markdownit from 'markdown-it';
import attr from 'markdown-it-link-attributes';
import { computed, ref, watch } from 'vue';

const { map_solarsystem, map_route_solarsystems } = defineProps<{
    map_solarsystem: TMapSolarSystem;
    map_route_solarsystems?: TMapRouteSolarsystem[];
    map: TMap;
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
    <Card>
        <CardHeader>
            <CardTitle class="flex items-center gap-2">
                <SolarsystemClass :wormhole_class="map_solarsystem.class" v-if="map_solarsystem.class" />
                <SecurityStatus :security="map_solarsystem.solarsystem?.security" v-else-if="map_solarsystem.solarsystem?.security !== undefined" />
                {{ map_solarsystem.name }}
                <SolarsystemEffect :effect="map_solarsystem.effect" :effects="map_solarsystem.effects" v-if="map_solarsystem.effect" />
                <SolarsystemSovereignty v-if="map_solarsystem.solarsystem?.sovereignty" :sovereignty="map_solarsystem.solarsystem.sovereignty" />
                <a
                    :href="`https://zkillboard.com/system/${map_solarsystem.solarsystem?.id}/`"
                    target="_blank"
                    class="text-xs text-muted-foreground hover:text-foreground"
                    rel="noopener noreferrer"
                >
                    <ExternalIcon />
                </a>
            </CardTitle>
            <CardDescription>
                {{ map_solarsystem.solarsystem?.region?.name }} |
                {{ map_solarsystem.solarsystem?.constellation?.name }}
            </CardDescription>
            <CardAction>
                <Deferred data="map_route_solarsystems">
                    <template #fallback>
                        <span class="flex items-center gap-2 text-xs text-muted-foreground">
                            <Spinner class="animate-spin" />
                            <span class="animate-pulse"> Loading pinned destinations...</span>
                        </span>
                    </template>
                    <div class="flex flex-wrap items-center justify-end gap-1">
                        <Destination v-for="jump in pinned" :key="jump.solarsystem.id" :destination="jump" />
                    </div>
                </Deferred>
            </CardAction>
        </CardHeader>
        <CardContent>
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
                                        :href="Statistics.store()"
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
        </CardContent>
    </Card>
</template>

<style scoped></style>
