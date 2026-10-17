<script lang="ts">
	// Who can see the map, and what they may do on it. A grant can target a character,
	// their corporation or their alliance, and the server refuses anything that would leave
	// the map without an owner.
	import { createQuery } from '@tanstack/svelte-query';
	import { page } from '$app/state';
	import { toast } from 'svelte-sonner';

	import { api } from '$lib/api/client';
	import { after, apiAction } from '$lib/api/mutations';
	import { copyText } from '$lib/clipboard';
	import { confirmDanger } from '$lib/confirm.svelte';
	import { key, q } from '$lib/api/queries';
	import type { AccessEntry } from '$lib/api/types/AccessEntry';
	import { Button } from '$lib/components/ui/button';
	import * as Card from '$lib/components/ui/card';

	import { Input } from '$lib/components/ui/input';
	import AccessTable from '$lib/components/AccessTable.svelte';
	import { sortState } from '$lib/sort-state.svelte';
	import { Switch } from '$lib/components/ui/switch';
	import SettingRow from '$lib/components/settings/SettingRow.svelte';
	import { byRole, ROLE_LABEL } from '$lib/map/roles';
	import GrantForm from './GrantForm.svelte';
	import { mapSettings } from '../context.svelte';
	import Hint from '$lib/components/Hint.svelte';

	const ctx = mapSettings();
	const mapId = $derived(ctx.mapId);
	const view = $derived(ctx.view);
	const accessQuery = createQuery(() => q.listAccess(mapId));
	const access = $derived(accessQuery.data ?? []);

	const canManage = $derived(ctx.canManage);

	let filter = $state('');
	const sort = sortState('map-access-sort', ['name', 'subject_type', 'role', 'expires_at'], {
		column: 'role',
		direction: 'asc',
	});

	const shown = $derived.by(() => {
		const needle = filter.trim().toLowerCase();
		const rows = access.filter(
			(e) =>
				!needle ||
				(e.name ?? '').toLowerCase().includes(needle) ||
				String(e.subject_id).includes(needle),
		);
		const compare = (a: AccessEntry, b: AccessEntry) => {
			switch (sort.current.column) {
				case 'role':
					return byRole(a.role, b.role);
				case 'subject_type':
					return a.subject_type.localeCompare(b.subject_type);
				case 'expires_at':
					// The ones that end come first; permanent grants sort last.
					return (
						(a.expires_at ? Date.parse(a.expires_at) : Infinity) -
						(b.expires_at ? Date.parse(b.expires_at) : Infinity)
					);
				default:
					return (a.name ?? '').localeCompare(b.name ?? '');
			}
		};
		return [...rows].sort(
			(a, b) =>
				(sort.current.direction === 'desc' ? -1 : 1) *
				(compare(a, b) || (a.name ?? '').localeCompare(b.name ?? '')),
		);
	});

	async function keepForever(entry: AccessEntry) {
		const until = entry.expires_at ? new Date(entry.expires_at).toLocaleString() : 'the date set';
		const sure = await confirmDanger({
			title: 'Drop the end date?',
			body:
				`${entry.name ?? 'They'} keeps ${ROLE_LABEL[entry.role].toLowerCase()} access until ` +
				`somebody takes it away, instead of until ${until}.`,
			action: 'Drop it',
			cancel: 'Keep the date',
		});
		if (!sure) return;
		act.mutate(() =>
			api.setAccess({
				map_id: mapId,
				subject_type: entry.subject_type,
				subject_id: entry.subject_id,
				role: entry.role,
				expires_at: null,
			}),
		);
	}

	// Only a manager sees the link: the API withholds the token from anyone else.
	const shareUrl = $derived(
		view.map.share_token ? `${page.url.origin}/share/${view.map.share_token}` : '',
	);

	function rotateShare() {
		after(
			act.mutateAsync(() => api.shareMap(mapId)),
			() => toast.success('Share link ready'),
		);
	}

	async function revokeShare() {
		const sure = await confirmDanger({
			title: 'Withdraw the share link?',
			body:
				'Everyone watching through it loses the map at once. People with a grant are not ' +
				'affected, and you can always make a new link.',
			action: 'Withdraw it',
			cancel: 'Keep it',
		});
		if (!sure) return;
		after(
			act.mutateAsync(() => api.unshareMap(mapId)),
			() => toast.success('Share link withdrawn'),
		);
	}

	function copyShare() {
		void copyText(shareUrl, { success: 'Share link copied' });
	}

	// Grants change the access list, and rotating the share link changes the view.
	const act = apiAction(() => [key.access(mapId), key.mapView(mapId)]);
