<script setup lang="ts">
import MapController from '@/actions/App/Http/Controllers/MapController';
import InputError from '@/components/InputError.vue';
import SeoHead from '@/components/SeoHead.vue';
import { Button } from '@/components/ui/button';
import { Card, CardContent, CardDescription, CardFooter, CardHeader, CardTitle } from '@/components/ui/card';
import { Input } from '@/components/ui/input';
import { Label } from '@/components/ui/label';
import AppLayout from '@/layouts/AppLayout.vue';
import { home } from '@/routes';
import { Link, useForm } from '@inertiajs/vue3';

const form = useForm({
    name: '',
});

function handleSubmit() {
    form.submit(MapController.store());
}
</script>

<template>
    <AppLayout>
        <SeoHead 
            title="Create a Map"
            description="Create a new wormhole mapping network. Start your exploration of dangerous wormhole space with collaborative mapping tools and real-time intel."
            keywords="create wormhole map, new map, wormhole exploration, eve online mapping"
        />
        <form @submit.prevent="handleSubmit" class="mx-auto mt-8 w-full max-w-2xl">
            <Card>
                <CardHeader>
                    <CardTitle> Create a Map</CardTitle>
                    <CardDescription> Fill in the details below to create a new map.</CardDescription>
                </CardHeader>
                <CardContent class="grid gap-4">
                    <Label for="name">Map Name</Label>
                    <Input id="name" v-model="form.name" type="text" placeholder="Enter map name" required />
                    <InputError :message="form.errors.name" class="mt-2" />
                    <CardFooter class="flex justify-between">
                        <Button as-child variant="secondary">
                            <Link :href="home()"> Cancel</Link>
                        </Button>
                        <Button type="submit" class="btn btn-primary"> Create Map</Button>
                    </CardFooter>
                </CardContent>
            </Card>
        </form>
    </AppLayout>
</template>
