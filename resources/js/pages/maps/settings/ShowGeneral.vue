<script setup lang="ts">
import { Button } from '@/components/ui/button';
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card';
import { Dialog, DialogContent, DialogDescription, DialogFooter, DialogHeader, DialogTitle, DialogTrigger } from '@/components/ui/dialog';
import { Input } from '@/components/ui/input';
import { Label } from '@/components/ui/label';
import useIsMapOwner from '@/composables/useIsMapOwner';
import SettingsLayout from '@/layouts/SettingsLayout.vue';
import { TMapSummary } from '@/pages/maps';
import { destroy, update } from '@/routes/maps';
import { router, useForm } from '@inertiajs/vue3';
import { ref } from 'vue';

const { map } = defineProps<{
    map: TMapSummary;
}>();

const user_is_owner = useIsMapOwner();

const form = useForm({
    name: map.name,
});

const delete_confirmation = ref('');
const is_deleting = ref(false);

function deleteMap() {
    if (delete_confirmation.value !== map.name) {
        return;
    }
    router.delete(destroy(map.slug));
}
</script>

<template>
    <SettingsLayout :map="map" title="General Settings" description="Configure your personal preferences and tracking settings for this map">
        <div class="space-y-6">
            <!-- Map Management Section (Owner Only) -->
            <Card v-if="user_is_owner">
                <CardHeader>
                    <CardTitle class="text-xl font-semibold">Map Management</CardTitle>
                    <CardDescription>Configure basic map settings (owner only)</CardDescription>
                </CardHeader>
                <CardContent>
                    <form @submit.prevent="() => form.put(update(map.slug).url)">
                        <Label for="name" class="text-sm font-medium">Map Name</Label>
                        <div class="flex gap-2">
                            <Input name="name" id="name" v-model:model-value="form.name" />
                            <Button type="submit" :disabled="!form.isDirty"> Update </Button>
                        </div>
                        <small v-if="form.errors.name" class="text-red-500">{{ form.errors.name }}</small>
                        <div class="text-sm text-muted-foreground">The display name for this map</div>
                    </form>
                </CardContent>
            </Card>

            <!-- Danger Zone (Owner Only) -->
            <Card v-if="user_is_owner">
                <CardHeader>
                    <CardTitle class="text-xl font-semibold">Danger Zone</CardTitle>
                    <CardDescription>Irreversible and destructive actions</CardDescription>
                </CardHeader>
                <CardContent>
                    <div class="flex items-center justify-between">
                        <div>
                            <h4 class="font-medium">Delete Map</h4>
                            <p class="text-sm text-muted-foreground">Permanently delete this map and all its data. This action cannot be undone.</p>
                        </div>
                        <Dialog v-model:open="is_deleting">
                            <DialogTrigger as-child>
                                <Button variant="destructive">Delete Map</Button>
                            </DialogTrigger>
                            <DialogContent class="max-w-md">
                                <DialogHeader>
                                    <DialogTitle class="text-foreground">Delete Map</DialogTitle>
                                    <DialogDescription class="text-muted-foreground">
                                        This will permanently delete <strong class="text-foreground">"{{ map.name }}"</strong> and all associated
                                        data:
                                    </DialogDescription>
                                </DialogHeader>

                                <div class="space-y-4">
                                    <div class="rounded-lg border border-border/50 p-3">
                                        <ul class="space-y-1 text-sm text-muted-foreground">
                                            <li>• Solar systems and positions</li>
                                            <li>• Wormhole connections</li>
                                            <li>• Signatures and bookmarks</li>
                                            <li>• Access permissions</li>
                                            <li>• User preferences</li>
                                        </ul>
                                    </div>

                                    <div>
                                        <Label for="delete-confirmation" class="text-sm font-medium">
                                            Type <strong>{{ map.name }}</strong> to confirm:
                                        </Label>
                                        <Input
                                            id="delete-confirmation"
                                            v-model="delete_confirmation"
                                            placeholder="Enter map name"
                                            class="mt-2"
                                            @keydown.enter="deleteMap"
                                        />
                                    </div>
                                </div>

                                <DialogFooter class="gap-2">
                                    <Button variant="outline" @click="is_deleting = false">Cancel</Button>
                                    <Button variant="destructive" @click="deleteMap" :disabled="delete_confirmation !== map.name">
                                        Delete Forever
                                    </Button>
                                </DialogFooter>
                            </DialogContent>
                        </Dialog>
                    </div>
                </CardContent>
            </Card>
        </div>
    </SettingsLayout>
</template>
