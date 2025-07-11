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
            <h1 class="font-[caveat] text-6xl">Hello guyz, welcum to my website</h1>
            <span class="mb-12 inline-block animate-spin text-6xl text-orange-500">Wow</span>

            <Link v-for="(map, i) in maps" :key="map.id" :href="route('maps.show', map.id)" class="mt-4 block text-3xl text-blue-500 hover:underline">
                <span v-if="i === 0">{{ map.name }} 🍆</span>
                <span v-else>or this</span>
            </Link>
            <div v-if="maps.length === 0" class="mt-4 text-2xl font-bold text-red-500">
                Oopsie 😢, looks like you has no frienz for spaghetti image generator yet. But you can have your own club if you want:
                <Link :href="route('maps.create')" class="text-blue-500 hover:underline">Create your own map</Link>
            </div>
            <div v-else class="mt-4 text-2xl font-bold text-green-500">
                You has {{ maps.length }} frienzs maps for spaghetti image generator! Want more noow?
                <Link :href="route('maps.create')" class="text-blue-500 hover:underline">Create your own map</Link>
            </div>
        </div>
    </AppLayout>
</template>

<style scoped></style>
