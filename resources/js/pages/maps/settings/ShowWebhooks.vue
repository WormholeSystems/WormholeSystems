<script setup lang="ts">
import PlusIcon from '@/components/icons/PlusIcon.vue';
import SolarsystemEffect from '@/components/map/SolarsystemEffect.vue';
import SolarsystemClass from '@/components/solarsystem/SolarsystemClass.vue';
import { Badge } from '@/components/ui/badge';
import { Button } from '@/components/ui/button';
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card';
import { Checkbox } from '@/components/ui/checkbox';
import { Combobox, ComboboxAnchor, ComboboxEmpty, ComboboxGroup, ComboboxInput, ComboboxItem, ComboboxList } from '@/components/ui/combobox';
import { Dialog, DialogContent, DialogDescription, DialogFooter, DialogHeader, DialogTitle } from '@/components/ui/dialog';
import { Input } from '@/components/ui/input';
import { Label } from '@/components/ui/label';
import { useMapWebhooks } from '@/composables/useMapWebhooks';
import usePermission from '@/composables/usePermission';
import { useStaticData } from '@/composables/useStaticData';
import SettingsLayout from '@/layouts/SettingsLayout.vue';
import { TMapSummary } from '@/pages/maps';
import { TMapWebhook } from '@/types/models';
import { TStaticSolarsystem } from '@/types/static-data';
import { Pencil, Trash2 } from 'lucide-vue-next';
import { computed, reactive, ref } from 'vue';

const { map, webhooks } = defineProps<{
    map: TMapSummary;
    webhooks: TMapWebhook[];
}>();

const { canManageAccess } = usePermission();
const { createWebhook, updateWebhook, deleteWebhook } = useMapWebhooks();
const { staticData, loadStaticData } = useStaticData();

void loadStaticData();

const solarsystems = computed(() => staticData.value?.solarsystems ?? []);

function findSolarsystem(id: number | null): TStaticSolarsystem | undefined {
    if (!id) return undefined;
    return solarsystems.value.find((solarsystem) => solarsystem.id === id);
}

type TWebhookForm = {
    id: number | null;
    name: string;
    discord_webhook_url: string;
    target_solarsystem_id: number;
    max_jumps: number;
    is_active: boolean;
};

function emptyForm(): TWebhookForm {
    return { id: null, name: '', discord_webhook_url: '', target_solarsystem_id: 0, max_jumps: 5, is_active: true };
}

const form = reactive<TWebhookForm>(emptyForm());
const dialogOpen = ref(false);
const search = ref('');

const isEditing = computed(() => form.id !== null);

const selectedTarget = computed(() => findSolarsystem(form.target_solarsystem_id));

const filteredSolarsystems = computed(() => {
    const query = search.value.trim().toLowerCase();
    if (!query) {
        return [] as TStaticSolarsystem[];
    }

    return solarsystems.value.filter((solarsystem) => solarsystem.name.toLowerCase().includes(query)).slice(0, 25);
});

const canSubmit = computed(
    () =>
        form.name.trim().length > 0 &&
        form.target_solarsystem_id > 0 &&
        form.max_jumps >= 1 &&
        form.max_jumps <= 20 &&
        (isEditing.value || form.discord_webhook_url.trim().length > 0),
);

function openCreate() {
    Object.assign(form, emptyForm());
    search.value = '';
    dialogOpen.value = true;
}

function openEdit(webhook: TMapWebhook) {
    Object.assign(form, {
        id: webhook.id,
        name: webhook.name,
        discord_webhook_url: '',
        target_solarsystem_id: webhook.target_solarsystem_id,
        max_jumps: webhook.max_jumps,
        is_active: webhook.is_active,
    });
    search.value = findSolarsystem(webhook.target_solarsystem_id)?.name ?? '';
    dialogOpen.value = true;
}

function handleTargetSelect(solarsystem: TStaticSolarsystem) {
    form.target_solarsystem_id = solarsystem.id;
    search.value = solarsystem.name;
}

