<script setup lang="ts">
import { Button } from '@/components/ui/button';
import { TMap, TMapSolarSystem } from '@/types/models';
import { router } from '@inertiajs/vue3';
import { useMagicKeys, whenever } from '@vueuse/core';

const { map, map_solarsystem } = defineProps<{
    map: TMap;
    map_solarsystem: TMapSolarSystem;
}>();

const { Ctrl_v } = useMagicKeys();

whenever(Ctrl_v, handleClipboardPaste);

async function handleClipboardPaste() {
    // check permission to access clipboard
    const clipboardData = navigator.clipboard?.readText();
    if (clipboardData) {
        const text = (await clipboardData).trim();

        let signatures = text
            ? text
                  .split('\n')
                  .map((sig) => sig.split('\t'))
                  .map((parts) => ({
                      signature_id: parts[0],
                      type: parts[1],
                      category: parts[2] || null,
                      name: parts[3] || null,
                  }))
            : [];

        router.post(
            route('map-solarsystems.signatures.store', map_solarsystem.id),
            {
                signatures,
            },
            {
                preserveScroll: true,
                preserveState: true,
                only: ['map', 'selected_map_solarsystem'],
            },
        );
    }
}
</script>

<template>
    <div class="mb-4 flex justify-between">
        <h3>Signatures</h3>
        <Button @click="handleClipboardPaste" class="mb-2" variant="outline" size="sm"> Read from clipboard</Button>
    </div>
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
                <tr v-for="signature in map_solarsystem.signatures" :key="signature.id" class="border-b last:border-b-0">
                    <td class="p-1">{{ signature.signature_id }}</td>
                    <td class="p-1">{{ signature.type }}</td>
                    <td class="p-1">{{ signature.category }}</td>
                    <td class="p-1">{{ signature.name }}</td>
                </tr>
                <tr v-if="map_solarsystem.signatures?.length === 0" class="border-b last:border-b-0">
                    <td colspan="4" class="py-4 text-left text-muted-foreground">No signatures found</td>
                </tr>
            </tbody>
        </table>
    </div>
</template>

<style scoped></style>
