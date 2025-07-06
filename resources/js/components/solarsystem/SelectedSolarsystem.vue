<script setup lang="ts">
import { Card, CardHeader, CardTitle } from '@/components/ui/card';
import { TMapSolarSystem } from '@/types/models';

const { map_solarsystem } = defineProps<{
    map_solarsystem: TMapSolarSystem;
}>();
</script>

<template>
    <Card>
        <CardHeader>
            <CardTitle>{{ map_solarsystem.name }}</CardTitle>
        </CardHeader>
    </Card>
</template>

<style scoped></style>
