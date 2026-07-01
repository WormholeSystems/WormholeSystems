<script setup lang="ts">
import PlusIcon from '@/components/icons/PlusIcon.vue';
import SolarsystemEffect from '@/components/map/SolarsystemEffect.vue';
import KillmailFilterEditor from '@/components/maps/webhooks/KillmailFilterEditor.vue';
import SolarsystemClass from '@/components/solarsystem/SolarsystemClass.vue';
import { Badge } from '@/components/ui/badge';
import { Button } from '@/components/ui/button';
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card';
import { Checkbox } from '@/components/ui/checkbox';
import { Combobox, ComboboxAnchor, ComboboxEmpty, ComboboxGroup, ComboboxInput, ComboboxItem, ComboboxList } from '@/components/ui/combobox';
import { Dialog, DialogContent, DialogDescription, DialogFooter, DialogHeader, DialogTitle } from '@/components/ui/dialog';
import { Input } from '@/components/ui/input';
import { Label } from '@/components/ui/label';
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from '@/components/ui/select';
import { Separator } from '@/components/ui/separator';
import { Tooltip, TooltipContent, TooltipTrigger } from '@/components/ui/tooltip';
import { useMapAlerts } from '@/composables/useMapAlerts';
import { useMapWebhookRoles } from '@/composables/useMapWebhookRoles';
import { useMapWebhooks } from '@/composables/useMapWebhooks';
import usePermission from '@/composables/usePermission';
import { useStaticData } from '@/composables/useStaticData';
import SettingsLayout from '@/layouts/SettingsLayout.vue';
import { TMapSummary } from '@/pages/maps';
import { TKillmailFilterMatch, TKillmailFilterRule, TMapAlert, TMapWebhook, TMapWebhookRole, TMapWebhookType } from '@/types/models';
import { TStaticSolarsystem } from '@/types/static-data';
import { AtSign, Bell, Pencil, Swords, Trash2, Webhook } from 'lucide-vue-next';
import { computed, reactive, ref } from 'vue';

const { map } = defineProps<{
    map: TMapSummary;
}>();

const { canManageAccess } = usePermission();
const { webhooks, createWebhook, updateWebhook, deleteWebhook } = useMapWebhooks();
const { roles, createRole, updateRole, deleteRole } = useMapWebhookRoles();
const { alerts, createAlert, updateAlert, deleteAlert } = useMapAlerts();
const { staticData, loadStaticData } = useStaticData();

void loadStaticData();

const solarsystems = computed(() => staticData.value?.solarsystems ?? []);

function findSolarsystem(id: number | null): TStaticSolarsystem | undefined {
    if (!id) return undefined;
    return solarsystems.value.find((solarsystem) => solarsystem.id === id);
}

const webhookName = (id: number) => webhooks.value.find((webhook) => webhook.id === id)?.name ?? 'Unknown webhook';
const roleName = (id: number | null) => (id ? (roles.value.find((role) => role.id === id)?.name ?? 'Unknown role') : null);

// --- Webhook destinations ---

type TWebhookForm = { id: number | null; name: string; discord_webhook_url: string };
const webhookForm = reactive<TWebhookForm>({ id: null, name: '', discord_webhook_url: '' });
const webhookDialogOpen = ref(false);
const editingWebhook = computed(() => webhookForm.id !== null);

function openCreateWebhook() {
    Object.assign(webhookForm, { id: null, name: '', discord_webhook_url: '' });
    webhookDialogOpen.value = true;
}

function openEditWebhook(webhook: TMapWebhook) {
    Object.assign(webhookForm, { id: webhook.id, name: webhook.name, discord_webhook_url: '' });
    webhookDialogOpen.value = true;
}

const canSubmitWebhook = computed(
    () => webhookForm.name.trim().length > 0 && (editingWebhook.value || webhookForm.discord_webhook_url.trim().length > 0),
);

function submitWebhook() {
    if (!canSubmitWebhook.value) return;
    const payload = { name: webhookForm.name.trim(), discord_webhook_url: webhookForm.discord_webhook_url.trim() || undefined };
    const onSuccess = () => (webhookDialogOpen.value = false);
    if (webhookForm.id !== null) {
        updateWebhook(webhookForm.id, payload, { onSuccess });
    } else {
        createWebhook(payload, { onSuccess });
    }
}

