<script setup lang="ts">
import { Button } from '@/components/ui/button';
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card';
import { Dialog, DialogContent, DialogDescription, DialogFooter, DialogHeader, DialogTitle, DialogTrigger } from '@/components/ui/dialog';
import { Input } from '@/components/ui/input';
import { Label } from '@/components/ui/label';
import { Switch } from '@/components/ui/switch';
import useIsMapOwner from '@/composables/useIsMapOwner';
import SettingsLayout from '@/layouts/SettingsLayout.vue';
import { TMapSummary } from '@/pages/maps';
import { destroy, update } from '@/routes/maps';
import { router, useForm } from '@inertiajs/vue3';
import { Check, Copy, Globe, Link2, Link2Off } from 'lucide-vue-next';
import { computed, ref } from 'vue';

const { map, is_public, share_token } = defineProps<{
    map: TMapSummary;
    is_public: boolean;
    share_token: string | null;
}>();

const user_is_owner = useIsMapOwner();

const form = useForm({
    name: map.name,
});

const delete_confirmation = ref('');
const is_deleting = ref(false);
const copied = ref(false);
const shareUrl = computed(() => (share_token ? `${window.location.origin}/share/${share_token}` : ''));

function deleteMap() {
    if (delete_confirmation.value !== map.name) {
        return;
    }
    router.delete(destroy(map.slug));
}

function togglePublic() {
    router.post(
        `/maps/${map.slug}/settings/toggle-public`,
        {},
        {
            preserveState: true,
            preserveScroll: true,
        },
    );
}

function generateShareToken() {
    router.post(
        `/maps/${map.slug}/settings/generate-share-token`,
        {},
        {
            preserveState: true,
            preserveScroll: true,
        },
    );
}

function revokeShareToken() {
    router.delete(`/maps/${map.slug}/settings/revoke-share-token`, {
        preserveState: true,
        preserveScroll: true,
    });
}

function copyShareLink() {
    if (!shareUrl.value) return;
    navigator.clipboard.writeText(shareUrl.value);
    copied.value = true;
    setTimeout(() => {
        copied.value = false;
    }, 2000);
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

            <!-- Public Access Section -->
            <Card>
                <CardHeader>
                    <CardTitle class="flex items-center gap-2 text-xl font-semibold">
                        <Globe class="size-5" />
                        Public Access
                    </CardTitle>
                    <CardDescription>Allow anyone to view this map without logging in</CardDescription>
                </CardHeader>
                <CardContent>
                    <div class="space-y-4">
                        <div class="flex items-center justify-between">
                            <div>
                                <h4 class="font-medium">Public Map</h4>
                                <p class="text-sm text-muted-foreground">
                                    When enabled, anyone can view this map as a Viewer without needing an account.
                                </p>
                            </div>
                            <Switch :model-value="is_public" @update:modelValue="togglePublic" />
                        </div>
                    </div>
                </CardContent>
            </Card>

            <!-- Share Link Section -->
            <Card>
                <CardHeader>
                    <CardTitle class="flex items-center gap-2 text-xl font-semibold">
                        <Link2 class="size-5" />
                        Share Link
                    </CardTitle>
                    <CardDescription>Generate a shareable link for viewer-only access</CardDescription>
                </CardHeader>
                <CardContent>
                    <div class="space-y-4">
                        <p class="text-sm text-muted-foreground">
                            Share links provide viewer-only access to this map. Anyone with the link can view the map topology, signatures, killmails,
                            and EVE Scout connections.
                        </p>

                        <div v-if="share_token" class="space-y-3">
                            <div class="flex items-center gap-2">
                                <Input :model-value="shareUrl" readonly class="font-mono text-sm" />
                                <Button variant="outline" size="icon" @click="copyShareLink">
                                    <Check v-if="copied" class="size-4 text-green-500" />
                                    <Copy v-else class="size-4" />
                                </Button>
                            </div>
                            <Button variant="destructive" size="sm" @click="revokeShareToken">
                                <Link2Off class="mr-2 size-4" />
                                Revoke Share Link
                            </Button>
                        </div>

                        <Button v-else variant="outline" @click="generateShareToken">
                            <Link2 class="mr-2 size-4" />
                            Generate Share Link
                        </Button>
                    </div>
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