</script>

<Card.Root>
	<Card.Header>
		<Card.Title>Access</Card.Title>
		<Card.Description>Granting a corporation or alliance covers every pilot in it.</Card.Description
		>
	</Card.Header>
	<Card.Content class="flex flex-col gap-4">
		{#if canManage}
			<GrantForm
				ongrant={(grant) => act.mutateAsync(() => api.setAccess({ map_id: mapId, ...grant }))}
			/>
		{/if}

		<div class="flex items-center gap-2">
			<Input
				bind:value={filter}
				placeholder="Filter by name…"
				class="h-7 w-56 text-xs"
				data-testid="access-filter"
			/>
			<span class="text-xs text-muted-foreground">
				{shown.length} of {access.length}
			</span>
		</div>

		<AccessTable
			entries={shown}
			{canManage}
			sort={sort.current}
			onsort={sort.toggle}
			actions={{
				setRole: (entry, role) =>
					act.mutate(() =>
						api.setAccess({
							map_id: mapId,
							subject_type: entry.subject_type,
							subject_id: entry.subject_id,
							role,
						}),
					),
				revoke: (entry) =>
					act.mutate(() => api.revokeAccess({ map_id: mapId, subject_id: entry.subject_id })),
				clearExpiry: (entry) => keepForever(entry),
			}}
		/>
	</Card.Content>
</Card.Root>

<!-- Sharing lives here because it answers the same question as the grants (who can see the
     chain) for people without an account. -->
{#if canManage}
	<Card.Root class="mt-6" data-testid="sharing-card">
		<Card.Header>
			<Card.Title>Sharing</Card.Title>
			<Card.Description>
				Both open the map itself, read-only: the chain as it is scanned, with no editing and no
				pilots.
			</Card.Description>
		</Card.Header>
		<Card.Content class="flex flex-col py-0">
			<SettingRow
				id="share-link"
				label="Share link"
				description="Anyone holding this address can watch the map without an account; following it opens the map itself. Making a new link locks out whoever had the old one."
			>
				{#snippet control()}
					<span class="flex items-center gap-2">
						{#if shareUrl}
							<Input
								readonly
								value={shareUrl}
								class="h-7 w-64 font-mono text-[11px]"
								data-testid="share-url"
								onfocus={(e) => e.currentTarget.select()}
							/>
							<Button variant="outline" onclick={copyShare} data-testid="share-copy">Copy</Button>
							<Hint text={'Replace the link'}>
								{#snippet child({ props })}
									<Button variant="ghost" onclick={rotateShare} {...props}>New</Button>
								{/snippet}
							</Hint>
							<Button
								variant="ghost"
								class="text-destructive hover:text-destructive"
								onclick={revokeShare}
								data-testid="share-revoke"
							>
								Revoke
							</Button>
						{:else}
							<Button variant="outline" onclick={rotateShare} data-testid="share-create">
								Create a link
							</Button>
						{/if}
					</span>
				{/snippet}
			</SettingRow>

			<SettingRow
				id="map-public"
				label="Public map"
				description="Anyone at all can watch it, with no link needed and nothing to guess. Turn this on for a map you would put on a forum post."
			>
				{#snippet control()}
					<Switch
						checked={view.map.is_public}
						aria-label="Public map"
						data-testid="share-public"
						onCheckedChange={(v) =>
							act.mutate(() => api.updateMap({ map_id: mapId, is_public: v }))}
					/>
				{/snippet}
			</SettingRow>
		</Card.Content>
	</Card.Root>
{/if}
