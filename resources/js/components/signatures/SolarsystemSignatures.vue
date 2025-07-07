<script setup lang="ts">
import { TMap, TMapSolarSystem } from '@/types/models';
import { useMagicKeys, whenever } from '@vueuse/core';
import { ref } from 'vue';

const { map, map_solarsystem } = defineProps<{
    map: TMap;
    map_solarsystem: TMapSolarSystem;
}>();

const { Ctrl_v } = useMagicKeys();

whenever(Ctrl_v, handleClipboardPaste);

type TSignature = {
    id: string;
    type: string;
    category: string | null;
    name: string | null;
    strength: string | null;
    distance: string | null;
};

const signatures = ref<TSignature[]>([]);

async function handleClipboardPaste() {
    // check permission to access clipboard
    const clipboardData = navigator.clipboard?.readText();
    if (clipboardData) {
        const text = await clipboardData;
        signatures.value = text
            .split('\n')
            .map((sig) => sig.split('\t'))
            .map((parts) => ({
                id: parts[0],
                type: parts[1],
                category: parts[2] || null,
                name: parts[3] || null,
                strength: parts[4] || null,
                distance: parts[5] || null,
            }));
    }
}
</script>

<template>
    <div class="overflow-x-auto">
        <table class="w-full text-xs">
            <thead>
                <tr class="border-b text-left">
                    <th class="p-1">ID</th>
                    <th class="p-1">Type</th>
                    <th class="p-1">Category</th>
                    <th class="p-1">Name</th>
                </tr>
            </thead>
            <tbody class="text-muted-foreground">
                <tr v-for="signature in signatures" :key="signature.id" class="border-b last:border-b-0">
                    <td class="p-1">{{ signature.id }}</td>
                    <td class="p-1">{{ signature.type }}</td>
                    <td class="p-1">{{ signature.category }}</td>
                    <td class="p-1">{{ signature.name }}</td>
                </tr>
            </tbody>
        </table>
    </div>
</template>

<style scoped></style>
