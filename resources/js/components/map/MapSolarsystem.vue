<script setup lang="ts">
import MapSolarsystemButton from '@/components/map/MapSolarsystemButton.vue';
import MapSolarsystemContextMenu from '@/components/map/MapSolarsystemContextMenu.vue';
import { Button } from '@/components/ui/button';
import { Input } from '@/components/ui/input';
import { Popover, PopoverContent, PopoverTrigger } from '@/components/ui/popover';
import { useMapSolarsystem } from '@/composables/map';
import { useHasWritePermission } from '@/composables/useHasPermission';
import { useNewConnection } from '@/composables/useNewConnection';
import { TShowMapProps } from '@/pages/maps';
import MapSolarsystems from '@/routes/map-solarsystems';
import { AppPageProps } from '@/types';
import { TMapSolarSystem } from '@/types/models';
import { router, useForm, usePage } from '@inertiajs/vue3';
import { ref, useTemplateRef } from 'vue';

const { map_solarsystem } = defineProps<{
    map_solarsystem: TMapSolarSystem & { is_selected?: boolean };
}>();

const element = useTemplateRef('element');
const handle = useTemplateRef('handle');
const new_connection_handle = useTemplateRef('new_connection_handle');

const drag = useMapSolarsystem(
    () => map_solarsystem,
    () => element.value!,
    () => handle.value!,
);

const open = ref(false);

const page = usePage<AppPageProps<TShowMapProps>>();
const can_write = useHasWritePermission();
const form = useForm<{
    alias: string;
    occupier_alias: string;
}>({
    alias: map_solarsystem.alias ?? '',
    occupier_alias: map_solarsystem.occupier_alias ?? '',
});

function handleSubmit() {
    form.put(MapSolarsystems.update(map_solarsystem.id).url, {
        onSuccess: () => {
            open.value = false;
        },
        preserveScroll: true,
        preserveState: true,
        only: ['map'],
    });
}

useNewConnection(
    () => new_connection_handle.value!,
    () => map_solarsystem,
    () => element.value!,
);

function handleBadgeClick() {
    if (page.props.selected_map_solarsystem?.id === map_solarsystem.id) {
        return;
    }
    router.reload({
        data: {
            map_solarsystem_id: map_solarsystem.id,
        },
        only: ['selected_map_solarsystem', 'map_route_solarsystems', 'map_characters'],
    });
}

function handleBadgeDblClick() {
    open.value = true;
}
</script>

<template>
    <div
        ref="element"
        :style="drag.style.value"
        class="pointer-events-none absolute hover:z-20 data-[active=true]:z-10"
        :data-active="page.props.selected_map_solarsystem?.id === map_solarsystem.id"
    >
        <MapSolarsystemContextMenu :map_solarsystem>
            <div class="group relative -translate-x-[40px] -translate-y-[20px]">
                <Popover :open="open" @update:open="(value) => open && (open = value)">
                    <PopoverTrigger as-child>
                        <MapSolarsystemButton
                            @click="handleBadgeClick"
                            @dblclick="handleBadgeDblClick"
                            :map_solarsystem
                            :is_active="page.props.selected_map_solarsystem?.id === map_solarsystem.id"
                        />
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
                    v-if="!map_solarsystem.pinned && can_write"
                    class="absolute top-[1px] left-1/2 hidden h-2 w-12 -translate-x-1/2 -translate-y-1/2 cursor-move rounded border border-neutral-300 bg-white group-hover:z-50 group-hover:block dark:border-neutral-600 dark:bg-neutral-700"
                ></div>
                <div
                    ref="new_connection_handle"
                    v-if="can_write"
                    class="absolute top-1/2 left-full hidden h-4 w-4 -translate-x-1/2 -translate-y-1/2 cursor-pointer rounded-full border border-neutral-300 bg-white group-hover:block dark:border-neutral-600 dark:bg-neutral-700"
                ></div>
            </div>
        </MapSolarsystemContextMenu>
    </div>
</template>

<style scoped></style>
