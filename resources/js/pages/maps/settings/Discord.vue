<script setup lang="ts">
import MapAlertController from '@/actions/App/Http/Controllers/MapAlertController';
import MapAlertStateController from '@/actions/App/Http/Controllers/MapAlertStateController';
import MapDiscordController from '@/actions/App/Http/Controllers/MapDiscordController';
import MapWebhookController from '@/actions/App/Http/Controllers/MapWebhookController';
import MapWebhookRoleController from '@/actions/App/Http/Controllers/MapWebhookRoleController';
import PlusIcon from '@/components/icons/PlusIcon.vue';
import DiscordIcon from '@/components/icons/DiscordIcon.vue';
import KillmailFilterEditor from '@/components/maps/webhooks/KillmailFilterEditor.vue';
import SolarsystemClass from '@/components/solarsystem/SolarsystemClass.vue';
import VirtualizedSolarsystemList from '@/components/solarsystem/VirtualizedSolarsystemList.vue';
import { Badge } from '@/components/ui/badge';
import { Button } from '@/components/ui/button';
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card';
import { Checkbox } from '@/components/ui/checkbox';
import { Combobox, ComboboxAnchor, ComboboxInput } from '@/components/ui/combobox';
import { Dialog, DialogContent, DialogDescription, DialogFooter, DialogHeader, DialogTitle } from '@/components/ui/dialog';
import { Input } from '@/components/ui/input';
import { Label } from '@/components/ui/label';
import { RadioGroup, RadioGroupItem } from '@/components/ui/radio-group';
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from '@/components/ui/select';
import { Separator } from '@/components/ui/separator';
import { Tabs, TabsContent, TabsList, TabsTrigger } from '@/components/ui/tabs';
import { Tooltip, TooltipContent, TooltipTrigger } from '@/components/ui/tooltip';
import { useMapAlerts } from '@/composables/useMapAlerts';
import { useMapWebhookRoles } from '@/composables/useMapWebhookRoles';
import { useMapWebhooks } from '@/composables/useMapWebhooks';
import usePermission from '@/composables/usePermission';
import { useStaticData } from '@/composables/useStaticData';
import { jumpShipLabel, jumpShipTypes, maxRangeLy } from '@/const/jumpShipTypes';
import SettingsLayout from '@/layouts/SettingsLayout.vue';
import { type TComboboxSection } from '@/lib/comboboxSections';
import { alertTriggerLabel } from '@/lib/mapAlerts';
import { MAX_SEARCH_RESULTS, takeRanked } from '@/lib/searchRank';
import { TMapSummary } from '@/pages/maps';
import {
    TJumpShipType,
    TKillmailFilterMatch,
    TKillmailFilterRule,
    TMapAlert,
    TMapAlertEvent,
    TMapWebhook,
    TMapWebhookRole,
    TMapAlertType,
} from '@/types/models';
import { TStaticSolarsystem } from '@/types/static-data';
import { router, useForm } from '@inertiajs/vue3';
import { AtSign, Bell, Bot, ExternalLink, Pencil, Rocket, Route, ShieldCheck, Swords, Trash2, Webhook } from 'lucide-vue-next';
import { computed, ref, watch, type Component } from 'vue';

const { map, tab, botAlerts, alertEvents, discordInviteUrl } = defineProps<{
    map: TMapSummary;
    tab: IntegrationTab;
    botAlerts: TMapAlert[];
    alertEvents: TMapAlertEvent[];
    discordInviteUrl: string | null;
}>();

const { canManageAccess } = usePermission();
const { webhooks, deleteWebhook } = useMapWebhooks();
const { roles, deleteRole } = useMapWebhookRoles();
const { alerts, deleteAlert } = useMapAlerts();
const { staticData, loadStaticData } = useStaticData();
type IntegrationTab = 'bot' | 'webhooks';

function changeIntegrationTab(value: string | number): void {
    const selectedTab: IntegrationTab = value === 'webhooks' ? 'webhooks' : 'bot';
    if (selectedTab === tab) return;

    router.visit(MapDiscordController.show(map.slug, { query: { tab: selectedTab } }), {
        only: ['tab'],
        preserveState: true,
        preserveScroll: true,
    });
}

void loadStaticData();

const solarsystems = computed(() => staticData.value?.solarsystems ?? []);

function findSolarsystem(id: number | null): TStaticSolarsystem | undefined {
    if (!id) return undefined;
    return solarsystems.value.find((solarsystem) => solarsystem.id === id);
}

const webhookName = (id: number | null) => (id ? (webhooks.value.find((webhook) => webhook.id === id)?.name ?? 'Unknown webhook') : 'Webhook');
const roleName = (id: number | null) => (id ? (roles.value.find((role) => role.id === id)?.name ?? 'Unknown role') : null);
const systemName = (id: number | null) => findSolarsystem(id)?.name ?? (id ? String(id) : 'Unknown system');

// --- Webhook destinations ---

type TWebhookForm = { id: number | null; name: string; discord_webhook_url: string };
const webhookForm = useForm<TWebhookForm>({ id: null, name: '', discord_webhook_url: '' });
const webhookDialogOpen = ref(false);
const editingWebhook = computed(() => webhookForm.id !== null);
const webhookErrors = computed(() => Object.values(webhookForm.errors));

function openCreateWebhook() {
    webhookForm.clearErrors();
    Object.assign(webhookForm, { id: null, name: '', discord_webhook_url: '' });
    webhookDialogOpen.value = true;
}

function openEditWebhook(webhook: TMapWebhook) {
    webhookForm.clearErrors();
    Object.assign(webhookForm, { id: webhook.id, name: webhook.name, discord_webhook_url: '' });
    webhookDialogOpen.value = true;
}

const canSubmitWebhook = computed(
    () => webhookForm.name.trim().length > 0 && (editingWebhook.value || webhookForm.discord_webhook_url.trim().length > 0),
);

