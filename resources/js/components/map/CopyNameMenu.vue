<script setup lang="ts">
import { ContextMenuItem, ContextMenuSubContent, ContextMenuSubTrigger } from '@/components/ui/context-menu';
import { TMapSolarSystem } from '@/types/models';
import { computed } from 'vue';
import { toast } from 'vue-sonner';

const { map_solarsystem } = defineProps<{
    map_solarsystem: TMapSolarSystem;
}>();

const class_string = computed(() => {
    if (map_solarsystem.class) return `C${map_solarsystem.class}`;
    if (map_solarsystem.solarsystem!.security >= 0.5) return 'HS';
    if (map_solarsystem.solarsystem!.security > 0.0) return 'LS';
    return 'NS';
});

const default_name = computed(() => {
    return `${class_string.value} ${map_solarsystem.name} ${map_solarsystem.solarsystem?.region?.name}`;
});

function copyNameToClipboard(value?: string) {
    navigator.clipboard.writeText(value || default_name.value);
    toast.success('Successfully copied name to clipboard');
}
</script>

<template>
    <ContextMenuSub>
        <ContextMenuSubTrigger>Copy name</ContextMenuSubTrigger>
        <ContextMenuSubContent>
            <ContextMenuItem @select="copyNameToClipboard()">
                {{ default_name }}
            </ContextMenuItem>
        </ContextMenuSubContent>
    </ContextMenuSub>
</template>

<style scoped></style>
