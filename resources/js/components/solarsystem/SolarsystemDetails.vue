<script setup lang="ts">
import ExternalIcon from '@/components/icons/ExternalIcon.vue';
import Spinner from '@/components/icons/Spinner.vue';
import SolarsystemEffect from '@/components/map/SolarsystemEffect.vue';
import SovereigntyIcon from '@/components/map/SovereigntyIcon.vue';
import Destination from '@/components/solarsystem/Destination.vue';
import SecurityStatus from '@/components/solarsystem/SecurityStatus.vue';
import SolarsystemClass from '@/components/SolarsystemClass.vue';
import { Button } from '@/components/ui/button';
import { Card, CardAction, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card';
import { Textarea } from '@/components/ui/textarea';
import { TMap, TMapRouteSolarsystem, TMapSolarSystem } from '@/types/models';
import { Deferred, useForm } from '@inertiajs/vue3';
import markdownit from 'markdown-it';
import { computed, ref } from 'vue';

const { map_solarsystem, map_route_solarsystems } = defineProps<{
    map_solarsystem: TMapSolarSystem;
    map_route_solarsystems?: TMapRouteSolarsystem[];
    map: TMap;
}>();

const pinned = computed(() => map_route_solarsystems?.filter((m) => m.is_pinned));

const editing = ref(false);

const form = useForm({
    notes: map_solarsystem.notes || '',
});

const md = markdownit({
    html: true,
    linkify: true,
    typographer: true,
    breaks: true,
});

const description = computed(() => {
    if (map_solarsystem.notes) {
        return md.render(map_solarsystem.notes);
    }
    return '';
});

function handleSubmit() {
    form.put(route('map-solarsystems.update', map_solarsystem.id), {
        onSuccess: () => {
            editing.value = false;
        },
        preserveScroll: true,
        preserveState: true,
        only: ['selected_map_solarsystem'],
    });
}
</script>

<template>
    <Card>
        <CardHeader>
            <CardTitle class="flex items-center gap-2">
                <SolarsystemClass :wormhole_class="map_solarsystem.class" v-if="map_solarsystem.class" />
                <SecurityStatus :security="map_solarsystem.solarsystem?.security" v-else-if="map_solarsystem.solarsystem?.security !== undefined" />
                {{ map_solarsystem.name }}
                <SolarsystemEffect :effect="map_solarsystem.effect" :effects="map_solarsystem.effects" v-if="map_solarsystem.effect" />
                <SovereigntyIcon v-if="map_solarsystem.solarsystem?.sovereignty" :sovereignty="map_solarsystem.solarsystem.sovereignty" />
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
                    <div class="flex items-center gap-2">
                        <Destination v-for="jump in pinned" :key="jump.solarsystem.id" :destination="jump" />
                    </div>
                </Deferred>
            </CardAction>
        </CardHeader>
        <CardContent>
            <form @submit.prevent="handleSubmit" class="w-full">
                <div class="mb-4 flex justify-between">
                    <h3 class="text-lg font-semibold">Notes</h3>
                    <Button variant="outline" @click="editing = true" v-if="!editing"> Edit</Button>
                    <Button variant="outline" v-else type="submit"> Save</Button>
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
