<script lang="ts">
	// Discord alerts for one map: standing questions about the chain that answer themselves
	// into a channel.
	import BellIcon from '@lucide/svelte/icons/bell';
	import PlusIcon from '@lucide/svelte/icons/plus';
	import TrashIcon from '@lucide/svelte/icons/trash-2';
	import { lookup } from '$lib/lookup';

	import { createQuery } from '@tanstack/svelte-query';
	import { api, errorMessage } from '$lib/api/client';
	import { confirmDanger } from '$lib/confirm.svelte';
	import { after, apiAction } from '$lib/api/mutations';
	import {
		DELIVERY_LABEL,
		DISABLED_REASON,
		kindLabel,
		mentionSummary,
		alertSummary,
	} from '$lib/alerts/vocabulary';
	import { key, q } from '$lib/api/queries';
	import type { MapAlert } from '$lib/api/types/MapAlert';
	import type { SaveAlert } from '$lib/api/types/SaveAlert';
	import { Badge } from '$lib/components/ui/badge';
	import { Button } from '$lib/components/ui/button';
	import * as Card from '$lib/components/ui/card';
	import DiscordInviteCard from '$lib/components/DiscordInviteCard.svelte';
	import InstanceNotice from '$lib/components/InstanceNotice.svelte';
	import { Switch } from '$lib/components/ui/switch';
	import { timeAgo } from '$lib/format';
	import { cn } from '$lib/utils';
	import AlertForm from './AlertForm.svelte';
	import DestinationsCard from './DestinationsCard.svelte';
	import { mapSettings } from '../context.svelte';

	const ctx = mapSettings();
	const mapId = $derived(ctx.mapId);

	let editing = $state<MapAlert | null>(null);
	let creating = $state(false);

	// A deployment without a Discord bot can still post to a webhook, so this only dims the
	// half that needs one rather than the whole page.
	const instanceQuery = createQuery(() => q.instance());
	const alertsQuery = createQuery(() => q.listAlerts(mapId));
	const eventsQuery = createQuery(() => q.alertEvents(mapId));
	const webhooksQuery = createQuery(() => q.listWebhooks(mapId));
	const rolesQuery = createQuery(() => q.listAlertRoles(mapId));

	const instance = $derived(instanceQuery.data ?? null);
	const alerts = $derived(alertsQuery.data ?? []);
	const events = $derived(eventsQuery.data ?? []);
	const webhooks = $derived(webhooksQuery.data ?? []);
	const roles = $derived(rolesQuery.data ?? []);
	// The server is the authority on who may manage alerts, so nothing renders until the list
	// comes back: otherwise the controls would offer buttons that 403.
	const canManage = $derived(alertsQuery.isSuccess);
	const error = $derived(alertsQuery.error ? errorMessage(alertsQuery.error) : null);

	const act = apiAction(() => [key.alerting(mapId)]);

	function save(body: SaveAlert) {
		const id = editing?.id;
		after(
			act.mutateAsync(() => (id ? api.updateAlert(mapId, id, body) : api.createAlert(mapId, body))),
			() => {
				editing = null;
				creating = false;
			},
		);
	}

	async function remove(alert: MapAlert) {
		if (!(await confirmDanger({ title: `Delete "${alert.name}"?` }))) return;
		act.mutate(() => api.deleteAlert(mapId, alert.id));
	}
</script>

