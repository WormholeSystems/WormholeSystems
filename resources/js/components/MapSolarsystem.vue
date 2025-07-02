<script setup lang="ts">
import SolarsystemClass from '@/components/SolarsystemClass.vue';
import SolarsystemEffect from '@/components/SolarsystemEffect.vue';
import { Button } from '@/components/ui/button';
import { Input } from '@/components/ui/input';
import { Popover, PopoverContent, PopoverTrigger } from '@/components/ui/popover';
import { useMap } from '@/composables/useMap';
import { useNewConnection } from '@/composables/useNewConnection';
import { useSystemDrag } from '@/composables/useSystemDrag';
import { TMapSolarSystem } from '@/types/models';
import { router, useForm } from '@inertiajs/vue3';
import { useDraggable } from '@vueuse/core';
import { ref, useTemplateRef } from 'vue';

const { map_solarsystem } = defineProps<{
    map_solarsystem: TMapSolarSystem & { is_selected?: boolean };
}>();

const map = useMap();
const element = useTemplateRef('element');
const handle = useTemplateRef('handle');
const new_connection_handle = useTemplateRef('new_connection_handle');

const open = ref(false);

const form = useForm<{
    alias: string;
    occupier_alias: string;
}>({
    alias: map_solarsystem.alias ?? '',
    occupier_alias: map_solarsystem.occupier_alias ?? '',
});

const { style, x, y } = useDraggable(element, {
    initialValue() {
        return {
            x: map_solarsystem.position?.x ?? 0,
            y: map_solarsystem.position?.y ?? 0,
        };
    },
    containerElement: () => map.value.container,
    handle,
    onEnd: handleDragEnd,
    onMove: handleDrag,
});

const { updateDragPosition } = useSystemDrag();

useNewConnection(
    () => new_connection_handle.value!,
    () => map_solarsystem,
    () => element.value!,
);

function handleDragEnd() {
    router.put(
        route('map-solarsystems.update', map_solarsystem.id),
        {
            position_x: x.value,
            position_y: y.value,
        },
        {
            onSuccess: () => {
                updateDragPosition(null, null);
            },
        },
    );
}

function handleSubmit() {
    form.put(route('map-solarsystems.update', map_solarsystem.id), {
        onSuccess: () => {
            open.value = false;
        },
    });
}

function handleDrag() {
    updateDragPosition(
        {
            x: x.value,
            y: y.value,
        },
        map_solarsystem,
    );
}
</script>

<template>
    <div ref="element" :style="style" class="absolute" @close="() => (open = false)">
        <div class="group relative -translate-x-1/2 -translate-y-1/2">
            <Popover :open="open" @update:open="(value) => open && (open = value)">
                <PopoverTrigger as-child>
                    <button
                        @click="
                            () =>
                                router.reload({
                                    data: {
                                        map_solarsystem_id: map_solarsystem.id,
                                    },
                                })
                        "
                        @dblclick="() => (open = true)"
                        :data-selected="map_solarsystem.is_selected"
                        class="grid h-[40px] grid-cols-[auto_1fr_auto] items-center justify-center gap-x-1 rounded border border-neutral-700 bg-neutral-800 px-2 text-xs select-none data-[selected=true]:bg-amber-900"
                    >
                        <SolarsystemClass :security="map_solarsystem.solarsystem!.security" :wormhole_class="map_solarsystem.class" />
                        <p>
                            <span v-if="map_solarsystem.alias">{{ map_solarsystem.alias }} - </span>
                            <span>{{ map_solarsystem.solarsystem?.name }}</span>
                            <span v-if="map_solarsystem.occupier_alias"> ({{ map_solarsystem.occupier_alias }})</span>
                        </p>
                        <SolarsystemEffect :effect="map_solarsystem.effect" v-if="map_solarsystem.effect" />
                        <span class="col-span-2 row-start-2 block text-xs text-muted-foreground">{{
                            map_solarsystem.solarsystem?.region?.name
                        }}</span>
                    </button>
                </PopoverTrigger>
                <PopoverContent>
                    <form @submit.prevent="handleSubmit" class="grid gap-2">
                        <Input v-model="form.alias" type="text" placeholder="Alias" class="w-full" />
                        <Input v-model="form.occupier_alias" type="text" placeholder="Occupier Alias" class="w-full" />
                        <Button type="submit"> Save</Button>
                    </form>
                </PopoverContent>
            </Popover>

            <div
                ref="handle"
                class="absolute top-[1px] left-1/2 hidden h-2 w-16 -translate-x-1/2 -translate-y-1/2 cursor-move rounded border border-neutral-600 bg-neutral-700 opacity-0 transition-discrete duration-200 group-hover:block group-hover:opacity-100 starting:hidden starting:opacity-0"
            ></div>
            <div
                ref="new_connection_handle"
                class="absolute top-1/2 left-full h-4 w-4 -translate-x-1/2 -translate-y-1/2 rounded-full bg-neutral-700 opacity-0 transition-opacity duration-200 group-hover:opacity-100"
            ></div>
        </div>
    </div>
</template>

<style scoped></style>
