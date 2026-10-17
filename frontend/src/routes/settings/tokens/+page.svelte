<script lang="ts">
	// Personal access tokens: the keys a script presents instead of signing in. The secret
	// is shown once, in the dialog that created it; the table afterwards never carries it.
	import CopyIcon from '@lucide/svelte/icons/copy';
	import KeyRoundIcon from '@lucide/svelte/icons/key-round';
	import PlusIcon from '@lucide/svelte/icons/plus';
	import TrashIcon from '@lucide/svelte/icons/trash-2';

	import { createQuery, useQueryClient } from '@tanstack/svelte-query';
	import { toast } from 'svelte-sonner';

	import { page } from '$app/state';
	import { api, errorMessage } from '$lib/api/client';
	import { apiAction } from '$lib/api/mutations';
	import { key, q } from '$lib/api/queries';
	import type { PersonalAccessToken } from '$lib/api/types/PersonalAccessToken';
	import { copyText } from '$lib/clipboard';
	import { confirmDanger } from '$lib/confirm.svelte';
	import { Button } from '$lib/components/ui/button';
	import * as Card from '$lib/components/ui/card';
	import * as Dialog from '$lib/components/ui/dialog';
	import * as Field from '$lib/components/ui/field';
	import { Input } from '$lib/components/ui/input';
	import * as Select from '$lib/components/ui/select';
	import * as Table from '$lib/components/ui/table';
	import { timeAgo, utcShort } from '$lib/format';
	import { cn } from '$lib/utils';
	import Hint from '$lib/components/Hint.svelte';

	const tokensQuery = createQuery(() => q.myTokens());
	const tokens = $derived(tokensQuery.data ?? []);
	const error = $derived(tokensQuery.error ? errorMessage(tokensQuery.error) : null);
	const client = useQueryClient();

	const lifetimes = [
		{ value: 'never', label: 'Until revoked' },
		{ value: '30', label: '30 days' },
		{ value: '90', label: '90 days' },
		{ value: '365', label: 'A year' },
	];

	let open = $state(false);
	let name = $state('');
	let lifetime = $state('never');
	let creating = $state(false);
	let created = $state<{ name: string; plaintext: string } | null>(null);

	function startCreating() {
		name = '';
		lifetime = 'never';
		created = null;
		open = true;
	}

	async function create(event: SubmitEvent) {
		event.preventDefault();
		if (!name.trim() || creating) return;
		creating = true;
		try {
			const result = await api.createToken({
				name: name.trim(),
				expires_in_days: lifetime === 'never' ? undefined : Number(lifetime),
			});
			created = { name: result.token.name, plaintext: result.plaintext };
			await client.invalidateQueries({ queryKey: key.myTokens });
		} catch (err) {
			toast.error(errorMessage(err));
		} finally {
			creating = false;
		}
	}

	const act = apiAction(() => [key.myTokens]);

	async function revoke(token: PersonalAccessToken) {
		const sure = await confirmDanger({
			title: `Revoke ${token.name}?`,
			body: 'Whatever uses it is locked out on its next request. This cannot be undone.',
			action: 'Revoke',
		});
		if (!sure) return;
		act.mutate(() => api.revokeToken(token.id));
	}

	const DAY = 24 * 60 * 60 * 1000;

	type Expiry = { label: string; tone: 'none' | 'soon' | 'expired' };

	function expiry(token: PersonalAccessToken): Expiry {
		if (!token.expires_at) return { label: 'Never', tone: 'none' };
		const remaining = new Date(token.expires_at).getTime() - Date.now();
		if (remaining < 0) return { label: `Expired ${utcShort(token.expires_at)}`, tone: 'expired' };
		const days = Math.ceil(remaining / DAY);
		if (days <= 7) return { label: days === 1 ? 'Within a day' : `In ${days} days`, tone: 'soon' };
		return { label: utcShort(token.expires_at), tone: 'none' };
	}

	const snippet = $derived(`curl -H "Authorization: Bearer wst_..." ${page.url.origin}/api/maps`);
</script>