function submitWebhook() {
    if (!canSubmitWebhook.value) return;
    webhookForm.transform(({ name, discord_webhook_url }) => ({
        name: name.trim(),
        discord_webhook_url: discord_webhook_url.trim() || undefined,
        ...(webhookForm.id === null ? { map_id: map.id } : {}),
    }));
    const options = { preserveScroll: true, onSuccess: () => (webhookDialogOpen.value = false) };
    if (webhookForm.id !== null) {
        webhookForm.put(MapWebhookController.update(webhookForm.id).url, options);
    } else {
        webhookForm.post(MapWebhookController.store().url, options);
    }
}

// --- Roles ---

type TRoleForm = { id: number | null; name: string; mention_type: TMapWebhookRole['mention_type']; discord_role_id: string };
const roleForm = useForm<TRoleForm>({ id: null, name: '', mention_type: 'role', discord_role_id: '' });
const roleDialogOpen = ref(false);
const editingRole = computed(() => roleForm.id !== null);
const roleErrors = computed(() => Object.values(roleForm.errors));

function openCreateRole() {
    roleForm.clearErrors();
    Object.assign(roleForm, { id: null, name: '', mention_type: 'role', discord_role_id: '' });
    roleDialogOpen.value = true;
}

function openEditRole(role: TMapWebhookRole) {
    roleForm.clearErrors();
    Object.assign(roleForm, { id: role.id, name: role.name, mention_type: role.mention_type, discord_role_id: role.discord_role_id });
    roleDialogOpen.value = true;
}

const canSubmitRole = computed(() => roleForm.name.trim().length > 0 && /^\d+$/.test(roleForm.discord_role_id.trim()));

function submitRole() {
    if (!canSubmitRole.value) return;
    roleForm.transform(({ name, mention_type, discord_role_id }) => ({
        name: name.trim(),
        mention_type,
        discord_role_id: discord_role_id.trim(),
        ...(roleForm.id === null ? { map_id: map.id } : {}),
    }));
    const options = { preserveScroll: true, onSuccess: () => (roleDialogOpen.value = false) };
    if (roleForm.id !== null) {
        roleForm.put(MapWebhookRoleController.update(roleForm.id).url, options);
    } else {
        roleForm.post(MapWebhookRoleController.store().url, options);
    }
}

// --- Alerts ---

type TAlertForm = {
    id: number | null;
    map_webhook_id: number | null;
    map_webhook_role_id: number | null;
    mention_mode: 'none' | 'everyone';
    type: TMapAlertType;
    target_solarsystem_id: number;
    origin_solarsystem_id: number;
    ship_type: TJumpShipType;
    jdc_level: number;
    include_highsec: boolean;
    max_jumps: number;
    filter_match: TKillmailFilterMatch;
    filters: TKillmailFilterRule[];
    is_active: boolean;
};

function emptyAlertForm(type: TMapAlertType = 'proximity'): TAlertForm {
    return {
        id: null,
        map_webhook_id: webhooks.value[0]?.id ?? null,
        map_webhook_role_id: null,
        mention_mode: 'none',
        type,
        target_solarsystem_id: 0,
        origin_solarsystem_id: 0,
        ship_type: 'dreadnought',
        jdc_level: 5,
        include_highsec: false,
        max_jumps: 5,
        filter_match: 'any',
        filters: [],
        is_active: true,
    };
}

type TTriggerTypeMeta = { value: TMapAlertType; label: string; description: string; icon: Component };

const triggerTypes: TTriggerTypeMeta[] = [
    {
        value: 'proximity',
        label: 'System near chain',
        description: 'A new connection puts the chain within a few gate jumps of a system you care about.',
        icon: Route,
    },
    {
        value: 'jump_range',
        label: 'Capital jump range',
        description: 'A new known-space exit lands within capital jump range of your target system.',
        icon: Rocket,
    },
    {
        value: 'killmail',
        label: 'Kills near chain',
        description: 'A kill matching your filters happens close to your chain.',
        icon: Swords,
    },
];

const triggerMeta = (type: TMapAlertType): TTriggerTypeMeta => triggerTypes.find((trigger) => trigger.value === type) ?? triggerTypes[0];

type TAlertDialogTab = 'trigger' | 'conditions' | 'delivery';

const alertForm = useForm<TAlertForm>(emptyAlertForm());
const alertDialogOpen = ref(false);
const alertDialogTab = ref<TAlertDialogTab>('trigger');
const search = ref('');
const originSearch = ref('');
const editingAlert = computed(() => alertForm.id !== null);
const alertErrors = computed(() => Object.values(alertForm.errors));
const isKillmail = computed(() => alertForm.type === 'killmail');
const isJumpRange = computed(() => alertForm.type === 'jump_range');
const selectedTarget = computed(() => findSolarsystem(alertForm.target_solarsystem_id));
const selectedOrigin = computed(() => findSolarsystem(alertForm.origin_solarsystem_id));
const formJumpRangeLy = computed(() => maxRangeLy(alertForm.ship_type, alertForm.jdc_level));

const filteredSolarsystems = computed(() => {
    const query = search.value.trim().toLowerCase();
    if (!query) return [] as TStaticSolarsystem[];
    return takeRanked(
        solarsystems.value,
        query,
        MAX_SEARCH_RESULTS,
        (solarsystem) => [solarsystem.name],
        (solarsystem) => solarsystem.name,
    );
});

const filteredOriginSolarsystems = computed(() => {
    const query = originSearch.value.trim().toLowerCase();
    if (!query) return [] as TStaticSolarsystem[];
    return takeRanked(
        solarsystems.value,
        query,
        MAX_SEARCH_RESULTS,
        (solarsystem) => [solarsystem.name],
        (solarsystem) => solarsystem.name,
    );
});

const origin_search_sections = computed<TComboboxSection<TStaticSolarsystem>[]>(() => [
    { key: 'results', heading: 'Search Results', items: filteredOriginSolarsystems.value },
]);

