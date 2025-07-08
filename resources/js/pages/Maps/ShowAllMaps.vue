<script setup lang="ts">
import AppLayout from '@/layouts/AppLayout.vue';
import { TMap } from '@/types/models';
import { Link } from '@inertiajs/vue3';

defineProps<{
    maps: TMap[];
}>();
</script>

<template>
    <AppLayout>
        <div class="p-16">
            <h1 class="font-[caveat] text-6xl">Hello friendz, welcum to my website</h1>
            <span class="mb-12 inline-block animate-spin text-6xl text-orange-500">Wow</span>

            <Link v-for="(map, i) in maps" :key="map.id" :href="route('maps.show', map.id)" class="mt-4 block text-blue-500 hover:underline">
                <span v-if="i === 0">Plz click this 🍆</span>
                <span v-else>or this</span>
            </Link>
        </div>
    </AppLayout>
</template>

<style scoped></style>