// --- Roles ---

type TRoleForm = { id: number | null; name: string; discord_role_id: string };
const roleForm = reactive<TRoleForm>({ id: null, name: '', discord_role_id: '' });
const roleDialogOpen = ref(false);
const editingRole = computed(() => roleForm.id !== null);

function openCreateRole() {
    Object.assign(roleForm, { id: null, name: '', discord_role_id: '' });
    roleDialogOpen.value = true;
}

function openEditRole(role: TMapWebhookRole) {
    Object.assign(roleForm, { id: role.id, name: role.name, discord_role_id: role.discord_role_id });
    roleDialogOpen.value = true;
}

const canSubmitRole = computed(() => roleForm.name.trim().length > 0 && /^\d+$/.test(roleForm.discord_role_id.trim()));

function submitRole() {
    if (!canSubmitRole.value) return;
    const payload = { name: roleForm.name.trim(), discord_role_id: roleForm.discord_role_id.trim() };
    const onSuccess = () => (roleDialogOpen.value = false);
    if (roleForm.id !== null) {
        updateRole(roleForm.id, payload, { onSuccess });
    } else {
        createRole(payload, { onSuccess });
    }
}

// --- Alerts ---

type TAlertForm = {
    id: number | null;
    map_webhook_id: number | null;
    map_webhook_role_id: number | null;
    type: TMapWebhookType;
    target_solarsystem_id: number;
    max_jumps: number;
    filter_match: TKillmailFilterMatch;
    filters: TKillmailFilterRule[];
    is_active: boolean;
};

function emptyAlertForm(type: TMapWebhookType = 'proximity'): TAlertForm {
    return {
        id: null,
        map_webhook_id: webhooks.value[0]?.id ?? null,
        map_webhook_role_id: null,
        type,
        target_solarsystem_id: 0,
        max_jumps: 5,
        filter_match: 'any',
        filters: [],
        is_active: true,
    };
}

const alertForm = reactive<TAlertForm>(emptyAlertForm());
const alertDialogOpen = ref(false);
const search = ref('');
const editingAlert = computed(() => alertForm.id !== null);
const isKillmail = computed(() => alertForm.type === 'killmail');
const selectedTarget = computed(() => findSolarsystem(alertForm.target_solarsystem_id));

const filteredSolarsystems = computed(() => {
    const query = search.value.trim().toLowerCase();
    if (!query) return [] as TStaticSolarsystem[];
    return solarsystems.value.filter((solarsystem) => solarsystem.name.toLowerCase().includes(query)).slice(0, 25);
});

const canSubmitAlert = computed(
    () =>
        alertForm.map_webhook_id !== null &&
        (isKillmail.value || alertForm.target_solarsystem_id > 0) &&
        alertForm.max_jumps >= 1 &&
        alertForm.max_jumps <= 20,
);

function openCreateAlert() {
    Object.assign(alertForm, emptyAlertForm());
    search.value = '';
    alertDialogOpen.value = true;
}

function openEditAlert(alert: TMapAlert) {
    Object.assign(alertForm, emptyAlertForm(alert.type), {
        id: alert.id,
        map_webhook_id: alert.map_webhook_id,
        map_webhook_role_id: alert.map_webhook_role_id,
        target_solarsystem_id: alert.target_solarsystem_id ?? 0,
        max_jumps: alert.max_jumps,
        filter_match: alert.filter_match,
        filters: alert.filters.map((filter) => ({ ...filter, ids: [...filter.ids] })),
        is_active: alert.is_active,
    });
    search.value = findSolarsystem(alert.target_solarsystem_id)?.name ?? '';
    alertDialogOpen.value = true;
}

function handleTargetSelect(solarsystem: TStaticSolarsystem) {
    alertForm.target_solarsystem_id = solarsystem.id;
    search.value = solarsystem.name;
}