<div class="flex flex-col gap-6">
	{#if instance && !instance.discord.bot}
		<InstanceNotice title="The Discord bot is not set up on this instance">
			Alerts to a <strong>channel webhook</strong> work as normal. Direct messages and posting as the
			bot need a bot token, which whoever runs this instance has not configured.
		</InstanceNotice>
	{/if}
	{#if error}
		<p class="text-sm text-destructive" data-testid="alerts-error">{error}</p>
	{/if}

	<Card.Root>
		<Card.Header>
			<div class="flex items-start justify-between gap-3">
				<div class="flex flex-col gap-1.5">
					<Card.Title class="flex items-center gap-2">
						<BellIcon class="size-4" />
						Discord alerts
					</Card.Title>
					<Card.Description>
						Standing questions about the chain, answered into a Discord channel.
					</Card.Description>
				</div>
				{#if canManage && !creating && !editing}
					<Button
						size="sm"
						onclick={() => {
							creating = true;
							editing = null;
						}}
						data-testid="alert-new"
					>
						<PlusIcon data-icon="inline-start" />
						New alert
					</Button>
				{/if}
			</div>
		</Card.Header>
		<Card.Content class="flex flex-col gap-3">
			{#if creating || editing}
				<!-- Keyed so switching alerts reseeds the form instead of keeping the old values. -->
				{#key editing?.id ?? 'new'}
					<AlertForm
						alert={editing}
						{webhooks}
						{roles}
						onsave={save}
						oncancel={() => {
							creating = false;
							editing = null;
						}}
					/>
				{/key}
			{/if}

			{#if canManage && alerts.length === 0 && !creating}
				<p class="py-6 text-center text-sm text-muted-foreground" data-testid="alerts-empty">
					No alerts yet. A channel webhook is the quickest way to start: Discord's channel settings,
					Integrations, New Webhook.
				</p>
			{/if}

			{#each alerts as alert (alert.id)}
				<div
					class={cn(
						'flex flex-col gap-2 border border-border/60 p-3',
						!alert.is_active && 'opacity-60',
					)}
					data-testid="alert-row"
					data-alert={alert.id}
				>
					<div class="flex items-start justify-between gap-3">
						<div class="flex min-w-0 flex-col gap-1">
							<span class="flex items-center gap-2">
								<span class="truncate text-sm font-medium">{alert.name}</span>
								<Badge variant="outline" class="shrink-0">{kindLabel(alert.kind)}</Badge>
							</span>
							<span class="text-xs text-muted-foreground">{alertSummary(alert)}</span>
							<span
								class="flex flex-wrap items-center gap-x-2 text-[11px] text-muted-foreground/70"
							>
								<span>{DELIVERY_LABEL[alert.delivery]}</span>
								{#if alert.webhook_name}
									<span>→ {alert.webhook_name}</span>
								{/if}
								{#if alert.mention === 'role' && alert.role_name}
									<span>@{alert.role_name}</span>
								{/if}
								<span>·</span>
								<span>{mentionSummary(alert.mention)}</span>
								{#if alert.last_fired_at}
									<span>·</span>
									<span data-testid="alert-fired">fired {timeAgo(alert.last_fired_at)}</span>
								{/if}
							</span>
							{#if !alert.is_active && alert.disabled_reason}
								<span class="text-[11px] text-amber-500" data-testid="alert-reason">
									{lookup(DISABLED_REASON, alert.disabled_reason) ?? alert.disabled_reason}
								</span>
							{/if}
						</div>
						<div class="flex shrink-0 items-center gap-2">
							<Switch
								checked={alert.is_active}
								aria-label="Enable {alert.name}"
								onCheckedChange={(value) =>
									act.mutate(() => api.setAlertActive(mapId, alert.id, value))}
							/>
							<Button
								variant="ghost"
								size="sm"
								onclick={() => {
									editing = alert;
									creating = false;
								}}
								data-testid="alert-edit">Edit</Button
							>
							<Button
								variant="ghost"
								size="icon"
								class="size-8"
								aria-label="Delete {alert.name}"
								onclick={() => remove(alert)}
							>
								<TrashIcon />
							</Button>
						</div>
					</div>
				</div>
			{/each}
		</Card.Content>
	</Card.Root>

	{#if canManage && instance?.discord.invite_url}
		<DiscordInviteCard inviteUrl={instance.discord.invite_url} />
	{/if}

	{#if canManage}
		<DestinationsCard {mapId} {webhooks} {roles} />
	{/if}

	<Card.Root>
		<Card.Header>
			<Card.Title>History</Card.Title>
			<Card.Description>Who changed what, and every time an alert fired or failed.</Card.Description
			>
		</Card.Header>
		<Card.Content>
			{#if events.length === 0}
				<p class="py-4 text-sm text-muted-foreground">Nothing yet.</p>
			{:else}
				<ul class="flex flex-col text-xs" data-testid="alert-events">
					{#each events as event (event.id)}
						<li class="flex items-center gap-2 border-b border-border/30 py-1.5 last:border-b-0">
							<span class="w-16 shrink-0 font-mono text-[10px] text-muted-foreground uppercase">
								{event.kind}
							</span>
							<span class="min-w-0 flex-1 truncate">
								{event.alert_name ?? event.detail ?? '—'}
								{#if event.detail && event.alert_name}
									<span class="text-muted-foreground">: {event.detail}</span>
								{/if}
							</span>
							<span class="shrink-0 text-muted-foreground">{event.actor ?? 'system'}</span>
							<span class="w-16 shrink-0 text-right text-muted-foreground/70">
								{timeAgo(event.created_at)}
							</span>
						</li>
					{/each}
				</ul>
			{/if}
		</Card.Content>
	</Card.Root>
</div>
