<script setup lang="ts">
import { ContextMenuItem, ContextMenuSub, ContextMenuSubContent, ContextMenuSubTrigger } from '@/components/ui/context-menu';
import { TMapSolarsystem } from '@/pages/maps';
import { Copy } from 'lucide-vue-next';
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
    const parts = [];
    if (map_solarsystem.alias) parts.push(map_solarsystem.alias);
    parts.push(class_string.value);
    parts.push(map_solarsystem.solarsystem.name);
    parts.push(map_solarsystem.solarsystem?.region?.name);
    return parts.join(' ');
});

function copyNameToClipboard(value?: string) {
    navigator.clipboard.writeText(value || default_name.value);
    toast.success('Successfully copied name to clipboard');
}
</script>

<template>
    <ContextMenuSub>
        <ContextMenuSubTrigger>
            <Copy class="size-4" />
            Copy name
        </ContextMenuSubTrigger>
        <ContextMenuSubContent>
            <ContextMenuItem @select="copyNameToClipboard()">
                {{ default_name }}
            </ContextMenuItem>
        </ContextMenuSubContent>
    </ContextMenuSub>
</template>

<style scoped></style>
