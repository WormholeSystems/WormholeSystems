<script setup lang="ts">
import { ContextMenuItem, ContextMenuSub, ContextMenuSubContent, ContextMenuSubTrigger } from '@/components/ui/context-menu';
import { TMapSolarsystem } from '@/pages/maps';
import { computed } from 'vue';
import { toast } from 'vue-sonner';

const { map_solarsystem } = defineProps<{
    map_solarsystem: TMapSolarsystem;
}>();

const class_string = computed(() => {
    if (map_solarsystem.solarsystem.class) return `C${map_solarsystem.solarsystem.class}`;
    if (map_solarsystem.solarsystem!.security >= 0.5) return 'HS';
    if (map_solarsystem.solarsystem!.security > 0.0) return 'LS';
    return 'NS';
});

const default_name = computed(() => {
    return `${class_string.value} ${map_solarsystem.solarsystem.name} ${map_solarsystem.solarsystem?.region?.name}`;
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
