<script setup lang="ts">
import { ContextMenuItem, ContextMenuSub, ContextMenuSubContent, ContextMenuSubTrigger } from '@/components/ui/context-menu';
import { formatBookmarkName, getSignatureIdShort, TProcessedConnection } from '@/composables/map';
import { Copy } from 'lucide-vue-next';
import { computed } from 'vue';
import { toast } from 'vue-sonner';

const { map_connection } = defineProps<{
    map_connection: TProcessedConnection;
}>();

const source_signature = computed(() => {
    if (!map_connection.signatures?.length) return null;
    return map_connection.signatures.find((sig) => sig.map_solarsystem_id === map_connection.source.id) || null;
});

const target_signature = computed(() => {
    if (!map_connection.signatures?.length) return null;
    return map_connection.signatures.find((sig) => sig.map_solarsystem_id === map_connection.target.id) || null;
});

const source_name = computed(() => formatBookmarkName(map_connection.source, getSignatureIdShort(target_signature.value?.signature_id)));

const target_name = computed(() => formatBookmarkName(map_connection.target, getSignatureIdShort(source_signature.value?.signature_id)));

function copyNameToClipboard(value: string) {
    navigator.clipboard.writeText(value);
    toast.success('Copied to clipboard', { description: 'You successfully copied the name to your clipboard.' });
}
</script>

<template>
    <ContextMenuSub>
        <ContextMenuSubTrigger>
            <Copy class="size-4" />
            Copy name
        </ContextMenuSubTrigger>
        <ContextMenuSubContent>
            <ContextMenuItem @select="copyNameToClipboard(source_name)">
                {{ source_name }}
            </ContextMenuItem>
            <ContextMenuItem @select="copyNameToClipboard(target_name)">
                {{ target_name }}
            </ContextMenuItem>
        </ContextMenuSubContent>
    </ContextMenuSub>
</template>

<style scoped></style>
