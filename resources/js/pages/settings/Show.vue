<script setup lang="ts">
import DiscordAccountController from '@/actions/App/Http/Controllers/DiscordAccountController';
import PreferredCharacterController from '@/actions/App/Http/Controllers/PreferredCharacterController';
import ScopeController from '@/actions/App/Http/Controllers/ScopeController';
import SettingsController from '@/actions/App/Http/Controllers/SettingsController';
import TokenManagementController from '@/actions/App/Http/Controllers/TokenManagementController';
import UserCharacterController from '@/actions/App/Http/Controllers/UserCharacterController';
import DiscordIcon from '@/components/icons/DiscordIcon.vue';
import CharacterImage from '@/components/images/CharacterImage.vue';
import { Badge } from '@/components/ui/badge';
import { Button } from '@/components/ui/button';
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card';
import { Dialog, DialogClose, DialogContent, DialogDescription, DialogFooter, DialogHeader, DialogTitle } from '@/components/ui/dialog';
import { DropdownMenu, DropdownMenuContent, DropdownMenuItem, DropdownMenuSeparator, DropdownMenuTrigger } from '@/components/ui/dropdown-menu';
import { Input } from '@/components/ui/input';
import { Label } from '@/components/ui/label';
import { RadioGroup, RadioGroupItem } from '@/components/ui/radio-group';
import { Table, TableBody, TableCell, TableHead, TableHeader, TableRow } from '@/components/ui/table';
import { Tabs, TabsContent, TabsList, TabsTrigger } from '@/components/ui/tabs';
import { Tooltip, TooltipContent, TooltipTrigger } from '@/components/ui/tooltip';
import AppLayout from '@/layouts/AppLayout.vue';
import SeoHead from '@/layouts/SeoHead.vue';
import { alertTriggerLabel } from '@/lib/mapAlerts';
import type { TCharacter, TDiscordAccount, TMapAlert, TToken } from '@/types/models';
import { UTCDate } from '@date-fns/utc';
import { router, useForm } from '@inertiajs/vue3';
import { format } from 'date-fns';
import { Check, Copy, Ellipsis, ExternalLink, HelpCircle, KeyRound, Plus, Radio, ShieldCheck, ShieldOff, Trash2, UserMinus } from 'lucide-vue-next';
import { computed, ref } from 'vue';
import { toast } from 'vue-sonner';

type SettingsCharacter = TCharacter & { is_preferred: boolean };

const props = defineProps<{
    section: SettingsSection;
    characters: SettingsCharacter[];
    discordAccount: TDiscordAccount | null;
    discordAlerts: TMapAlert[];
    pendingDiscordAccount: { id: string; username: string; global_name: string | null } | null;
    discordConfigured: boolean;
    discordInviteUrl: string | null;
    tokens: TToken[];
    token: string | null;
}>();
const managedScopes = [
    { scope: 'esi-location.read_online.v1', name: 'Online' },
    { scope: 'esi-location.read_location.v1', name: 'Location' },
    { scope: 'esi-location.read_ship_type.v1', name: 'Ship' },
    { scope: 'esi-ui.write_waypoint.v1', name: 'Waypoints' },
];
const allScopeNames = managedScopes.map(({ scope }) => scope).join(',');
const preferredCharacterId = computed(() => props.characters.find((character) => character.is_preferred)?.id.toString());
type SettingsSection = 'esi' | 'discord' | 'tokens';
const tokenForm = useForm({ name: '' });
const disconnectForm = useForm({});
const deleteTokenForm = useForm({});
const characterPendingRemoval = ref<SettingsCharacter | null>(null);
const scopePendingRemoval = ref<SettingsCharacter | null>(null);
const disconnectPending = ref(false);
const tokenPendingDeletion = ref<TToken | null>(null);
const grantUrl = (scopes: string): string => ScopeController.show({ query: { scopes } }).url;
const grantedScopes = (character: SettingsCharacter): number => managedScopes.filter(({ scope }) => character.esi_scopes?.includes(scope)).length;

function changeSection(value: string | number): void {
    const section: SettingsSection = value === 'discord' || value === 'tokens' ? value : 'esi';
    if (section === props.section) return;

    router.visit(SettingsController.show({ query: { section } }), { preserveScroll: true });
}