const search_sections = computed<TComboboxSection<TStaticSolarsystem>[]>(() => [
    { key: 'results', heading: 'Search Results', items: filteredSolarsystems.value },
]);

const canSubmitAlert = computed(
    () =>
        alertForm.map_webhook_id !== null &&
        (isKillmail.value || alertForm.target_solarsystem_id > 0) &&
        (isJumpRange.value || (alertForm.max_jumps >= 1 && alertForm.max_jumps <= 20)) &&
        (!isKillmail.value || alertForm.filters.every((filter) => filter.ids.length > 0)),
);

watch(search, (value) => {
    if (selectedTarget.value && value.trim() !== selectedTarget.value.name) {
        alertForm.target_solarsystem_id = 0;
    }
});

watch(originSearch, (value) => {
    if (selectedOrigin.value && value.trim() !== selectedOrigin.value.name) {
        alertForm.origin_solarsystem_id = 0;
    }
});

function openCreateAlert() {
    alertForm.clearErrors();
    Object.assign(alertForm, emptyAlertForm());
    search.value = '';
    originSearch.value = '';
    alertDialogTab.value = 'trigger';
    alertDialogOpen.value = true;
}

function openEditAlert(alert: TMapAlert) {
    alertForm.clearErrors();
    Object.assign(alertForm, emptyAlertForm(alert.type), {
        id: alert.id,
        map_webhook_id: alert.map_webhook_id,
        map_webhook_role_id: alert.map_webhook_role_id,
        mention_mode: alert.mention_mode === 'everyone' ? 'everyone' : 'none',
        target_solarsystem_id: alert.target_solarsystem_id ?? 0,
        origin_solarsystem_id: alert.origin_solarsystem_id ?? 0,
        ship_type: alert.ship_type ?? 'dreadnought',
        jdc_level: alert.jdc_level ?? 5,
        include_highsec: alert.include_highsec,
        max_jumps: alert.max_jumps ?? 5,
        filter_match: alert.filter_match,
        filters: alert.filters.map((filter) => ({ ...filter, ids: [...filter.ids] })),
        is_active: alert.is_active,
    });
    search.value = findSolarsystem(alert.target_solarsystem_id)?.name ?? '';
    originSearch.value = findSolarsystem(alert.origin_solarsystem_id)?.name ?? '';
    alertDialogTab.value = 'trigger';
    alertDialogOpen.value = true;
}

function handleTargetSelect(solarsystem: TStaticSolarsystem) {
    alertForm.target_solarsystem_id = solarsystem.id;
    search.value = solarsystem.name;
}

function handleOriginSelect(solarsystem: TStaticSolarsystem) {
    alertForm.origin_solarsystem_id = solarsystem.id;
    originSearch.value = solarsystem.name;
}

function submitAlert() {
    if (!canSubmitAlert.value || alertForm.map_webhook_id === null) return;
    const alertId = alertForm.id;
    alertForm.transform(() => ({
        map_webhook_id: alertForm.map_webhook_id,
        map_webhook_role_id: alertForm.mention_mode === 'everyone' ? null : alertForm.map_webhook_role_id,
        mention_mode: alertForm.mention_mode,
        type: alertForm.type,
        target_solarsystem_id: isKillmail.value ? null : alertForm.target_solarsystem_id,
        origin_solarsystem_id: alertForm.type === 'proximity' && alertForm.origin_solarsystem_id > 0 ? alertForm.origin_solarsystem_id : null,
        ship_type: isJumpRange.value ? alertForm.ship_type : null,
        jdc_level: isJumpRange.value ? alertForm.jdc_level : null,
        include_highsec: isJumpRange.value ? alertForm.include_highsec : false,
        max_jumps: isJumpRange.value ? null : alertForm.max_jumps,
        filter_match: alertForm.filter_match,
        filters: isKillmail.value ? alertForm.filters : [],
        is_active: alertForm.is_active,
        ...(alertId === null ? { map_id: map.id } : {}),
    }));
    const options = { preserveScroll: true, onSuccess: () => (alertDialogOpen.value = false) };
    if (alertId !== null) {
        alertForm.put(MapAlertController.update(alertId).url, options);
    } else {
        alertForm.post(MapAlertController.store().url, options);
    }
}

const ROLE_NONE = 'none';
const PING_EVERYONE = 'everyone';

const alertPingValue = computed(() =>
    alertForm.mention_mode === 'everyone' ? PING_EVERYONE : (alertForm.map_webhook_role_id?.toString() ?? ROLE_NONE),
);

function setAlertPing(value: unknown): void {
    if (value === PING_EVERYONE) {
        alertForm.mention_mode = 'everyone';
        alertForm.map_webhook_role_id = null;
        return;
    }

    alertForm.mention_mode = 'none';
    alertForm.map_webhook_role_id = value === ROLE_NONE ? null : Number(value);
}

