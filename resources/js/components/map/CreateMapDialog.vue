<script setup lang="ts">
import MapController from '@/actions/App/Http/Controllers/MapController';
import { Button } from '@/components/ui/button';
import {
    Dialog,
    DialogClose,
    DialogContent,
    DialogDescription,
    DialogFooter,
    DialogHeader,
    DialogTitle,
    DialogTrigger,
} from '@/components/ui/dialog';
import InputError from '@/components/ui/error/InputError.vue';
import { Input } from '@/components/ui/input';
import { Label } from '@/components/ui/label';
import { useForm } from '@inertiajs/vue3';
import { ref } from 'vue';

const open = ref(false);

const form = useForm({
    name: '',
});

function handleOpenChange(value: boolean): void {
    open.value = value;
    if (!value) {
        form.reset();
        form.clearErrors();
    }
}

function handleSubmit(): void {
    form.submit(MapController.store());
}
</script>

<template>
    <Dialog :open="open" @update:open="handleOpenChange">
        <DialogTrigger as-child>
            <slot />
        </DialogTrigger>
        <DialogContent>
            <form class="grid gap-6" @submit.prevent="handleSubmit">
                <DialogHeader>
                    <DialogTitle>Create a map</DialogTitle>
                    <DialogDescription>Name your new map. You can configure access and settings afterwards.</DialogDescription>
                </DialogHeader>
                <div class="grid gap-2">
                    <Label for="create-map-name">Map name</Label>
                    <Input id="create-map-name" v-model="form.name" type="text" placeholder="Enter map name" required />
                    <InputError :message="form.errors.name" />
                </div>
                <DialogFooter>
                    <DialogClose as-child>
                        <Button type="button" variant="secondary">Cancel</Button>
                    </DialogClose>
                    <Button type="submit" :disabled="form.processing">Create map</Button>
                </DialogFooter>
            </form>
        </DialogContent>
    </Dialog>
</template>