function setPreferredById(characterId: unknown): void {
    const character = props.characters.find(({ id }) => id.toString() === String(characterId));
    if (character && !character.is_preferred) router.visit(PreferredCharacterController.store(character.id));
}

function removeCharacter(): void {
    if (!characterPendingRemoval.value) return;

    router.visit(UserCharacterController.delete(characterPendingRemoval.value.id), {
        onFinish: () => (characterPendingRemoval.value = null),
    });
}

function removeScopes(): void {
    if (!scopePendingRemoval.value) return;

    router.visit(ScopeController.destroy(scopePendingRemoval.value.id), {
        preserveScroll: true,
        onFinish: () => (scopePendingRemoval.value = null),
    });
}

function createToken(): void {
    tokenForm.submit(TokenManagementController.store(), { onSuccess: () => tokenForm.reset() });
}

function disconnectDiscord(): void {
    disconnectForm.submit(DiscordAccountController.destroy(), {
        preserveScroll: true,
        onSuccess: () => (disconnectPending.value = false),
    });
}

function deleteToken(): void {
    if (!tokenPendingDeletion.value) return;

    deleteTokenForm.submit(TokenManagementController.destroy(tokenPendingDeletion.value.id), {
        preserveScroll: true,
        onSuccess: () => (tokenPendingDeletion.value = null),
    });
}

function copyToken(): void {
    if (!props.token) return;
    navigator.clipboard
        .writeText(props.token)
        .then(() => toast.success('Token copied'))
        .catch(() => toast.error('Could not copy token'));
}

const formatDate = (date: string | null): string => (date ? format(new UTCDate(date), 'PP p') : 'Never');
</script>