function submitAlert() {
    if (!canSubmitAlert.value || alertForm.map_webhook_id === null) return;
    const payload = {
        map_webhook_id: alertForm.map_webhook_id,
        map_webhook_role_id: alertForm.map_webhook_role_id,
        type: alertForm.type,
        target_solarsystem_id: isKillmail.value ? null : alertForm.target_solarsystem_id,
        max_jumps: alertForm.max_jumps,
        filter_match: alertForm.filter_match,
        filters: isKillmail.value ? alertForm.filters : [],
        is_active: alertForm.is_active,
    };
    const onSuccess = () => (alertDialogOpen.value = false);
    if (alertForm.id !== null) {
        updateAlert(alertForm.id, payload, { onSuccess });
    } else {
        createAlert(payload, { onSuccess });
    }
}

const ROLE_NONE = 'none';

// --- Delete confirmation ---

const pendingDelete = ref<{ title: string; description: string; confirm: () => void } | null>(null);

function askDelete(title: string, description: string, confirm: () => void) {
    pendingDelete.value = { title, description, confirm };
}

function confirmPendingDelete() {
    pendingDelete.value?.confirm();
    pendingDelete.value = null;
}
</script>

<template>
    <SettingsLayout :map="map" title="Alerts" description="Get notified in Discord when something happens near your chain">
        <div class="space-y-6">
            <!-- Webhooks -->
            <Card>
                <CardHeader class="flex flex-row items-start justify-between gap-4">
                    <div class="space-y-1.5">
                        <CardTitle class="flex items-center gap-2 text-xl font-semibold">
                            <Webhook class="size-5 text-muted-foreground" />
                            Webhooks
                        </CardTitle>
                        <CardDescription>
                            The Discord channels alerts post to. Add a channel's webhook URL once, then point any number of alerts at it.
                        </CardDescription>
                    </div>
                    <Button v-if="canManageAccess" variant="outline" size="sm" @click="openCreateWebhook">
                        <PlusIcon class="mr-2 h-4 w-4" />
                        Add webhook
                    </Button>
                </CardHeader>
                <CardContent>
                    <p v-if="webhooks.length === 0" class="text-sm text-muted-foreground">No webhooks yet.</p>
                    <ul v-else class="divide-y divide-border rounded-lg border">
                        <li v-for="webhook in webhooks" :key="webhook.id" class="flex items-center gap-3 px-3 py-2">
                            <span class="min-w-0 truncate font-medium">{{ webhook.name }}</span>
                            <span class="text-sm text-muted-foreground">{{ webhook.alerts_count ?? 0 }} alert{{ webhook.alerts_count === 1 ? '' : 's' }}</span>
                            <div v-if="canManageAccess" class="ml-auto flex items-center gap-1">
                                <Button variant="ghost" size="icon" class="text-muted-foreground" @click="openEditWebhook(webhook)">
                                    <Pencil class="h-4 w-4" />
                                </Button>
                                <Tooltip v-if="webhook.alerts_count" :delay-duration="300">
                                    <TooltipTrigger as-child>
                                        <span><Button variant="ghost" size="icon" class="text-muted-foreground" disabled><Trash2 class="h-4 w-4" /></Button></span>
                                    </TooltipTrigger>
                                    <TooltipContent>Remove its alerts before deleting this webhook.</TooltipContent>
                                </Tooltip>
                                <Button
                                    v-else
                                    variant="ghost"
                                    size="icon"
                                    class="text-muted-foreground hover:text-destructive"
                                    @click="askDelete('Delete webhook?', `This removes the “${webhook.name}” webhook.`, () => deleteWebhook(webhook.id))"
                                >
                                    <Trash2 class="h-4 w-4" />
                                </Button>
                            </div>
                        </li>
                    </ul>
                </CardContent>
            </Card>

            <!-- Roles -->
            <Card>
                <CardHeader class="flex flex-row items-start justify-between gap-4">
                    <div class="space-y-1.5">
                        <CardTitle class="flex items-center gap-2 text-xl font-semibold">
                            <AtSign class="size-5 text-muted-foreground" />
                            Roles
                        </CardTitle>
                        <CardDescription>Discord roles an alert can ping. Add a role once, then pick it on any alert that should mention it.</CardDescription>
                    </div>
                    <Button v-if="canManageAccess" variant="outline" size="sm" @click="openCreateRole">
                        <PlusIcon class="mr-2 h-4 w-4" />
                        Add role
                    </Button>
                </CardHeader>
                <CardContent>
                    <p v-if="roles.length === 0" class="text-sm text-muted-foreground">No roles yet.</p>
                    <ul v-else class="divide-y divide-border rounded-lg border">
                        <li v-for="role in roles" :key="role.id" class="flex items-center gap-3 px-3 py-2">
                            <span class="min-w-0 truncate font-medium">{{ role.name }}</span>
                            <span class="font-mono text-xs text-muted-foreground">{{ role.discord_role_id }}</span>
                            <div v-if="canManageAccess" class="ml-auto flex items-center gap-1">
                                <Button variant="ghost" size="icon" class="text-muted-foreground" @click="openEditRole(role)">
                                    <Pencil class="h-4 w-4" />
                                </Button>
                                <Button
                                    variant="ghost"
                                    size="icon"
                                    class="text-muted-foreground hover:text-destructive"
                                    @click="askDelete('Delete role?', `Alerts using “${role.name}” will keep firing, just without a ping.`, () => deleteRole(role.id))"
                                >
                                    <Trash2 class="h-4 w-4" />
                                </Button>
                            </div>
                        </li>
                    </ul>
                </CardContent>
            </Card>

            <!-- Alerts -->
            <Card>
                <CardHeader class="flex flex-row items-start justify-between gap-4">
                    <div class="space-y-1.5">
                        <CardTitle class="flex items-center gap-2 text-xl font-semibold">
                            <Bell class="size-5 text-muted-foreground" />
                            Alerts
                        </CardTitle>
                        <CardDescription>
                            What to be notified about. Each alert picks a webhook to post to and, optionally, a role to ping.
                        </CardDescription>
                    </div>
                    <Tooltip v-if="canManageAccess && webhooks.length === 0" :delay-duration="300">
                        <TooltipTrigger as-child>
                            <span><Button variant="outline" size="sm" disabled><PlusIcon class="mr-2 h-4 w-4" />Add alert</Button></span>
                        </TooltipTrigger>
                        <TooltipContent>Add a webhook first.</TooltipContent>
                    </Tooltip>
                    <Button v-else-if="canManageAccess" variant="outline" size="sm" @click="openCreateAlert">
                        <PlusIcon class="mr-2 h-4 w-4" />
                        Add alert
                    </Button>
                </CardHeader>
                <CardContent>
                    <p v-if="alerts.length === 0" class="text-sm text-muted-foreground">No alerts configured yet.</p>
                    <ul v-else class="divide-y divide-border rounded-lg border">
                        <li v-for="alert in alerts" :key="alert.id" class="flex items-center gap-3 px-3 py-2">
                            <Swords v-if="alert.type === 'killmail'" class="size-4 shrink-0 text-muted-foreground" />
                            <SolarsystemClass
                                v-else-if="findSolarsystem(alert.target_solarsystem_id)"
                                :solarsystem_class="findSolarsystem(alert.target_solarsystem_id)!.class"
                                :name="findSolarsystem(alert.target_solarsystem_id)!.name"
                            />
                            <div class="min-w-0">
                                <div class="flex items-center gap-2">
                                    <span class="truncate font-medium">{{ webhookName(alert.map_webhook_id) }}</span>
                                    <Badge v-if="roleName(alert.map_webhook_role_id)" variant="secondary">@{{ roleName(alert.map_webhook_role_id) }}</Badge>
                                    <Badge v-if="!alert.is_active" variant="outline">Inactive</Badge>
                                </div>
                                <div class="text-sm text-muted-foreground">
                                    <template v-if="alert.type === 'killmail'">
                                        Kills within {{ alert.max_jumps }} jump{{ alert.max_jumps === 1 ? '' : 's' }} of your chain ·
                                        {{ alert.filters.length }} filter{{ alert.filters.length === 1 ? '' : 's' }}
                                    </template>
                                    <template v-else>
                                        {{ findSolarsystem(alert.target_solarsystem_id)?.name ?? alert.target_solarsystem_id }} within
                                        {{ alert.max_jumps }} gate jump{{ alert.max_jumps === 1 ? '' : 's' }}
                                    </template>
                                    <template v-if="alert.last_fired_at"> · last fired {{ new Date(alert.last_fired_at).toLocaleString() }}</template>
                                </div>
                            </div>
                            <div v-if="canManageAccess" class="ml-auto flex items-center gap-1">
                                <Button variant="ghost" size="icon" class="text-muted-foreground" @click="openEditAlert(alert)">
                                    <Pencil class="h-4 w-4" />
                                </Button>
                                <Button
                                    variant="ghost"
                                    size="icon"
                                    class="text-muted-foreground hover:text-destructive"
                                    @click="askDelete('Delete alert?', `This removes the alert posting to “${webhookName(alert.map_webhook_id)}”.`, () => deleteAlert(alert.id))"
                                >
                                    <Trash2 class="h-4 w-4" />
                                </Button>
                            </div>
                        </li>
                    </ul>
                </CardContent>
            </Card>
        </div>

        <!-- Webhook dialog -->
        <Dialog v-model:open="webhookDialogOpen">
            <DialogContent class="sm:max-w-lg">
                <DialogHeader>
                    <DialogTitle>{{ editingWebhook ? 'Edit webhook' : 'Add webhook' }}</DialogTitle>
                    <DialogDescription>A name and the Discord channel's webhook URL.</DialogDescription>
                </DialogHeader>
                <div class="space-y-4 py-1">
                    <div class="space-y-1.5">
                        <Label for="wh-name">Name</Label>
                        <Input id="wh-name" v-model="webhookForm.name" placeholder="e.g. #intel channel" />
                    </div>
                    <div class="space-y-1.5">
                        <Label for="wh-url">Discord webhook URL</Label>
                        <Input id="wh-url" v-model="webhookForm.discord_webhook_url" placeholder="https://discord.com/api/webhooks/…" :type="editingWebhook ? 'text' : 'url'" />
                        <p v-if="editingWebhook" class="text-xs text-muted-foreground">Leave blank to keep the current URL.</p>
                    </div>
                </div>
                <DialogFooter>
                    <Button variant="ghost" @click="webhookDialogOpen = false">Cancel</Button>
                    <Button :disabled="!canSubmitWebhook" @click="submitWebhook">{{ editingWebhook ? 'Save' : 'Create' }}</Button>
                </DialogFooter>
            </DialogContent>
        </Dialog>

        <!-- Role dialog -->
        <Dialog v-model:open="roleDialogOpen">
            <DialogContent class="sm:max-w-lg">
                <DialogHeader>
                    <DialogTitle>{{ editingRole ? 'Edit role' : 'Add role' }}</DialogTitle>
                    <DialogDescription>A name and the Discord role ID to ping.</DialogDescription>
                </DialogHeader>
                <div class="space-y-4 py-1">
                    <div class="space-y-1.5">
                        <Label for="role-name">Name</Label>
                        <Input id="role-name" v-model="roleForm.name" placeholder="e.g. Fleet" />
                    </div>
                    <div class="space-y-1.5">
                        <Label for="role-id">Discord role ID</Label>
                        <Input id="role-id" v-model="roleForm.discord_role_id" inputmode="numeric" placeholder="e.g. 123456789012345678" />
                    </div>
                </div>
                <DialogFooter>
                    <Button variant="ghost" @click="roleDialogOpen = false">Cancel</Button>
                    <Button :disabled="!canSubmitRole" @click="submitRole">{{ editingRole ? 'Save' : 'Create' }}</Button>
                </DialogFooter>
            </DialogContent>
        </Dialog>

        <!-- Alert dialog -->
        <Dialog v-model:open="alertDialogOpen">
            <DialogContent class="max-h-[85vh] overflow-y-auto sm:max-w-2xl">
                <DialogHeader>
                    <DialogTitle>{{ editingAlert ? 'Edit alert' : 'Add alert' }}</DialogTitle>
                    <DialogDescription>Choose where it posts, an optional role to ping, and what triggers it.</DialogDescription>
                </DialogHeader>

                <div class="space-y-6 py-1">
                    <section class="grid gap-4 sm:grid-cols-2">
                        <div class="space-y-1.5">
                            <Label>Webhook</Label>
                            <Select :model-value="alertForm.map_webhook_id?.toString()" @update:model-value="(v) => (alertForm.map_webhook_id = Number(v))">
                                <SelectTrigger class="w-full"><SelectValue placeholder="Select a webhook" /></SelectTrigger>
                                <SelectContent>
                                    <SelectItem v-for="webhook in webhooks" :key="webhook.id" :value="webhook.id.toString()">{{ webhook.name }}</SelectItem>
                                </SelectContent>
                            </Select>
                        </div>
                        <div class="space-y-1.5">
                            <Label>Ping role</Label>
                            <Select
                                :model-value="alertForm.map_webhook_role_id?.toString() ?? ROLE_NONE"
                                @update:model-value="(v) => (alertForm.map_webhook_role_id = v === ROLE_NONE ? null : Number(v))"
                            >
                                <SelectTrigger class="w-full"><SelectValue /></SelectTrigger>
                                <SelectContent>
                                    <SelectItem :value="ROLE_NONE">No ping</SelectItem>
                                    <SelectItem v-for="role in roles" :key="role.id" :value="role.id.toString()">{{ role.name }}</SelectItem>
                                </SelectContent>
                            </Select>
                        </div>
                        <div class="space-y-1.5">
                            <Label>Trigger</Label>
                            <Select :model-value="alertForm.type" @update:model-value="(v) => (alertForm.type = v as TMapWebhookType)">
                                <SelectTrigger class="w-full"><SelectValue /></SelectTrigger>
                                <SelectContent>
                                    <SelectItem value="proximity">Known-space connection</SelectItem>
                                    <SelectItem value="killmail">Killmail in range</SelectItem>
                                </SelectContent>
                            </Select>
                        </div>
                        <div class="flex items-end gap-2 pb-1.5">
                            <Checkbox id="alert-active" :model-value="alertForm.is_active" @update:model-value="(v) => (alertForm.is_active = v === true)" />
                            <Label for="alert-active" class="text-sm font-medium">Active</Label>
                        </div>
                    </section>

                    <Separator />

                    <section class="space-y-3">
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
                                    <ComboboxGroup heading="Search Results" v-if="filteredSolarsystems.length > 0" class="grid grid-cols-[auto_1fr_auto]">
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
                                            <span class="truncate text-muted-foreground" v-if="!solarsystem.class">{{ solarsystem.region?.name }}</span>
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
                                <Label for="alert-jumps">{{ isKillmail ? 'Max jumps from chain' : 'Max gate jumps' }}</Label>
                                <Input id="alert-jumps" type="number" min="1" max="20" v-model.number="alertForm.max_jumps" />
                                <p class="text-xs text-muted-foreground">
                                    {{ isKillmail ? 'Gate and wormhole jumps from any chain system (1–20).' : 'Stargate jumps from the added system to the target (1–20).' }}
                                </p>
                            </div>
                            <div v-if="isKillmail" class="space-y-1.5">
                                <Label>Match</Label>
                                <Select :model-value="alertForm.filter_match" @update:model-value="(v) => (alertForm.filter_match = v as TKillmailFilterMatch)">
                                    <SelectTrigger class="w-full"><SelectValue /></SelectTrigger>
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
                            <KillmailFilterEditor v-model:filters="alertForm.filters" />
                        </section>
                    </template>
                </div>

                <DialogFooter>
                    <Button variant="ghost" @click="alertDialogOpen = false">Cancel</Button>
                    <Button :disabled="!canSubmitAlert" @click="submitAlert">{{ editingAlert ? 'Save' : 'Create' }}</Button>
                </DialogFooter>
            </DialogContent>
        </Dialog>

        <!-- Delete confirmation -->
        <Dialog :open="pendingDelete !== null" @update:open="(open) => !open && (pendingDelete = null)">
            <DialogContent class="sm:max-w-md">
                <DialogHeader>
                    <DialogTitle>{{ pendingDelete?.title }}</DialogTitle>
                    <DialogDescription>{{ pendingDelete?.description }}</DialogDescription>
                </DialogHeader>
                <DialogFooter>
                    <Button variant="ghost" @click="pendingDelete = null">Cancel</Button>
                    <Button variant="destructive" @click="confirmPendingDelete">Delete</Button>
                </DialogFooter>
            </DialogContent>
        </Dialog>
    </SettingsLayout>
</template>
