<script lang="ts">
	// The list of who may see a map: one row per grant, with the role editable in place by
	// anyone who manages the map.
	//
	// Takes the grants and the writes it can make rather than the page around it, so the
	// settings screen and any read-only rendering of it are the same component and cannot
	// drift apart. With no `actions` nothing here can be written.
	import TrashIcon from '@lucide/svelte/icons/trash-2';

	import type { AccessEntry } from '$lib/api/types/AccessEntry';
	import type { Role } from '$lib/api/types/Role';
	import EveImage from '$lib/components/EveImage.svelte';
	import { Badge } from '$lib/components/ui/badge';
	import { Button } from '$lib/components/ui/button';
	import * as Select from '$lib/components/ui/select';
	import * as Table from '$lib/components/ui/table';
	import SortHeader from './SortHeader.svelte';
	import { ROLE_LABEL } from '$lib/map/roles';
	import Hint from '$lib/components/Hint.svelte';

	export type SortKey = 'name' | 'subject_type' | 'role' | 'expires_at';

	export interface AccessActions {
		setRole(entry: AccessEntry, role: Role): void;
		revoke(entry: AccessEntry): void;
		clearExpiry(entry: AccessEntry): void;
	}

	let {
		entries,
		canManage = false,
		sort,
		onsort,
		actions,
	}: {
		/** Already filtered and sorted; this only draws them. */
		entries: AccessEntry[];
		canManage?: boolean;
		/** Omitted where the header is not sortable, which leaves the arrows off. */
		sort?: { column: SortKey; direction: 'asc' | 'desc' };
		onsort?: (key: SortKey) => void;
		actions?: AccessActions;
	} = $props();

	const COLUMNS: { key: SortKey; label: string }[] = [
		{ key: 'name', label: 'Who' },
		{ key: 'subject_type', label: 'Kind' },
		{ key: 'role', label: 'Role' },
		{ key: 'expires_at', label: 'Ends' },
	];

	// Owner is not offered: it moves by transfer, not by picking it from a list.
	const ASSIGNABLE: Role[] = ['viewer', 'member', 'manager'];
</script>

<Table.Root data-testid="access-list">
	<Table.Header>
		<Table.Row>
			{#each COLUMNS as column (column.key)}
				<Table.Head>
					{#if onsort}
						<SortHeader column={column.key} {sort} {onsort} testid="sort-{column.key}">
							{column.label}
						</SortHeader>
					{:else}
						{column.label}
					{/if}
				</Table.Head>
			{/each}
			<Table.Head class="w-24"></Table.Head>
		</Table.Row>
	</Table.Header>
	<Table.Body>
		{#each entries as entry (entry.subject_id)}
			<Table.Row data-testid="access-row">
				<Table.Cell>
					<span class="flex min-w-0 items-center gap-2">
						<EveImage
							kind={entry.subject_type}
							id={entry.subject_id}
							size={64}
							title={entry.name ?? String(entry.subject_id)}
							class="size-7 shrink-0 rounded-sm"
						/>
						<span class="truncate">{entry.name ?? `Unknown (${entry.subject_id})`}</span>
					</span>
				</Table.Cell>
				<Table.Cell class="text-muted-foreground">{entry.subject_type}</Table.Cell>
				<Table.Cell>
					{#if entry.role === 'owner'}
						<Badge variant="outline">{ROLE_LABEL.owner}</Badge>
					{:else if canManage && actions}
						<Select.Root
							type="single"
							value={entry.role}
							onValueChange={(role) => actions?.setRole(entry, role as Role)}
						>
							<Select.Trigger class="w-28">{ROLE_LABEL[entry.role]}</Select.Trigger>
							<Select.Content>
								<Select.Group>
									{#each ASSIGNABLE as r (r)}
										<Select.Item value={r} label={ROLE_LABEL[r]}>{ROLE_LABEL[r]}</Select.Item>
									{/each}
								</Select.Group>
							</Select.Content>
						</Select.Root>
					{:else}
						<Badge variant="outline">{ROLE_LABEL[entry.role]}</Badge>
					{/if}
				</Table.Cell>
				<Table.Cell>
					{#if entry.expires_at}
						{@const expires = entry.expires_at}
						<Hint text={canManage ? 'Remove the end date' : undefined}>
							{#snippet child({ props })}
								<button
									type="button"
									class="text-xs text-amber-500 hover:underline disabled:no-underline"
									data-testid="access-expiry"
									{...props}
									disabled={!canManage || !actions}
									onclick={() => actions?.clearExpiry(entry)}
								>
									{new Date(expires).toLocaleDateString(undefined, {
										day: 'numeric',
										month: 'short',
										year: 'numeric',
									})}
								</button>
							{/snippet}
						</Hint>
					{:else}
						<span class="text-xs text-muted-foreground">—</span>
					{/if}
				</Table.Cell>
				<Table.Cell class="text-right">
					{#if canManage && actions && entry.role !== 'owner'}
						<Button
							variant="ghost"
							size="icon"
							class="size-7 text-muted-foreground hover:text-destructive"
							aria-label="Revoke access for {entry.name ?? entry.subject_id}"
							onclick={() => actions?.revoke(entry)}
						>
							<TrashIcon />
						</Button>
					{/if}
				</Table.Cell>
			</Table.Row>
		{/each}
	</Table.Body>
</Table.Root>