function submit() {
    if (!canSubmit.value) return;

    const payload = {
        name: form.name,
        discord_webhook_url: form.discord_webhook_url.trim() || undefined,
        target_solarsystem_id: form.target_solarsystem_id,
        max_jumps: form.max_jumps,
        is_active: form.is_active,
    };

    const onSuccess = () => {
        dialogOpen.value = false;
    };

    if (form.id !== null) {
        updateWebhook(form.id, payload, { onSuccess });
    } else {
        createWebhook(payload, { onSuccess });
    }
}
</script>

<template>
    <SettingsLayout :map="map" title="Webhooks" description="Get notified in Discord when a system comes within range of your chain">
        <Card>
            <CardHeader class="flex flex-row items-start justify-between gap-4">
                <div class="space-y-1.5">
                    <CardTitle class="text-xl font-semibold">Known-space connection alerts</CardTitle>
                    <CardDescription>
                        Get a Discord alert when a system is added to the map that is within a set number of <strong>stargate jumps</strong> of a
                        target system — e.g. "a new 10-jump Jita connection". Only known-space (gate) routes are counted; wormhole connections in your
                        chain are not factored in.
                    </CardDescription>
                </div>
                <Button v-if="canManageAccess" variant="outline" size="sm" @click="openCreate">
                    <PlusIcon class="mr-2 h-4 w-4" />
                    Add alert
                </Button>
            </CardHeader>
            <CardContent class="space-y-4">
                <p v-if="webhooks.length === 0" class="text-sm text-muted-foreground">No connection alerts configured yet.</p>

                <ul v-else class="divide-y divide-border rounded-lg border">
                    <li v-for="webhook in webhooks" :key="webhook.id" class="flex items-center gap-3 px-3 py-2">
                        <SolarsystemClass
                            v-if="findSolarsystem(webhook.target_solarsystem_id)"
                            :wormhole_class="findSolarsystem(webhook.target_solarsystem_id)!.class"
                            :security="findSolarsystem(webhook.target_solarsystem_id)!.security"
                            :name="findSolarsystem(webhook.target_solarsystem_id)!.name"
                        />
                        <div class="min-w-0">
                            <div class="flex items-center gap-2">
                                <span class="font-medium">{{ webhook.name }}</span>
                                <Badge v-if="!webhook.is_active" variant="outline">Inactive</Badge>
                            </div>
                            <div class="text-sm text-muted-foreground">
                                {{ findSolarsystem(webhook.target_solarsystem_id)?.name ?? webhook.target_solarsystem_id }} within
                                {{ webhook.max_jumps }} gate jump{{ webhook.max_jumps === 1 ? '' : 's' }}
                                <template v-if="webhook.last_fired_at"> · last fired {{ new Date(webhook.last_fired_at).toLocaleString() }}</template>
                            </div>
                        </div>
                        <div v-if="canManageAccess" class="ml-auto flex items-center gap-1">
                            <Button variant="ghost" size="icon" class="text-muted-foreground" @click="openEdit(webhook)">
                                <Pencil class="h-4 w-4" />
                            </Button>
                            <Button
                                variant="ghost"
                                size="icon"
                                class="text-muted-foreground hover:text-destructive"
                                @click="deleteWebhook(webhook.id)"
                            >
                                <Trash2 class="h-4 w-4" />
                            </Button>
                        </div>
                    </li>
                </ul>
            </CardContent>
        </Card>

        <Dialog v-model:open="dialogOpen">
            <DialogContent>
                <DialogHeader>
                    <DialogTitle>{{ isEditing ? 'Edit connection alert' : 'Add connection alert' }}</DialogTitle>
                    <DialogDescription>
                        Posts to Discord when a newly-mapped system is within the chosen number of stargate jumps of the target. Wormhole jumps in
                        your chain are not counted.
                    </DialogDescription>
                </DialogHeader>

                <div class="space-y-4">
                    <div class="space-y-1.5">
                        <Label>Alert type</Label>
                        <div class="rounded-md border bg-muted/40 px-3 py-2 text-sm">
                            <span class="font-medium">Known-space connection</span>
                            <span class="text-muted-foreground"> — distance measured over stargates only</span>
                        </div>
                    </div>

                    <div class="space-y-1.5">
                        <Label for="webhook-name">Name</Label>
                        <Input id="webhook-name" v-model="form.name" placeholder="e.g. Jita exit alert" />
                    </div>

                    <div class="space-y-1.5">
                        <Label>Target system</Label>
                        <div v-if="selectedTarget" class="flex items-center gap-2 text-sm">
                            <SolarsystemClass
                                :wormhole_class="selectedTarget.class"
                                :security="selectedTarget.security"
                                :name="selectedTarget.name"
                            />
                            <span class="font-medium">{{ selectedTarget.name }}</span>
                        </div>
                        <Combobox class="rounded-lg border bg-neutral-900">
                            <ComboboxAnchor>
                                <ComboboxInput v-model="search" placeholder="Search for a system…" />
                            </ComboboxAnchor>
                            <ComboboxList align="start">
                                <ComboboxEmpty>No results found</ComboboxEmpty>
                                <ComboboxGroup heading="Search Results" v-if="filteredSolarsystems.length > 0" class="grid grid-cols-[auto_1fr_auto]">
                                    <ComboboxItem
                                        v-for="solarsystem in filteredSolarsystems"
                                        :key="solarsystem.id"
                                        :value="solarsystem.name"
                                        @select.prevent="() => handleTargetSelect(solarsystem)"
                                        class="col-span-full grid grid-cols-subgrid"
                                    >
                                        <div class="justify-self-center">
                                            <SolarsystemClass
                                                :wormhole_class="solarsystem.class"
                                                :security="solarsystem.security"
                                                :name="solarsystem.name"
                                            />
                                        </div>
                                        <span class="whitespace-nowrap">{{ solarsystem.name }}</span>
                                        <span class="truncate text-muted-foreground" v-if="!solarsystem.class">{{ solarsystem.region?.name }}</span>
                                        <div class="justify-self-end" v-else-if="solarsystem.effect">
                                            <SolarsystemEffect :effect="solarsystem.effect" />
                                        </div>
                                    </ComboboxItem>
                                </ComboboxGroup>
                            </ComboboxList>
                        </Combobox>
                    </div>

                    <div class="space-y-1.5">
                        <Label for="webhook-jumps">Max gate jumps</Label>
                        <Input id="webhook-jumps" type="number" min="1" max="20" v-model.number="form.max_jumps" />
                        <p class="text-xs text-muted-foreground">Stargate jumps from the added system to the target (1–20).</p>
                    </div>

                    <div class="space-y-1.5">
                        <Label for="webhook-url">Discord webhook URL</Label>
                        <Input
                            id="webhook-url"
                            v-model="form.discord_webhook_url"
                            placeholder="https://discord.com/api/webhooks/…"
                            :type="isEditing ? 'text' : 'url'"
                        />
                        <p v-if="isEditing" class="text-xs text-muted-foreground">Leave blank to keep the current URL.</p>
                    </div>

                    <div class="flex items-center gap-2">
                        <Checkbox id="webhook-active" :model-value="form.is_active" @update:model-value="(v) => (form.is_active = v === true)" />
                        <Label for="webhook-active" class="text-sm font-medium">Active</Label>
                    </div>
                </div>

                <DialogFooter>
                    <Button variant="ghost" @click="dialogOpen = false">Cancel</Button>
                    <Button :disabled="!canSubmit" @click="submit">{{ isEditing ? 'Save' : 'Create' }}</Button>
                </DialogFooter>
            </DialogContent>
        </Dialog>
    </SettingsLayout>
</template>
