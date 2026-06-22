<script setup lang="ts">
import SolarsystemEffect from '@/components/map/SolarsystemEffect.vue';
import KillmailFilterEditor from '@/components/maps/webhooks/KillmailFilterEditor.vue';
import WebhookList from '@/components/maps/webhooks/WebhookList.vue';
import SolarsystemClass from '@/components/solarsystem/SolarsystemClass.vue';
import { Button } from '@/components/ui/button';
import { Checkbox } from '@/components/ui/checkbox';
import { Combobox, ComboboxAnchor, ComboboxEmpty, ComboboxGroup, ComboboxInput, ComboboxItem, ComboboxList } from '@/components/ui/combobox';
import { Dialog, DialogContent, DialogDescription, DialogFooter, DialogHeader, DialogTitle } from '@/components/ui/dialog';
import { Input } from '@/components/ui/input';
import { Label } from '@/components/ui/label';
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from '@/components/ui/select';
import { Separator } from '@/components/ui/separator';
import { useMapWebhooks } from '@/composables/useMapWebhooks';
import usePermission from '@/composables/usePermission';
import { useStaticData } from '@/composables/useStaticData';
import SettingsLayout from '@/layouts/SettingsLayout.vue';
import { TMapSummary } from '@/pages/maps';
import { TKillmailFilterMatch, TKillmailFilterRule, TMapWebhook, TMapWebhookType } from '@/types/models';
import { TStaticSolarsystem } from '@/types/static-data';
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
    discord_role_id: string;
    type: TMapWebhookType;
    target_solarsystem_id: number;
    max_jumps: number;
    filter_match: TKillmailFilterMatch;
    filters: TKillmailFilterRule[];
    is_active: boolean;
};

function emptyForm(type: TMapWebhookType = 'proximity'): TWebhookForm {
    return {
        id: null,
        name: '',
        discord_webhook_url: '',
        discord_role_id: '',
        type,
        target_solarsystem_id: 0,
        max_jumps: 5,
        filter_match: 'any',
        filters: [],
        is_active: true,
    };
}

const form = reactive<TWebhookForm>(emptyForm());
const dialogOpen = ref(false);
const search = ref('');

const isEditing = computed(() => form.id !== null);
const isKillmail = computed(() => form.type === 'killmail');

const proximityWebhooks = computed(() => webhooks.filter((webhook) => webhook.type === 'proximity'));
const killmailWebhooks = computed(() => webhooks.filter((webhook) => webhook.type === 'killmail'));

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
        (isKillmail.value || form.target_solarsystem_id > 0) &&
        form.max_jumps >= 1 &&
        form.max_jumps <= 20 &&
        (isEditing.value || form.discord_webhook_url.trim().length > 0),
);

function openCreate(type: TMapWebhookType) {
    Object.assign(form, emptyForm(type));
    search.value = '';
    dialogOpen.value = true;
}

