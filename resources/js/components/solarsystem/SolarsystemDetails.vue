<script setup lang="ts">
import { Button } from '@/components/ui/button';
import MapPanel from '@/components/ui/map-panel/MapPanel.vue';
import MapPanelContent from '@/components/ui/map-panel/MapPanelContent.vue';
import MapPanelHeader from '@/components/ui/map-panel/MapPanelHeader.vue';
import { Textarea } from '@/components/ui/textarea';
import usePermission from '@/composables/usePermission';
import type { TResolvedSelectedMapSolarsystem } from '@/pages/maps';
import MapSolarsystems from '@/routes/map-solarsystems';
import { useForm } from '@inertiajs/vue3';
import { Pencil } from 'lucide-vue-next';
import markdownit from 'markdown-it';
import attr from 'markdown-it-link-attributes';
import { computed, ref, watch } from 'vue';

const { map_solarsystem } = defineProps<{
    map_solarsystem: TResolvedSelectedMapSolarsystem;
}>();

const { canEdit: can_write } = usePermission();

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
        <MapPanelHeader>
            Notes
            <template #actions>
                <template v-if="editing && can_write">
                    <Button variant="ghost" size="sm" class="h-5 px-1.5 text-[10px]" @click="editing = false" type="button"> Cancel </Button>
                    <Button
                        variant="ghost"
                        size="sm"
                        class="h-5 px-1.5 text-[10px] text-amber-400 hover:text-amber-400"
                        @click="handleSubmit"
                        type="button"
                    >
                        Save
                    </Button>
                </template>
                <Button
                    v-else-if="can_write"
                    variant="ghost"
                    size="icon"
                    class="size-5 text-muted-foreground hover:text-foreground"
                    @click="editing = true"
                    type="button"
                >
                    <Pencil class="size-3" />
                </Button>
            </template>
        </MapPanelHeader>
        <MapPanelContent>
            <Textarea
                v-if="editing"
                v-model="form.notes"
                class="h-full w-full resize-none border-0 bg-transparent px-3 py-2 font-mono text-xs focus-visible:ring-0"
                placeholder="Add notes..."
            />
            <div
                v-else-if="map_solarsystem.notes"
                v-html="description"
                class="prose prose-sm max-w-none px-3 py-2 prose-invert prose-headings:my-2 prose-headings:text-foreground prose-p:my-2 prose-a:text-amber-500 prose-a:no-underline hover:prose-a:underline prose-code:rounded prose-code:bg-muted prose-code:px-1 prose-code:py-0.5 prose-code:text-amber-400 prose-code:before:content-none prose-code:after:content-none prose-pre:border prose-pre:border-border/50 prose-pre:bg-muted prose-ol:my-2 prose-ul:my-2 prose-li:my-0.5"
            />
            <div v-else class="flex h-full flex-col items-center justify-center gap-2 p-4">
                <p class="font-mono text-[10px] tracking-wider text-muted-foreground/60 uppercase">No notes</p>
            </div>
        </MapPanelContent>
    </MapPanel>
</template>
