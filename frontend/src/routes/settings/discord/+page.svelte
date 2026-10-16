<script lang="ts">
	// Linking this WormholeSystems account to a Discord one. Per account, not per map: a direct message
	// and a slash command both need to know which Discord user this is.
	import CheckCircleIcon from '@lucide/svelte/icons/check-circle-2';
	import ExternalLinkIcon from '@lucide/svelte/icons/external-link';

	import { createQuery } from '@tanstack/svelte-query';

	import { page } from '$app/state';
	import { api, errorMessage } from '$lib/api/client';
	import { confirmDanger } from '$lib/confirm.svelte';
	import { apiAction } from '$lib/api/mutations';
	import { Button } from '$lib/components/ui/button';
	import * as Card from '$lib/components/ui/card';
	import DiscordInviteCard from '$lib/components/DiscordInviteCard.svelte';
	import InstanceNotice from '$lib/components/InstanceNotice.svelte';
	import { q } from '$lib/api/queries';

	// What this deployment configured, not what the account did: a self-hosted instance may
	// have no Discord application at all.
	const instanceQuery = createQuery(() => q.instance());
	const accountQuery = createQuery(() => q.myDiscord());
	const instance = $derived(instanceQuery.data ?? null);
	const account = $derived(accountQuery.data ?? null);
	const loaded = $derived(!accountQuery.isPending);
	const error = $derived(accountQuery.error ? errorMessage(accountQuery.error) : null);

	const justLinked = $derived(page.url.searchParams.get('linked') === '1');

	const act = apiAction(() => [q.myDiscord().queryKey]);

	async function unlink() {
		const sure = await confirmDanger({
			title: 'Unlink Discord?',
			body: 'Alerts that direct-message you will stop.',
			action: 'Unlink',
		});
		if (!sure) return;
		act.mutate(() => api.unlinkDiscord());
	}
</script>

<div class="flex flex-col gap-6">
	{#if loaded && instance && !instance.discord.linking}
		<InstanceNotice title="Discord is not set up on this instance">
			Whoever runs it has not configured a Discord application, so there is nothing to link an
			account to. Alerts can still be delivered to a channel webhook, which needs no application.
		</InstanceNotice>
	{:else if loaded && instance && !instance.discord.bot}
		<InstanceNotice title="The Discord bot is not set up on this instance">
			Linking works, but there is no bot token, so nothing can direct-message you, post to a channel
			as the bot, or answer <code>/wh</code>. Channel webhooks are unaffected.
		</InstanceNotice>
	{/if}
	{#if error}
		<p class="text-sm text-destructive" data-testid="discord-error">{error}</p>
	{/if}

	<Card.Root>
		<Card.Header>
			<Card.Title>Your Discord account</Card.Title>
			<Card.Description>
				Linking lets the bot know which maps are yours, so <code>/wh</code> can answer about them and
				alerts can reach you directly.
			</Card.Description>
		</Card.Header>
		<Card.Content class="flex flex-col gap-4">
			{#if !loaded}
				<p class="text-sm text-muted-foreground">Checking…</p>
			{:else if account}
				<div class="flex items-center justify-between gap-3" data-testid="discord-linked">
					<span class="flex items-center gap-2 text-sm">
						<CheckCircleIcon class="size-4 text-emerald-500" />
						Linked to
						<span class="font-medium">{account.display_name ?? account.username}</span>
						<span class="font-mono text-xs text-muted-foreground">@{account.username}</span>
					</span>
					<Button variant="outline" size="sm" onclick={unlink} data-testid="discord-unlink">
						Unlink
					</Button>
				</div>
				{#if justLinked}
					<p class="text-xs text-emerald-500">Linked. Try <code>/wh account</code> in Discord.</p>
				{/if}
			{:else}
				<p class="text-sm text-muted-foreground" data-testid="discord-unlinked">
					No Discord account linked yet.
				</p>
				<Button href="/discord/connect" class="w-fit" data-testid="discord-connect">
					<ExternalLinkIcon data-icon="inline-start" />
					Connect Discord
				</Button>
			{/if}
		</Card.Content>
	</Card.Root>

	{#if instance?.discord.invite_url}
		<DiscordInviteCard inviteUrl={instance.discord.invite_url} />
	{/if}

	<Card.Root>
		<Card.Header>
			<Card.Title>Commands</Card.Title>
			<Card.Description>What the bot answers, wherever you ask it.</Card.Description>
		</Card.Header>
		<Card.Content>
			<ul class="flex flex-col gap-2 text-sm">
				<li>
					<code class="text-foreground">/wh account</code>
					<span class="text-muted-foreground"
						>: which WormholeSystems account this Discord user is.</span
					>
				</li>
				<li>
					<code class="text-foreground">/wh alerts list</code>
					<span class="text-muted-foreground"
						>: the alerts you created, and whether they are on.</span
					>
				</li>
				<li>
					<code class="text-foreground">/wh alerts enable · disable · remove</code>
					<span class="text-muted-foreground">: manage one without leaving Discord.</span>
				</li>
				<li>
					<code class="text-foreground">/wh route</code>
					<span class="text-muted-foreground">
						: how far a system is from one of your chains, counting wormholes as free.
					</span>
				</li>
			</ul>
			<p class="mt-3 text-xs text-muted-foreground">
				Replies are only visible to you, even in a busy channel.
			</p>
		</Card.Content>
	</Card.Root>
</div>