function openEdit(webhook: TMapWebhook) {
    // Start from defaults (discord_webhook_url stays blank — it never round-trips) and
    // overlay the saved values.
    Object.assign(form, emptyForm(webhook.type), {
        id: webhook.id,
        name: webhook.name,
        discord_role_id: webhook.discord_role_id ?? '',
        target_solarsystem_id: webhook.target_solarsystem_id ?? 0,
        max_jumps: webhook.max_jumps,
        filter_match: webhook.filter_match,
        filters: webhook.filters.map((filter) => ({ ...filter, ids: [...filter.ids] })),
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
        discord_role_id: form.discord_role_id.trim() || null,
        type: form.type,
        target_solarsystem_id: isKillmail.value ? null : form.target_solarsystem_id,
        max_jumps: form.max_jumps,
        filter_match: form.filter_match,
        filters: isKillmail.value ? form.filters : [],
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
        <WebhookList
            title="Known-space connection alerts"
            empty-text="No connection alerts configured yet."
            :webhooks="proximityWebhooks"
            :can-manage="canManageAccess"
            @add="openCreate('proximity')"
            @edit="openEdit"
            @delete="deleteWebhook"
        >
            <template #description>
                Get a Discord alert when a system is added to the map that is within a set number of <strong>stargate jumps</strong> of a target
                system — e.g. "a new 10-jump Jita connection". Only known-space (gate) routes are counted; wormhole connections in your chain are not
                factored in.
            </template>
            <template #icon="{ webhook }">
                <SolarsystemClass
                    v-if="findSolarsystem(webhook.target_solarsystem_id)"
                    :solarsystem_class="findSolarsystem(webhook.target_solarsystem_id)!.class"
                    :name="findSolarsystem(webhook.target_solarsystem_id)!.name"
                />
            </template>
            <template #detail="{ webhook }">
                {{ findSolarsystem(webhook.target_solarsystem_id)?.name ?? webhook.target_solarsystem_id }} within {{ webhook.max_jumps }} gate jump{{
                    webhook.max_jumps === 1 ? '' : 's'
                }}
            </template>
        </WebhookList>

        <WebhookList
            title="Killmail alerts"
            empty-text="No killmail alerts configured yet."
            :webhooks="killmailWebhooks"
            :can-manage="canManageAccess"
            @add="openCreate('killmail')"
            @edit="openEdit"
            @delete="deleteWebhook"
        >
            <template #description>
                Get a Discord alert when a killmail matching your filters happens within range of your chain. Distance is measured from any system in
                your chain, so both <strong>gate and wormhole jumps</strong> are factored in. Filter by ship type/group, character, corporation or
                alliance.
            </template>
            <template #detail="{ webhook }">
                Kills within {{ webhook.max_jumps }} jump{{ webhook.max_jumps === 1 ? '' : 's' }} of your chain ·
                {{ webhook.filters.length }} filter{{ webhook.filters.length === 1 ? '' : 's' }}
            </template>
        </WebhookList>

        <Dialog v-model:open="dialogOpen">
            <DialogContent class="max-h-[85vh] overflow-y-auto sm:max-w-2xl">
                <DialogHeader>
                    <DialogTitle>{{ isEditing ? 'Edit alert' : 'Add alert' }} · {{ isKillmail ? 'Killmail' : 'Connection' }}</DialogTitle>
                    <DialogDescription v-if="isKillmail">
                        Posts to Discord when a killmail matching your filters happens within range of your chain.
                    </DialogDescription>
                    <DialogDescription v-else>
                        Posts to Discord when a newly-mapped system comes within the chosen number of stargate jumps of the target.
                    </DialogDescription>
                </DialogHeader>

                <div class="space-y-6 py-1">
                    <section class="space-y-3">
                        <h3 class="text-sm font-semibold">Alert</h3>
                        <div class="grid gap-4 sm:grid-cols-2">
                            <div class="space-y-1.5">
                                <Label for="webhook-name">Name</Label>
                                <Input id="webhook-name" v-model="form.name" placeholder="e.g. Jita exit alert" />
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
                            <div class="space-y-1.5">
                                <Label for="webhook-role">Ping role ID</Label>
                                <Input
                                    id="webhook-role"
                                    v-model="form.discord_role_id"
                                    inputmode="numeric"
                                    placeholder="Optional — Discord role ID"
                                />
                                <p class="text-xs text-muted-foreground">
                                    Pasted role ID gets pinged when this alert fires. Leave blank for no ping.
                                </p>
                            </div>
                        </div>
                        <div class="flex items-center gap-2">
                            <Checkbox id="webhook-active" :model-value="form.is_active" @update:model-value="(v) => (form.is_active = v === true)" />
                            <Label for="webhook-active" class="text-sm font-medium">Active</Label>
                        </div>
                    </section>

                    <Separator />

                    <section class="space-y-3">
                        <h3 class="text-sm font-semibold">{{ isKillmail ? 'Range' : 'Target & range' }}</h3>

                        <div v-if="!isKillmail" class="space-y-1.5">
                            <Label>Target system</Label>
                            <div v-if="selectedTarget" class="flex items-center gap-2 text-sm">
                                <SolarsystemClass :solarsystem_class="selectedTarget.class" :name="selectedTarget.name" />
                                <span class="font-medium">{{ selectedTarget.name }}</span>
                            </div>
                            <Combobox class="rounded-lg border bg-neutral-900">
                                <ComboboxAnchor>
                                    <ComboboxInput v-model="search" placeholder="Search for a system…" />
                                </ComboboxAnchor>
                                <ComboboxList align="start">
                                    <ComboboxEmpty>No results found</ComboboxEmpty>
                                    <ComboboxGroup
                                        heading="Search Results"
                                        v-if="filteredSolarsystems.length > 0"
                                        class="grid grid-cols-[auto_1fr_auto]"
                                    >
                                        <ComboboxItem
                                            v-for="solarsystem in filteredSolarsystems"
                                            :key="solarsystem.id"
                                            :value="solarsystem.name"
                                            @select.prevent="() => handleTargetSelect(solarsystem)"
                                            class="col-span-full grid grid-cols-subgrid"
                                        >
                                            <div class="justify-self-center">
                                                <SolarsystemClass :solarsystem_class="solarsystem.class" :name="solarsystem.name" />
                                            </div>
                                            <span class="whitespace-nowrap">{{ solarsystem.name }}</span>
                                            <span class="truncate text-muted-foreground" v-if="!solarsystem.class">{{
                                                solarsystem.region?.name
                                            }}</span>
                                            <div class="justify-self-end" v-else-if="solarsystem.effect">
                                                <SolarsystemEffect :effect="solarsystem.effect" />
                                            </div>
                                        </ComboboxItem>
                                    </ComboboxGroup>
                                </ComboboxList>
                            </Combobox>
                        </div>

                        <div :class="isKillmail ? 'grid gap-4 sm:grid-cols-2' : 'space-y-1.5'">
                            <div class="space-y-1.5">
                                <Label for="webhook-jumps">{{ isKillmail ? 'Max jumps from chain' : 'Max gate jumps' }}</Label>
                                <Input id="webhook-jumps" type="number" min="1" max="20" v-model.number="form.max_jumps" />
                                <p class="text-xs text-muted-foreground">
                                    {{
                                        isKillmail
                                            ? 'Gate and wormhole jumps from any chain system (1–20).'
                                            : 'Stargate jumps from the added system to the target (1–20).'
                                    }}
                                </p>
                            </div>
                            <div v-if="isKillmail" class="space-y-1.5">
                                <Label>Match</Label>
                                <Select :model-value="form.filter_match" @update:model-value="(v) => (form.filter_match = v as TKillmailFilterMatch)">
                                    <SelectTrigger class="w-full">
                                        <SelectValue />
                                    </SelectTrigger>
                                    <SelectContent>
                                        <SelectItem value="any">Match any filter</SelectItem>
                                        <SelectItem value="all">Match all filters</SelectItem>
                                    </SelectContent>
                                </Select>
                                <p class="text-xs text-muted-foreground">"Any" = at least one filter; "all" = every filter. Excludes always block.</p>
                            </div>
                        </div>
                    </section>

                    <template v-if="isKillmail">
                        <Separator />
                        <section class="space-y-3">
                            <KillmailFilterEditor v-model:filters="form.filters" />
                        </section>
                    </template>
                </div>

                <DialogFooter>
                    <Button variant="ghost" @click="dialogOpen = false">Cancel</Button>
                    <Button :disabled="!canSubmit" @click="submit">{{ isEditing ? 'Save' : 'Create' }}</Button>
                </DialogFooter>
            </DialogContent>
        </Dialog>
    </SettingsLayout>
</template>