const formatDate = (date: string | null): string => (date ? new Date(date).toLocaleString() : 'Never');
const setBotAlertState = (alert: TMapAlert): void =>
    router.visit(MapAlertStateController.update(alert.id), {
        data: { enabled: !alert.is_active },
        preserveScroll: true,
    });

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
    <SettingsLayout :map="map" title="Discord" description="Manage Discord alerts, delivery destinations, mentions, and oversight">
        <div class="space-y-6">
            <section
                class="relative overflow-hidden rounded-xl border border-indigo-400/25 bg-indigo-600 px-6 pt-7 pb-20 text-white shadow-lg shadow-indigo-950/15 sm:px-8 sm:pr-44"
            >
                <div class="absolute -top-16 -right-12 size-48 rounded-full bg-white/10 blur-2xl"></div>
                <div class="relative flex items-start gap-4">
                    <div class="flex size-12 shrink-0 items-center justify-center rounded-xl bg-white/15 ring-1 ring-white/20">
                        <DiscordIcon class="size-7" />
                    </div>
                    <div class="max-w-2xl space-y-2">
                        <p class="text-xs font-semibold tracking-[0.18em] text-indigo-100 uppercase">Discord integration</p>
                        <h2 class="text-2xl font-semibold tracking-tight">Keep your crew informed</h2>
                        <p class="text-sm leading-6 text-indigo-100">
                            Turn chain activity and nearby threats into focused Discord alerts, then control exactly where they land and who they
                            mention.
                        </p>
                    </div>
                </div>
                <Button v-if="discordInviteUrl" as-child class="absolute right-6 bottom-6 bg-white text-black shadow-sm hover:bg-white/90 sm:right-8">
                    <a :href="discordInviteUrl" target="_blank" rel="noopener noreferrer">
                        Invite bot
                        <ExternalLink class="size-4" />
                    </a>
                </Button>
            </section>

            <Tabs :model-value="tab" class="space-y-6" @update:model-value="changeIntegrationTab">
                <TabsList class="grid h-auto w-full grid-cols-2 p-1">
                    <TabsTrigger value="bot" data-testid="discord-tab-bot" class="gap-2 py-2.5">
                        <Bot class="size-4" />
                        Discord bot
                    </TabsTrigger>
                    <TabsTrigger value="webhooks" data-testid="discord-tab-webhooks" class="gap-2 py-2.5">
                        <Webhook class="size-4" />
                        Webhooks
                    </TabsTrigger>
                </TabsList>

                <TabsContent value="webhooks" class="mt-0 space-y-6">
                    <!-- Alert rules -->
                    <Card class="gap-0 py-0">
                        <CardHeader class="flex flex-col items-start justify-between gap-4 border-b py-4 sm:flex-row sm:items-center">
                            <div class="space-y-1">
                                <CardTitle class="flex items-center gap-2 text-lg">
                                    <Bell class="size-5 text-indigo-400" />
                                    Webhook alerts
                                </CardTitle>
                                <CardDescription>Rules delivered through the webhook destinations configured below.</CardDescription>
                            </div>
                            <div class="flex shrink-0 items-center gap-3">
                                <Badge variant="outline">{{ alerts.length }}</Badge>
                                <Tooltip v-if="canManageAccess && webhooks.length === 0" :delay-duration="300">
                                    <TooltipTrigger as-child>
                                        <span
                                            ><Button variant="outline" size="sm" disabled><PlusIcon class="mr-2 h-4 w-4" />Add alert</Button></span
                                        >
                                    </TooltipTrigger>
                                    <TooltipContent>Add a webhook destination first.</TooltipContent>
                                </Tooltip>
                                <Button v-else-if="canManageAccess" variant="outline" size="sm" @click="openCreateAlert">
                                    <PlusIcon class="mr-2 h-4 w-4" />
                                    Add alert
                                </Button>
                            </div>
                        </CardHeader>
                        <CardContent class="p-4">
                            <p v-if="alerts.length === 0" class="text-sm text-muted-foreground">No alert rules configured yet.</p>
                            <ul v-else class="divide-y divide-border/60">
                                <li
                                    v-for="alert in alerts"
                                    :key="alert.id"
                                    class="flex flex-col gap-3 py-3 first:pt-0 last:pb-0 sm:flex-row sm:items-start"
                                >
                                    <div class="min-w-0 flex-1 space-y-1">
                                        <div class="flex flex-wrap items-center gap-2">
                                            <component :is="triggerMeta(alert.type).icon" class="size-4 shrink-0 text-muted-foreground" />
                                            <span class="font-medium text-foreground">{{ alertTriggerLabel(alert, systemName) }}</span>
                                            <Badge v-if="!alert.is_active" variant="outline">Inactive</Badge>
                                        </div>
                                        <p class="text-xs text-muted-foreground">
                                            {{ triggerMeta(alert.type).label }}
                                            <template v-if="alert.type === 'jump_range'">
                                                · {{ jumpShipLabel(alert.ship_type ?? 'dreadnought') }} (JDC {{ alert.jdc_level ?? 5 }})</template
                                            >
                                            <template v-if="alert.type === 'killmail'">
                                                · {{ alert.filters.length }} filter{{ alert.filters.length === 1 ? '' : 's' }}</template
                                            >
                                            · posts to {{ webhookName(alert.map_webhook_id) }}
                                            <template v-if="alert.mention_mode === 'everyone'"> · pings @everyone</template>
                                            <template v-else-if="roleName(alert.map_webhook_role_id)">
                                                · pings @{{ roleName(alert.map_webhook_role_id) }}</template
                                            >
                                            <template v-if="alert.last_fired_at">
                                                · last fired {{ new Date(alert.last_fired_at).toLocaleString() }}</template
                                            >
                                        </p>
                                    </div>
                                    <div v-if="canManageAccess" class="flex shrink-0 items-center gap-1">
                                        <Button
                                            variant="ghost"
                                            size="icon"
                                            class="text-muted-foreground"
                                            :aria-label="`Edit ${triggerMeta(alert.type).label} alert`"
                                            @click="openEditAlert(alert)"
                                        >
                                            <Pencil class="h-4 w-4" />
                                        </Button>
                                        <Button
                                            variant="ghost"
                                            size="icon"
                                            class="text-muted-foreground hover:text-destructive"
                                            :aria-label="`Delete ${triggerMeta(alert.type).label} alert`"
                                            @click="
                                                askDelete(
                                                    'Delete alert?',
                                                    `This removes the alert posting to “${webhookName(alert.map_webhook_id)}”.`,
                                                    () => deleteAlert(alert.id, { only: ['webhooks', 'roles', 'alerts', 'alertEvents'] }),
                                                )
                                            "
                                        >
                                            <Trash2 class="h-4 w-4" />
                                        </Button>
                                    </div>
                                </li>
                            </ul>
                        </CardContent>
                    </Card>

                    <div class="grid items-start gap-6 lg:grid-cols-2">
                        <!-- Delivery destinations -->
                        <Card class="gap-0 py-0">
                            <CardHeader class="flex flex-col items-start justify-between gap-4 border-b py-4 sm:flex-row sm:items-center">
                                <div class="space-y-1">
                                    <CardTitle class="flex items-center gap-2 text-lg">
                                        <Webhook class="size-5 text-muted-foreground" />
                                        Delivery destinations
                                    </CardTitle>
                                    <CardDescription
                                        >Add Discord channel webhook URLs, then reuse each destination across alert rules.</CardDescription
                                    >
                                </div>
                                <Button v-if="canManageAccess" variant="outline" size="sm" class="shrink-0" @click="openCreateWebhook">
                                    <PlusIcon class="mr-2 h-4 w-4" />
                                    Add webhook
                                </Button>
                            </CardHeader>
                            <CardContent class="p-4">
                                <p v-if="webhooks.length === 0" class="text-sm text-muted-foreground">No delivery destinations yet.</p>
                                <ul v-else class="divide-y divide-border/60">
                                    <li v-for="webhook in webhooks" :key="webhook.id" class="flex items-center gap-3 py-2.5 first:pt-0 last:pb-0">
                                        <span class="min-w-0 truncate font-medium">{{ webhook.name }}</span>
                                        <span class="text-sm text-muted-foreground"
                                            >{{ webhook.alerts_count ?? 0 }} alert{{ webhook.alerts_count === 1 ? '' : 's' }}</span
                                        >
                                        <div v-if="canManageAccess" class="ml-auto flex items-center gap-1">
                                            <Button
                                                variant="ghost"
                                                size="icon"
                                                class="text-muted-foreground"
                                                :aria-label="`Edit ${webhook.name} webhook`"
                                                @click="openEditWebhook(webhook)"
                                            >
                                                <Pencil class="h-4 w-4" />
                                            </Button>
                                            <Tooltip v-if="webhook.alerts_count" :delay-duration="300">
                                                <TooltipTrigger as-child>
                                                    <span
                                                        ><Button
                                                            variant="ghost"
                                                            size="icon"
                                                            class="text-muted-foreground"
                                                            :aria-label="`Delete ${webhook.name} webhook`"
                                                            disabled
                                                            ><Trash2 class="h-4 w-4" /></Button
                                                    ></span>
                                                </TooltipTrigger>
                                                <TooltipContent>Remove its alerts before deleting this webhook.</TooltipContent>
                                            </Tooltip>
                                            <Button
                                                v-else
                                                variant="ghost"
                                                size="icon"
                                                class="text-muted-foreground hover:text-destructive"
                                                :aria-label="`Delete ${webhook.name} webhook`"
                                                @click="
                                                    askDelete('Delete webhook?', `This removes the “${webhook.name}” webhook.`, () =>
                                                        deleteWebhook(webhook.id),
                                                    )
                                                "
                                            >
                                                <Trash2 class="h-4 w-4" />
                                            </Button>
                                        </div>
                                    </li>
                                </ul>
                            </CardContent>
                        </Card>

                        <!-- Mentions -->
                        <Card class="gap-0 py-0">
                            <CardHeader class="flex flex-col items-start justify-between gap-4 border-b py-4 sm:flex-row sm:items-center">
                                <div class="space-y-1">
                                    <CardTitle class="flex items-center gap-2 text-lg">
                                        <AtSign class="size-5 text-muted-foreground" />
                                        Webhook mentions
                                    </CardTitle>
                                    <CardDescription
                                        >Reusable role or user pings for webhook alerts. Bot alerts pick mentions in Discord.</CardDescription
                                    >
                                </div>
                                <Button v-if="canManageAccess" variant="outline" size="sm" class="shrink-0" @click="openCreateRole">
                                    <PlusIcon class="mr-2 h-4 w-4" />
                                    Add mention
                                </Button>
                            </CardHeader>
                            <CardContent class="p-4">
                                <p v-if="roles.length === 0" class="text-sm text-muted-foreground">No mentions yet.</p>
                                <ul v-else class="divide-y divide-border/60">
                                    <li v-for="role in roles" :key="role.id" class="flex items-center gap-3 py-2.5 first:pt-0 last:pb-0">
                                        <span class="min-w-0 truncate font-medium">@{{ role.name }}</span>
                                        <Badge variant="outline" class="shrink-0 capitalize">{{ role.mention_type }}</Badge>
                                        <span class="font-mono text-xs text-muted-foreground">{{ role.discord_role_id }}</span>
                                        <div v-if="canManageAccess" class="ml-auto flex items-center gap-1">
                                            <Button
                                                variant="ghost"
                                                size="icon"
                                                class="text-muted-foreground"
                                                :aria-label="`Edit ${role.name} mention`"
                                                @click="openEditRole(role)"
                                            >
                                                <Pencil class="h-4 w-4" />
                                            </Button>
                                            <Button
                                                variant="ghost"
                                                size="icon"
                                                class="text-muted-foreground hover:text-destructive"
                                                :aria-label="`Delete ${role.name} mention`"
                                                @click="
                                                    askDelete(
                                                        'Delete mention?',
                                                        `Alerts using “${role.name}” will keep firing, just without a ping.`,
                                                        () => deleteRole(role.id),
                                                    )
                                                "
                                            >
                                                <Trash2 class="h-4 w-4" />
                                            </Button>
                                        </div>
                                    </li>
                                </ul>
                            </CardContent>
                        </Card>
                    </div>
                </TabsContent>

                <TabsContent value="bot" class="mt-0 space-y-6">
                    <Card class="gap-0 py-0">
                        <CardHeader class="flex flex-col items-start justify-between gap-4 border-b py-4 sm:flex-row sm:items-center">
                            <div class="space-y-1">
                                <CardTitle class="flex items-center gap-2 text-lg"><Bot class="size-5 text-indigo-400" />Bot alerts</CardTitle>
                                <CardDescription
                                    >Alerts created with <code>/alerts</code>, including private alerts visible to managers.</CardDescription
                                >
                            </div>
                            <Badge variant="outline">{{ botAlerts.length }}</Badge>
                        </CardHeader>
                        <CardContent class="p-4">
                            <p v-if="!botAlerts.length" class="text-sm text-muted-foreground">No bot alerts for this map.</p>
                            <ul v-else class="divide-y divide-border/60">
                                <li
                                    v-for="alert in botAlerts"
                                    :key="alert.id"
                                    class="flex flex-col gap-3 py-3 first:pt-0 last:pb-0 sm:flex-row sm:items-start"
                                >
                                    <div class="min-w-0 flex-1 space-y-1">
                                        <div class="flex flex-wrap items-center gap-2">
                                            <span class="font-medium text-foreground">{{ alertTriggerLabel(alert) }}</span>
                                            <Badge :variant="alert.is_active ? 'secondary' : 'outline'">{{
                                                alert.is_active ? 'Active' : 'Disabled'
                                            }}</Badge>
                                        </div>
                                        <p class="text-xs text-muted-foreground">
                                            {{ alert.creator_name ?? 'Unknown creator' }} · {{ alert.destination_summary }} ·
                                            {{ alert.mention_mode_label }}
                                        </p>
                                        <p v-if="alert.disabled_reason_label" class="text-xs text-amber-600">
                                            {{ alert.disabled_reason_label }}
                                        </p>
                                        <p class="text-xs text-muted-foreground">
                                            Created {{ formatDate(alert.created_at) }} · Last fired {{ formatDate(alert.last_fired_at) }}
                                        </p>
                                    </div>
                                    <div class="flex shrink-0 gap-2">
                                        <Button variant="outline" size="sm" @click="setBotAlertState(alert)">{{
                                            alert.is_active ? 'Disable' : 'Enable'
                                        }}</Button>
                                        <Button
                                            variant="ghost"
                                            size="sm"
                                            class="text-destructive"
                                            @click="
                                                askDelete(
                                                    'Remove bot alert?',
                                                    'This removes the bot-created alert and retains its security history.',
                                                    () => deleteAlert(alert.id, { only: ['botAlerts', 'alertEvents'] }),
                                                )
                                            "
                                        >
                                            Remove
                                        </Button>
                                    </div>
                                </li>
                            </ul>
                        </CardContent>
                    </Card>

                    <!-- Security history -->
                    <details class="rounded-lg border border-border/50 bg-muted/5">
                        <summary
                            class="flex cursor-pointer list-none items-center gap-2 px-4 py-3 text-xs font-medium tracking-wider text-muted-foreground uppercase"
                        >
                            <ShieldCheck class="size-4" />
                            Security history ({{ alertEvents.length }})
                        </summary>
                        <p v-if="!alertEvents.length" class="border-t border-border/50 px-4 py-3 text-sm text-muted-foreground">
                            No disabled or removed alert activity recorded.
                        </p>
                        <ul v-else class="divide-y divide-border/50 border-t border-border/50 px-4">
                            <li
                                v-for="event in alertEvents"
                                :key="event.id"
                                class="flex flex-wrap items-center gap-x-2 gap-y-1 py-2 text-xs text-muted-foreground"
                            >
                                <Badge variant="outline" class="capitalize">{{ event.action }}</Badge>
                                <span>{{ event.destination_summary }}</span>
                                <span v-if="event.reason">· {{ event.reason.replaceAll('_', ' ') }}</span>
                                <span v-if="event.actor_name">· by {{ event.actor_name }}</span>
                                <time class="sm:ml-auto">{{ formatDate(event.created_at) }}</time>
                            </li>
                        </ul>
                    </details>
                </TabsContent>
            </Tabs>
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
                        <Input
                            id="wh-url"
                            v-model="webhookForm.discord_webhook_url"
                            placeholder="https://discord.com/api/webhooks/…"
                            :type="editingWebhook ? 'text' : 'url'"
                        />
                        <p v-if="editingWebhook" class="text-xs text-muted-foreground">Leave blank to keep the current URL.</p>
                    </div>
                    <div v-if="webhookErrors.length" class="space-y-1" role="alert">
                        <p v-for="error in webhookErrors" :key="error" class="text-sm text-destructive">{{ error }}</p>
                    </div>
                </div>
                <DialogFooter>
                    <Button variant="ghost" @click="webhookDialogOpen = false">Cancel</Button>
                    <Button :disabled="!canSubmitWebhook || webhookForm.processing" @click="submitWebhook">
                        {{ webhookForm.processing ? 'Saving…' : editingWebhook ? 'Save' : 'Create' }}
                    </Button>
                </DialogFooter>
            </DialogContent>
        </Dialog>

        <!-- Mention dialog -->
        <Dialog v-model:open="roleDialogOpen">
            <DialogContent class="sm:max-w-lg">
                <DialogHeader>
                    <DialogTitle>{{ editingRole ? 'Edit mention' : 'Add mention' }}</DialogTitle>
                    <DialogDescription>A name and the Discord role or user to ping.</DialogDescription>
                </DialogHeader>
                <div class="space-y-4 py-1">
                    <div class="space-y-1.5">
                        <Label for="role-name">Name</Label>
                        <Input id="role-name" v-model="roleForm.name" placeholder="e.g. Fleet" />
                    </div>
                    <div class="grid gap-4 sm:grid-cols-2">
                        <div class="space-y-1.5">
                            <Label for="mention-type">Type</Label>
                            <Select
                                :model-value="roleForm.mention_type"
                                @update:model-value="(v) => (roleForm.mention_type = v === 'user' ? 'user' : 'role')"
                            >
                                <SelectTrigger id="mention-type" class="w-full"><SelectValue /></SelectTrigger>
                                <SelectContent>
                                    <SelectItem value="role">Role</SelectItem>
                                    <SelectItem value="user">User</SelectItem>
                                </SelectContent>
                            </Select>
                        </div>
                        <div class="space-y-1.5">
                            <Label for="role-id">{{ roleForm.mention_type === 'user' ? 'Discord user ID' : 'Discord role ID' }}</Label>
                            <Input id="role-id" v-model="roleForm.discord_role_id" inputmode="numeric" placeholder="e.g. 123456789012345678" />
                        </div>
                    </div>
                    <div v-if="roleErrors.length" class="space-y-1" role="alert">
                        <p v-for="error in roleErrors" :key="error" class="text-sm text-destructive">{{ error }}</p>
                    </div>
                </div>
                <DialogFooter>
                    <Button variant="ghost" @click="roleDialogOpen = false">Cancel</Button>
                    <Button :disabled="!canSubmitRole || roleForm.processing" @click="submitRole">
                        {{ roleForm.processing ? 'Saving…' : editingRole ? 'Save' : 'Create' }}
                    </Button>
                </DialogFooter>
            </DialogContent>
        </Dialog>

        <!-- Alert dialog -->
        <Dialog v-model:open="alertDialogOpen">
            <DialogContent class="flex h-[min(640px,85dvh)] flex-col gap-0 overflow-hidden p-0 sm:max-w-3xl">
                <DialogHeader class="shrink-0 border-b px-6 py-4">
                    <DialogTitle>{{ editingAlert ? 'Edit alert' : 'Add alert' }}</DialogTitle>
                    <DialogDescription>Pick what triggers it, tune the range, then choose where it posts.</DialogDescription>
                </DialogHeader>

                <Tabs v-model="alertDialogTab" class="flex min-h-0 flex-1 flex-col gap-0">
                    <div class="shrink-0 px-6 pt-4">
                        <TabsList class="grid w-full grid-cols-3">
                            <TabsTrigger value="trigger">Trigger</TabsTrigger>
                            <TabsTrigger value="conditions">{{ isKillmail ? 'Matching' : 'Range' }}</TabsTrigger>
                            <TabsTrigger value="delivery">Delivery</TabsTrigger>
                        </TabsList>
                    </div>

                    <TabsContent value="trigger" class="mt-0 min-h-0 flex-1 overflow-y-auto px-6 py-4">
                        <RadioGroup
                            :model-value="alertForm.type"
                            class="grid gap-2"
                            aria-label="Trigger type"
                            @update:model-value="(value) => (alertForm.type = value as TMapAlertType)"
                        >
                            <Label
                                v-for="trigger in triggerTypes"
                                :key="trigger.value"
                                :for="`trigger-${trigger.value}`"
                                class="flex cursor-pointer items-start gap-3 rounded-lg border p-3 text-left transition-colors has-[:focus-visible]:ring-2 has-[:focus-visible]:ring-ring"
                                :class="
                                    alertForm.type === trigger.value
                                        ? 'border-primary/60 bg-accent/50 ring-1 ring-primary/60'
                                        : 'border-border hover:bg-accent/30'
                                "
                            >
                                <RadioGroupItem :id="`trigger-${trigger.value}`" :value="trigger.value" class="mt-0.5" />
                                <component :is="trigger.icon" class="mt-0.5 size-4 shrink-0 text-muted-foreground" />
                                <span class="min-w-0">
                                    <span class="block text-sm font-medium">{{ trigger.label }}</span>
                                    <span class="block text-xs font-normal text-muted-foreground">{{ trigger.description }}</span>
                                </span>
                            </Label>
                        </RadioGroup>
                    </TabsContent>

                    <TabsContent value="conditions" class="mt-0 min-h-0 flex-1 space-y-3 overflow-y-auto px-6 py-4">
                        <div v-if="!isKillmail" class="grid gap-4 sm:grid-cols-2">
                            <div class="space-y-1.5">
                                <Label for="alert-target-system">Target system</Label>
                                <div v-if="selectedTarget" class="flex items-center gap-2 text-sm">
                                    <SolarsystemClass :solarsystem_class="selectedTarget.class" :name="selectedTarget.name" />
                                    <span class="font-medium">{{ selectedTarget.name }}</span>
                                </div>
                                <Combobox class="rounded-lg border bg-neutral-900" :ignore-filter="true">
                                    <ComboboxAnchor>
                                        <ComboboxInput id="alert-target-system" v-model="search" placeholder="Search for a system…" />
                                    </ComboboxAnchor>
                                    <VirtualizedSolarsystemList align="start" :sections="search_sections" @select="handleTargetSelect" />
                                </Combobox>
                                <p v-if="isJumpRange" class="text-xs text-muted-foreground">
                                    Jump range is measured between this system and new exits on the map.
                                </p>
                            </div>
                            <div v-if="!isJumpRange" class="space-y-1.5">
                                <Label for="alert-jumps">Max gate jumps</Label>
                                <Input id="alert-jumps" type="number" min="1" max="20" v-model.number="alertForm.max_jumps" />
                                <p class="text-xs text-muted-foreground">Stargate jumps from the added system to the target (1–20).</p>
                            </div>
                        </div>

                        <div v-if="!isKillmail && !isJumpRange" class="space-y-1.5">
                            <Label for="alert-origin-system">Starting point (optional)</Label>
                            <div v-if="selectedOrigin" class="flex items-center gap-2 text-sm">
                                <SolarsystemClass :solarsystem_class="selectedOrigin.class" :name="selectedOrigin.name" />
                                <span class="font-medium">{{ selectedOrigin.name }}</span>
                            </div>
                            <Combobox class="rounded-lg border bg-neutral-900" :ignore-filter="true">
                                <ComboboxAnchor>
                                    <ComboboxInput id="alert-origin-system" v-model="originSearch" placeholder="Anywhere on the chain" />
                                </ComboboxAnchor>
                                <VirtualizedSolarsystemList align="start" :sections="origin_search_sections" @select="handleOriginSelect" />
                            </Combobox>
                            <p class="text-xs text-muted-foreground">
                                Measure from this system through the chain instead of from each newly added system.
                            </p>
                        </div>

                        <template v-if="isJumpRange">
                            <div class="grid gap-4 sm:grid-cols-2">
                                <div class="space-y-1.5">
                                    <Label for="alert-ship-class">Ship class</Label>
                                    <Select
                                        :model-value="alertForm.ship_type"
                                        @update:model-value="(v) => (alertForm.ship_type = v as TJumpShipType)"
                                    >
                                        <SelectTrigger id="alert-ship-class" class="w-full"><SelectValue /></SelectTrigger>
                                        <SelectContent>
                                            <SelectItem v-for="ship in jumpShipTypes" :key="ship.value" :value="ship.value">{{
                                                ship.label
                                            }}</SelectItem>
                                        </SelectContent>
                                    </Select>
                                </div>
                                <div class="space-y-1.5">
                                    <Label for="alert-jdc-level">Jump Drive Calibration</Label>
                                    <Select
                                        :model-value="alertForm.jdc_level.toString()"
                                        @update:model-value="(v) => (alertForm.jdc_level = Number(v))"
                                    >
                                        <SelectTrigger id="alert-jdc-level" class="w-full"><SelectValue /></SelectTrigger>
                                        <SelectContent>
                                            <SelectItem v-for="level in [1, 2, 3, 4, 5]" :key="level" :value="level.toString()"
                                                >Level {{ level }}</SelectItem
                                            >
                                        </SelectContent>
                                    </Select>
                                </div>
                            </div>
                            <p class="text-xs text-muted-foreground">
                                Alerts on exits within <span class="font-medium text-foreground">{{ formJumpRangeLy.toFixed(1) }} ly</span> of the
                                target system.
                            </p>
                            <div class="flex items-center gap-2">
                                <Checkbox
                                    id="alert-highsec"
                                    :model-value="alertForm.include_highsec"
                                    @update:model-value="(v) => (alertForm.include_highsec = v === true)"
                                />
                                <Label for="alert-highsec" class="text-sm font-medium">Include high-sec exits</Label>
                            </div>
                            <p class="text-xs text-muted-foreground">
                                Cynos can't be lit in high-sec, so high-sec exits are skipped unless you need them (e.g. jump freighters jumping out).
                            </p>
                        </template>

                        <div v-if="isKillmail" class="grid gap-4 sm:grid-cols-2">
                            <div class="space-y-1.5">
                                <Label for="alert-km-jumps">Max jumps from chain</Label>
                                <Input id="alert-km-jumps" type="number" min="1" max="20" v-model.number="alertForm.max_jumps" />
                                <p class="text-xs text-muted-foreground">Gate and wormhole jumps from any chain system (1–20).</p>
                            </div>
                            <div class="space-y-1.5">
                                <Label for="alert-filter-match">Match</Label>
                                <Select
                                    :model-value="alertForm.filter_match"
                                    @update:model-value="(v) => (alertForm.filter_match = v as TKillmailFilterMatch)"
                                >
                                    <SelectTrigger id="alert-filter-match" class="w-full"><SelectValue /></SelectTrigger>
                                    <SelectContent>
                                        <SelectItem value="any">Match any filter</SelectItem>
                                        <SelectItem value="all">Match all filters</SelectItem>
                                    </SelectContent>
                                </Select>
                                <p class="text-xs text-muted-foreground">"Any" = at least one filter; "all" = every filter. Excludes always block.</p>
                            </div>
                        </div>
                        <template v-if="isKillmail">
                            <Separator />
                            <KillmailFilterEditor v-model:filters="alertForm.filters" />
                        </template>
                    </TabsContent>

                    <TabsContent value="delivery" class="mt-0 min-h-0 flex-1 space-y-3 overflow-y-auto px-6 py-4">
                        <div class="grid gap-4 sm:grid-cols-2">
                            <div class="space-y-1.5">
                                <Label for="alert-webhook">Webhook</Label>
                                <Select
                                    :model-value="alertForm.map_webhook_id?.toString()"
                                    @update:model-value="(v) => (alertForm.map_webhook_id = Number(v))"
                                >
                                    <SelectTrigger id="alert-webhook" class="w-full"><SelectValue placeholder="Select a webhook" /></SelectTrigger>
                                    <SelectContent>
                                        <SelectItem v-for="webhook in webhooks" :key="webhook.id" :value="webhook.id.toString()">{{
                                            webhook.name
                                        }}</SelectItem>
                                    </SelectContent>
                                </Select>
                            </div>
                            <div class="space-y-1.5">
                                <Label for="alert-role">Ping</Label>
                                <Select :model-value="alertPingValue" @update:model-value="setAlertPing">
                                    <SelectTrigger id="alert-role" class="w-full"><SelectValue /></SelectTrigger>
                                    <SelectContent>
                                        <SelectItem :value="ROLE_NONE">No ping</SelectItem>
                                        <SelectItem :value="PING_EVERYONE">@everyone</SelectItem>
                                        <SelectItem v-for="role in roles" :key="role.id" :value="role.id.toString()">@{{ role.name }}</SelectItem>
                                    </SelectContent>
                                </Select>
                            </div>
                        </div>
                        <div class="flex items-center gap-2">
                            <Checkbox
                                id="alert-active"
                                :model-value="alertForm.is_active"
                                @update:model-value="(v) => (alertForm.is_active = v === true)"
                            />
                            <Label for="alert-active" class="text-sm font-medium">Active</Label>
                        </div>
                    </TabsContent>
                </Tabs>

                <div v-if="alertErrors.length" class="shrink-0 space-y-1 px-6 pb-3" role="alert">
                    <p v-for="error in alertErrors" :key="error" class="text-sm text-destructive">{{ error }}</p>
                </div>

                <DialogFooter class="shrink-0 border-t px-6 py-4">
                    <Button variant="ghost" @click="alertDialogOpen = false">Cancel</Button>
                    <Button :disabled="!canSubmitAlert || alertForm.processing" @click="submitAlert">
                        {{ alertForm.processing ? 'Saving…' : editingAlert ? 'Save' : 'Create' }}
                    </Button>
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