<template>
    <AppLayout>
        <SeoHead title="Account settings" description="Manage EVE permissions, Discord alerts, and API access." />
        <main class="mx-auto flex w-full max-w-6xl flex-col gap-6 px-4 py-6 sm:px-6 sm:py-10">
            <header class="flex flex-col gap-2">
                <div class="flex items-center gap-2 text-xs font-semibold tracking-[0.2em] text-muted-foreground uppercase">
                    <Radio class="size-3.5 text-primary" /> Account control
                </div>
                <h1 class="text-3xl font-bold tracking-tight">Settings</h1>
                <p class="max-w-2xl text-sm text-muted-foreground">Your characters, connected services, and machine access in one place.</p>
            </header>
            <Tabs :model-value="section" class="space-y-6" @update:model-value="changeSection">
                <TabsList class="grid h-auto w-full grid-cols-3 p-1">
                    <TabsTrigger value="esi" data-testid="settings-tab-esi" class="gap-1 px-2 py-2.5 sm:gap-2 sm:px-3"
                        >Characters <Badge variant="outline" class="hidden sm:inline-flex">{{ characters.length }}</Badge></TabsTrigger
                    >
                    <TabsTrigger value="discord" data-testid="settings-tab-discord" class="gap-1 px-2 py-2.5 sm:gap-2 sm:px-3"
                        >Discord <Badge variant="outline" class="hidden sm:inline-flex">{{ discordAlerts.length }}</Badge></TabsTrigger
                    >
                    <TabsTrigger value="tokens" data-testid="settings-tab-tokens" class="gap-1 px-2 py-2.5 sm:gap-2 sm:px-3"
                        >API tokens <Badge variant="outline" class="hidden sm:inline-flex">{{ tokens.length }}</Badge></TabsTrigger
                    >
                </TabsList>

                <TabsContent value="esi" class="mt-0">
                    <Card id="esi" class="gap-0 overflow-hidden py-0">
                        <CardHeader class="border-b bg-muted/20 py-4 sm:flex-row sm:items-center sm:justify-between"
                            ><div class="space-y-1">
                                <CardTitle>Characters and ESI</CardTitle
                                ><CardDescription>Grant only the live data each character should share.</CardDescription>
                            </div>
                            <Button size="sm" as-child><a :href="grantUrl(allScopeNames)">Grant all permissions</a></Button></CardHeader
                        >
                        <CardContent class="p-0">
                            <RadioGroup :model-value="preferredCharacterId" @update:model-value="setPreferredById">
                                <Table>
                                    <TableHeader>
                                        <TableRow>
                                            <TableHead class="min-w-56">Character</TableHead>
                                            <TableHead class="min-w-44">Scope status</TableHead>
                                            <TableHead class="w-28 text-center">
                                                <Tooltip :delay-duration="300">
                                                    <TooltipTrigger class="inline-flex items-center gap-1 align-middle">
                                                        Preferred
                                                        <HelpCircle class="size-3.5 text-muted-foreground" />
                                                    </TooltipTrigger>
                                                    <TooltipContent class="max-w-64">
                                                        Your preferred character is used as your default character when you sign in and as the display
                                                        name on alerts you create.
                                                    </TooltipContent>
                                                </Tooltip>
                                            </TableHead>
                                            <TableHead class="w-16"><span class="sr-only">Actions</span></TableHead>
                                        </TableRow>
                                    </TableHeader>
                                    <TableBody>
                                        <TableRow v-for="character in characters" :key="character.id">
                                            <TableCell>
                                                <div class="flex items-center gap-3">
                                                    <CharacterImage
                                                        :character_id="character.id"
                                                        :character_name="character.name"
                                                        class="size-10 shrink-0 rounded-full"
                                                    />
                                                    <span class="font-medium whitespace-nowrap">{{ character.name }}</span>
                                                </div>
                                            </TableCell>
                                            <TableCell>
                                                <div class="flex items-center gap-2">
                                                    <DropdownMenu>
                                                        <DropdownMenuTrigger as-child>
                                                            <button
                                                                type="button"
                                                                class="rounded-md focus-visible:ring-2 focus-visible:ring-ring focus-visible:outline-none"
                                                            >
                                                                <Badge
                                                                    :variant="
                                                                        grantedScopes(character) === managedScopes.length ? 'secondary' : 'outline'
                                                                    "
                                                                    class="cursor-pointer whitespace-nowrap"
                                                                >
                                                                    {{ grantedScopes(character) }}/{{ managedScopes.length }} granted
                                                                </Badge>
                                                            </button>
                                                        </DropdownMenuTrigger>
                                                        <DropdownMenuContent align="start" class="w-60">
                                                            <template v-for="scope in managedScopes" :key="scope.scope">
                                                                <DropdownMenuItem
                                                                    v-if="character.esi_scopes?.includes(scope.scope)"
                                                                    disabled
                                                                    class="w-full"
                                                                >
                                                                    <Check class="size-4 text-emerald-500" />
                                                                    <span>{{ scope.name }}</span>
                                                                    <span class="ml-auto text-xs text-muted-foreground">Granted</span>
                                                                </DropdownMenuItem>
                                                                <DropdownMenuItem v-else as-child>
                                                                    <a :href="grantUrl(scope.scope)" class="flex w-full items-center gap-2">
                                                                        <Plus class="size-4" />
                                                                        Grant {{ scope.name }}
                                                                    </a>
                                                                </DropdownMenuItem>
                                                            </template>
                                                            <DropdownMenuSeparator />
                                                            <DropdownMenuItem as-child>
                                                                <a :href="grantUrl(allScopeNames)" class="flex w-full items-center gap-2">
                                                                    <ShieldCheck class="size-4" />
                                                                    Grant all permissions
                                                                </a>
                                                            </DropdownMenuItem>
                                                        </DropdownMenuContent>
                                                    </DropdownMenu>
                                                    <div class="flex gap-1" aria-hidden="true">
                                                        <span
                                                            v-for="scope in managedScopes"
                                                            :key="scope.scope"
                                                            class="size-1.5 rounded-full"
                                                            :class="
                                                                character.esi_scopes?.includes(scope.scope)
                                                                    ? 'bg-emerald-500'
                                                                    : 'bg-muted-foreground/30'
                                                            "
                                                        />
                                                    </div>
                                                    <span class="sr-only">
                                                        {{
                                                            managedScopes
                                                                .filter(({ scope }) => character.esi_scopes?.includes(scope))
                                                                .map(({ name }) => name)
                                                                .join(', ') || 'No scopes granted'
                                                        }}
                                                    </span>
                                                </div>
                                            </TableCell>
                                            <TableCell class="text-center">
                                                <div class="inline-flex rounded-full bg-muted/60 p-1.5">
                                                    <RadioGroupItem
                                                        :value="character.id.toString()"
                                                        :aria-label="`Use ${character.name} as preferred character`"
                                                        class="size-5 border-muted-foreground/40 bg-background data-[state=checked]:border-primary data-[state=checked]:bg-primary/10"
                                                    />
                                                </div>
                                            </TableCell>
                                            <TableCell class="text-right">
                                                <DropdownMenu>
                                                    <DropdownMenuTrigger as-child>
                                                        <Button variant="ghost" size="icon" :aria-label="`Actions for ${character.name}`">
                                                            <Ellipsis class="size-4" />
                                                        </Button>
                                                    </DropdownMenuTrigger>
                                                    <DropdownMenuContent align="end" class="w-56">
                                                        <DropdownMenuItem as-child>
                                                            <a :href="grantUrl(allScopeNames)" class="flex w-full items-center gap-2">
                                                                <ShieldCheck class="size-4" />
                                                                Grant all permissions
                                                            </a>
                                                        </DropdownMenuItem>
                                                        <DropdownMenuItem class="w-full" @select="scopePendingRemoval = character">
                                                            <ShieldOff class="size-4" />
                                                            Remove all permissions
                                                        </DropdownMenuItem>
                                                        <DropdownMenuSeparator />
                                                        <DropdownMenuItem
                                                            variant="destructive"
                                                            :disabled="characters.length === 1"
                                                            class="w-full"
                                                            @select="characterPendingRemoval = character"
                                                        >
                                                            <UserMinus class="size-4" />
                                                            Remove character
                                                        </DropdownMenuItem>
                                                    </DropdownMenuContent>
                                                </DropdownMenu>
                                            </TableCell>
                                        </TableRow>
                                    </TableBody>
                                </Table>
                            </RadioGroup>
                        </CardContent>
                    </Card>
                </TabsContent>

                <Dialog :open="characterPendingRemoval !== null" @update:open="(open) => !open && (characterPendingRemoval = null)">
                    <DialogContent>
                        <DialogHeader>
                            <DialogTitle>Remove {{ characterPendingRemoval?.name }}?</DialogTitle>
                            <DialogDescription>
                                This character will be removed from your Wormhole Systems account. You cannot remove your last character.
                            </DialogDescription>
                        </DialogHeader>
                        <DialogFooter>
                            <DialogClose as-child><Button variant="outline">Cancel</Button></DialogClose>
                            <Button variant="destructive" @click="removeCharacter">Remove character</Button>
                        </DialogFooter>
                    </DialogContent>
                </Dialog>

                <Dialog :open="scopePendingRemoval !== null" @update:open="(open) => !open && (scopePendingRemoval = null)">
                    <DialogContent>
                        <DialogHeader>
                            <DialogTitle>Remove all permissions for {{ scopePendingRemoval?.name }}?</DialogTitle>
                            <DialogDescription>
                                This removes every ESI permission from this character. Features that require live EVE data will stop working.
                            </DialogDescription>
                        </DialogHeader>
                        <DialogFooter>
                            <DialogClose as-child><Button variant="outline">Cancel</Button></DialogClose>
                            <Button variant="destructive" @click="removeScopes">Remove all permissions</Button>
                        </DialogFooter>
                    </DialogContent>
                </Dialog>

                <TabsContent value="discord" class="mt-0">
                    <Card id="discord" class="gap-0 overflow-hidden rounded-xl border-indigo-400/25 py-0">
                        <div class="relative overflow-hidden bg-indigo-600 px-6 py-7 text-white shadow-lg shadow-indigo-950/15 sm:px-8">
                            <div class="absolute -top-16 -right-12 size-48 rounded-full bg-white/10 blur-2xl"></div>
                            <div class="relative flex flex-col gap-6 sm:flex-row sm:items-center sm:justify-between">
                                <div class="flex min-w-0 items-start gap-4">
                                    <div class="flex size-12 shrink-0 items-center justify-center rounded-xl bg-white/15 ring-1 ring-white/20">
                                        <DiscordIcon class="size-7" />
                                    </div>
                                    <div class="min-w-0 space-y-1">
                                        <p class="text-xs font-semibold tracking-[0.18em] text-indigo-100 uppercase">Discord account</p>
                                        <h2 class="truncate text-2xl font-semibold tracking-tight">
                                            {{ discordAccount ? discordAccount.display_name || discordAccount.username : 'Not connected' }}
                                        </h2>
                                        <p class="truncate text-sm leading-6 text-indigo-100">
                                            <template v-if="discordAccount">Connected as @{{ discordAccount.username }}</template>
                                            <template v-else>Link your Discord account to receive personal alerts.</template>
                                        </p>
                                    </div>
                                </div>
                                <div class="flex shrink-0 flex-wrap items-center gap-2">
                                    <Button
                                        v-if="discordInviteUrl"
                                        as-child
                                        class="border border-white/30 bg-white/10 text-white hover:bg-white/20 hover:text-white"
                                    >
                                        <a :href="discordInviteUrl" target="_blank" rel="noopener noreferrer">
                                            Invite bot
                                            <ExternalLink class="size-4" />
                                        </a>
                                    </Button>
                                    <Button v-if="discordConfigured" as-child class="bg-white text-black shadow-sm hover:bg-white/90">
                                        <a :href="DiscordAccountController.redirect().url">{{ discordAccount ? 'Switch account' : 'Connect' }}</a>
                                    </Button>
                                </div>
                            </div>
                        </div>
                        <CardContent class="flex flex-col gap-4 px-4 py-4">
                            <div
                                v-if="pendingDiscordAccount"
                                class="flex items-center justify-between gap-3 rounded-lg border border-indigo-500/30 p-3"
                            >
                                <div class="min-w-0 text-sm">
                                    Connect as <strong>@{{ pendingDiscordAccount.username }}</strong>
                                </div>
                                <Button
                                    size="sm"
                                    class="shrink-0 bg-[#5865f2] text-white hover:bg-[#4752c4]"
                                    @click="router.visit(DiscordAccountController.confirmLink())"
                                >
                                    Confirm
                                </Button>
                            </div>
                            <p v-if="!discordConfigured" class="text-sm text-amber-600">Discord linking is not configured.</p>
                            <div class="flex items-center justify-between gap-3">
                                <div class="text-xs font-semibold tracking-wider text-muted-foreground uppercase">Your alerts</div>
                                <Badge variant="outline">{{ discordAlerts.length }}</Badge>
                            </div>
                            <div v-if="!discordAlerts.length" class="rounded-lg border border-dashed p-3 text-sm text-muted-foreground">
                                Use <code>/alerts add</code> in Discord to create an alert.
                            </div>
                            <div v-else class="divide-y rounded-lg border">
                                <div v-for="alert in discordAlerts" :key="alert.id" class="flex items-center justify-between gap-3 px-3 py-2 text-sm">
                                    <div class="min-w-0">
                                        <div class="truncate font-medium">{{ alertTriggerLabel(alert) }}</div>
                                        <div class="truncate text-xs text-muted-foreground">
                                            {{ alert.map?.name ?? 'Unknown map' }} · {{ alert.destination_summary }}
                                        </div>
                                    </div>
                                    <Badge :variant="alert.is_active ? 'secondary' : 'outline'">{{ alert.is_active ? 'Active' : 'Disabled' }}</Badge>
                                </div>
                            </div>
                            <div v-if="discordAccount" class="flex justify-end border-t pt-3">
                                <Button variant="ghost" size="sm" class="text-destructive hover:text-destructive" @click="disconnectPending = true">
                                    Disconnect Discord
                                </Button>
                            </div>
                        </CardContent>
                    </Card>
                </TabsContent>

                <TabsContent value="tokens" class="mt-0">
                    <Card id="tokens" class="gap-0 overflow-hidden py-0">
                        <CardHeader class="border-b bg-muted/20 py-4">
                            <CardTitle class="flex items-center gap-2"><KeyRound class="size-5" />API tokens</CardTitle>
                            <CardDescription>Credentials inherit your full map access. Treat them like passwords.</CardDescription>
                        </CardHeader>
                        <CardContent class="flex flex-col gap-4 px-4 py-4">
                            <form class="grid grid-cols-[1fr_auto] gap-2" @submit.prevent="createToken">
                                <Label for="token-name" class="col-span-full">New token</Label>
                                <Input id="token-name" v-model="tokenForm.name" placeholder="Integration name" required />
                                <Button type="submit" :disabled="tokenForm.processing">Create</Button>
                                <p v-if="tokenForm.errors.name" class="col-span-full text-xs text-destructive">{{ tokenForm.errors.name }}</p>
                            </form>
                            <div v-if="token" class="grid grid-cols-[1fr_auto] gap-2 rounded-lg border border-amber-500/30 bg-amber-500/5 p-3">
                                <code class="min-w-0 overflow-x-auto rounded bg-background p-2 text-xs">{{ token }}</code>
                                <Button size="icon" variant="outline" aria-label="Copy API token" @click="copyToken"><Copy class="size-4" /></Button>
                                <p class="col-span-full text-xs text-muted-foreground">Copy this now. It will not be shown again.</p>
                            </div>
                            <div class="flex items-center justify-between gap-3">
                                <div class="text-xs font-semibold tracking-wider text-muted-foreground uppercase">Your tokens</div>
                                <Badge variant="outline">{{ tokens.length }}</Badge>
                            </div>
                            <div v-if="!tokens.length" class="rounded-lg border border-dashed p-3 text-sm text-muted-foreground">No API tokens.</div>
                            <div v-else class="divide-y rounded-lg border">
                                <div v-for="apiToken in tokens" :key="apiToken.id" class="flex items-center justify-between gap-3 px-3 py-2 text-sm">
                                    <div class="min-w-0">
                                        <div class="truncate font-medium">{{ apiToken.name }}</div>
                                        <div class="text-xs text-muted-foreground">Last used {{ formatDate(apiToken.last_used_at) }}</div>
                                    </div>
                                    <Button
                                        size="icon"
                                        variant="ghost"
                                        class="text-destructive"
                                        :aria-label="`Delete ${apiToken.name} token`"
                                        @click="tokenPendingDeletion = apiToken"
                                        ><Trash2 class="size-4"
                                    /></Button>
                                </div>
                            </div>
                        </CardContent>
                    </Card>
                </TabsContent>
            </Tabs>

            <Dialog :open="disconnectPending" @update:open="(open) => !open && (disconnectPending = false)">
                <DialogContent>
                    <DialogHeader>
                        <DialogTitle>Disconnect Discord?</DialogTitle>
                        <DialogDescription>
                            Disconnecting your account disables alerts that depend on your Discord identity. You can reconnect later, but disabled
                            alerts will remain disabled until re-enabled.
                        </DialogDescription>
                    </DialogHeader>
                    <DialogFooter>
                        <DialogClose as-child><Button variant="outline">Cancel</Button></DialogClose>
                        <Button variant="destructive" :disabled="disconnectForm.processing" @click="disconnectDiscord">
                            {{ disconnectForm.processing ? 'Disconnecting…' : 'Disconnect' }}
                        </Button>
                    </DialogFooter>
                </DialogContent>
            </Dialog>

            <Dialog :open="tokenPendingDeletion !== null" @update:open="(open) => !open && (tokenPendingDeletion = null)">
                <DialogContent>
                    <DialogHeader>
                        <DialogTitle>Delete {{ tokenPendingDeletion?.name }}?</DialogTitle>
                        <DialogDescription>Applications using this token will immediately lose access. This cannot be undone.</DialogDescription>
                    </DialogHeader>
                    <DialogFooter>
                        <DialogClose as-child><Button variant="outline">Cancel</Button></DialogClose>
                        <Button variant="destructive" :disabled="deleteTokenForm.processing" @click="deleteToken">
                            {{ deleteTokenForm.processing ? 'Deleting…' : 'Delete token' }}
                        </Button>
                    </DialogFooter>
                </DialogContent>
            </Dialog>
        </main>
    </AppLayout>
</template>