<div class="flex flex-col gap-6">
	{#if error}
		<p class="text-sm text-destructive" data-testid="tokens-error">{error}</p>
	{/if}

	<Card.Root>
		<Card.Header>
			<div class="flex items-start justify-between gap-3">
				<div class="flex flex-col gap-1.5">
					<Card.Title class="flex items-center gap-2">
						<KeyRoundIcon class="size-4" />
						API tokens
					</Card.Title>
					<Card.Description>
						A token acts as you on every map you can reach, with the role you hold there. Everything
						the map screen does is open to it.
					</Card.Description>
				</div>
				<Button size="sm" onclick={startCreating} data-testid="token-new">
					<PlusIcon data-icon="inline-start" />
					New token
				</Button>
			</div>
		</Card.Header>

		<Card.Content class="px-0">
			{#if tokens.length > 0}
				<Table.Root data-testid="tokens-list">
					<Table.Header>
						<Table.Row class="hover:bg-transparent">
							<Table.Head class="pl-4">Name</Table.Head>
							<Table.Head>Created</Table.Head>
							<Table.Head>Last used</Table.Head>
							<Table.Head>Expires</Table.Head>
							<Table.Head class="w-12"></Table.Head>
						</Table.Row>
					</Table.Header>
					<Table.Body class="[&_tr:last-child]:border-0">
						{#each tokens as token (token.id)}
							{@const until = expiry(token)}
							<Table.Row data-testid="token-row" data-token-name={token.name}>
								<Table.Cell class="max-w-64 truncate pl-4 font-medium">{token.name}</Table.Cell>
								<Table.Cell class="text-muted-foreground">{utcShort(token.created_at)}</Table.Cell>
								<Table.Cell class="text-muted-foreground">
									{token.last_used_at ? timeAgo(token.last_used_at) : 'Never'}
								</Table.Cell>
								<Table.Cell
									class={cn(
										'text-muted-foreground',
										until.tone === 'soon' && 'text-amber-500',
										until.tone === 'expired' && 'text-destructive',
									)}
								>
									{until.label}
								</Table.Cell>
								<Table.Cell class="pr-3 text-right">
									<Button
										variant="ghost"
										size="icon"
										class="size-8 text-muted-foreground hover:text-destructive"
										aria-label="Revoke {token.name}"
										onclick={() => revoke(token)}
										data-testid="token-revoke"
									>
										<TrashIcon />
									</Button>
								</Table.Cell>
							</Table.Row>
						{/each}
					</Table.Body>
				</Table.Root>
			{:else if !tokensQuery.isPending}
				<div
					class="flex flex-col items-center gap-3 border-t border-border/40 px-4 py-10 text-center"
					data-testid="tokens-empty"
				>
					<KeyRoundIcon class="size-5 text-muted-foreground/60" />
					<p class="text-sm text-muted-foreground">
						No tokens yet. Make one for each script, named after what it does.
					</p>
					<Button variant="outline" size="sm" onclick={startCreating}>
						<PlusIcon data-icon="inline-start" />
						New token
					</Button>
				</div>
			{/if}
		</Card.Content>

		<Card.Footer class="flex flex-wrap items-center justify-between gap-x-6 gap-y-2 border-t">
			<Hint text={snippet} clippedOnly focusable={false}>
				{#snippet child({ props })}
					<code class="min-w-0 truncate font-mono text-xs text-muted-foreground" {...props}>
						{snippet}
					</code>
				{/snippet}
			</Hint>
			<span class="flex shrink-0 items-center gap-3 text-xs text-muted-foreground">
				<a href="/api/docs" class="underline hover:text-foreground" target="_blank" rel="noopener">
					API reference
				</a>
				<a
					href="/documentation/contributing-and-self-hosting/using-the-api"
					class="underline hover:text-foreground"
				>
					Using the API
				</a>
			</span>
		</Card.Footer>
	</Card.Root>
</div>

<Dialog.Root bind:open>
	<Dialog.Content class="sm:max-w-md" data-testid="token-dialog">
		{#if created}
			<Dialog.Header>
				<Dialog.Title>Copy your token</Dialog.Title>
				<Dialog.Description>
					<span class="font-medium text-foreground">{created.name}</span> is ready. This is the only time
					it is shown.
				</Dialog.Description>
			</Dialog.Header>
			<div class="flex min-w-0 items-center gap-2" data-testid="token-created">
				<code
					class="min-w-0 flex-1 border border-border/60 bg-muted px-2.5 py-2 font-mono text-xs break-all"
					data-testid="token-plaintext">{created.plaintext}</code
				>
				<Button
					variant="outline"
					size="icon"
					aria-label="Copy token"
					onclick={() => created && copyText(created.plaintext, { success: 'Token copied' })}
					data-testid="token-copy"
				>
					<CopyIcon class="size-4" />
				</Button>
			</div>
			<p class="text-xs text-muted-foreground">
				Send it as <code class="text-foreground">Authorization: Bearer</code> on any request. If it is
				lost, revoke it and make another.
			</p>
			<Dialog.Footer class="mt-2">
				<Button onclick={() => (open = false)} data-testid="token-done">Done</Button>
			</Dialog.Footer>
		{:else}
			<Dialog.Header>
				<Dialog.Title>New token</Dialog.Title>
				<Dialog.Description>
					Name it after the script that will use it, so revoking the right one later is easy.
				</Dialog.Description>
			</Dialog.Header>
			<form onsubmit={create}>
				<Field.FieldGroup>
					<Field.Field>
						<Field.FieldLabel for="token-name">Name</Field.FieldLabel>
						<Input
							id="token-name"
							bind:value={name}
							placeholder="Fleet intel bot"
							maxlength={255}
							required
							autofocus
							data-testid="token-name"
						/>
					</Field.Field>
					<Field.Field>
						<Field.FieldLabel for="token-lifetime">Lifetime</Field.FieldLabel>
						<Select.Root type="single" bind:value={lifetime}>
							<Select.Trigger id="token-lifetime" class="w-full" data-testid="token-lifetime">
								{lifetimes.find((l) => l.value === lifetime)?.label}
							</Select.Trigger>
							<Select.Content>
								{#each lifetimes as option (option.value)}
									<Select.Item value={option.value} label={option.label} />
								{/each}
							</Select.Content>
						</Select.Root>
						<Field.FieldDescription>
							A token that expires is one less thing to remember to revoke.
						</Field.FieldDescription>
					</Field.Field>
				</Field.FieldGroup>
				<Dialog.Footer class="mt-4">
					<Button type="button" variant="ghost" onclick={() => (open = false)}>Cancel</Button>
					<Button type="submit" disabled={creating || !name.trim()} data-testid="token-create">
						Create
					</Button>
				</Dialog.Footer>
			</form>
		{/if}
	</Dialog.Content>
</Dialog.Root>
