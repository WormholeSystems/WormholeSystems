<script lang="ts">
	// Right-clicking a connection: its degradable statuses, its kind, and removal.
	import CheckIcon from '@lucide/svelte/icons/check';
	import ClockIcon from '@lucide/svelte/icons/clock';
	import CopyIcon from '@lucide/svelte/icons/copy';
	import ShipIcon from '@lucide/svelte/icons/ship';
	import Trash2Icon from '@lucide/svelte/icons/trash-2';
	import TriangleAlertIcon from '@lucide/svelte/icons/triangle-alert';
	import WaypointsIcon from '@lucide/svelte/icons/waypoints';
	import WeightIcon from '@lucide/svelte/icons/weight';
	import { createQuery } from '@tanstack/svelte-query';

	import { q } from '$lib/api/queries';
	import type { ConnectionType } from '$lib/api/types/ConnectionType';
	import type { MapConnection } from '$lib/api/types/MapConnection';
	import type { MassStatus } from '$lib/api/types/MassStatus';
	import type { TimeStatus } from '$lib/api/types/TimeStatus';
	import type { WormholeSize } from '$lib/api/types/WormholeSize';
	import { copyText } from '$lib/clipboard';
	import * as DropdownMenu from '$lib/components/ui/dropdown-menu';
	import { LIFETIME_OPTIONS, MASS_OPTIONS, SIZE_OPTIONS } from '$lib/map/connection-status';
	import { sizeForJumpMass } from '$lib/map/mass';
	import { typeById } from '$lib/map/signatures';
	import { connectionBookmarks, formatsFromNaming } from '$lib/naming/bookmark';
	import type { MapState } from '../../state/map-state.svelte';

	let { map, connection }: { map: MapState; connection: MapConnection } = $props();

	const cid = $derived(connection.id);

	const catalogQuery = createQuery(() => q.signatureCatalog());
	const catalog = $derived(catalogQuery.data ?? null);

	/** The linked signature type that identifies the hole; its jump mass dictates the size. */
	const lockingType = $derived.by(() => {
		if (!catalog) return null;
		for (const sig of map.signatures.all) {
			if (sig.connection_id !== cid) continue;
			const type = typeById(catalog, sig.signature_type_id);
			if (sizeForJumpMass(type?.max_jump_mass)) return type;
		}
		return null;
	});

	/** The bookmark for each end, as it would be written in game. */
	const bookmarks = $derived(
		connectionBookmarks(
			connection,
			map.systems.all,
			map.signatures.all,
			(typeId) => (catalog ? (typeById(catalog, typeId)?.signature ?? null) : null),
			formatsFromNaming(map.naming),
		),
	);

	function copyBookmark(text: string) {
		void copyText(text, { success: 'Bookmark copied' });
	}

	function setKind(kind: ConnectionType) {
		map.connections.patch(cid, { kind });
	}

	function setMass(mass: MassStatus) {
		map.connections.patch(cid, { mass_status: mass });
	}

	function setLifetime(time: TimeStatus) {
		map.connections.patch(cid, { time_status: time });
	}

	function setSize(size: WormholeSize | null) {
		map.connections.patch(cid, { size });
	}

	function removeConnection() {
		map.connections.remove(cid);
	}
</script>

{#snippet dot(color: string, pulse = false)}
	<span
		class="inline-block size-2 shrink-0 rounded-full {pulse ? 'motion-safe:animate-pulse' : ''}"
		style="background-color: {color}"
	></span>
{/snippet}

{#snippet check(selected: boolean)}
	{#if selected}
		<CheckIcon class="size-3.5 shrink-0" />
	{/if}
{/snippet}

{#if bookmarks.length > 0}
	<DropdownMenu.Sub>
		<DropdownMenu.SubTrigger data-testid="copy-name-subtrigger">
			<CopyIcon class="size-4" />
			Copy name
		</DropdownMenu.SubTrigger>
		<DropdownMenu.SubContent data-testid="copy-name-submenu">
			{#each bookmarks as name, i (i)}
				<DropdownMenu.Item onSelect={() => copyBookmark(name)}>{name}</DropdownMenu.Item>
			{/each}
		</DropdownMenu.SubContent>
	</DropdownMenu.Sub>
{/if}

<DropdownMenu.Sub>
	<DropdownMenu.SubTrigger data-testid="lifetime-subtrigger">
		<ClockIcon class="size-4" />
		Lifetime
	</DropdownMenu.SubTrigger>
	<DropdownMenu.SubContent class="w-48" data-testid="lifetime-submenu">
		{#each LIFETIME_OPTIONS as o (o.value)}
			<DropdownMenu.Item onSelect={() => setLifetime(o.value)}>
				{@render dot(o.color, o.value === 'expired')}
				{o.label}
				<span class="ml-auto text-muted-foreground">{o.hint ?? ''}</span>
				{@render check(
					connection.time_status === o.value ||
						(o.value === 'stable' && connection.time_status === null),
				)}
			</DropdownMenu.Item>
		{/each}
	</DropdownMenu.SubContent>
</DropdownMenu.Sub>

<DropdownMenu.Sub>
	<DropdownMenu.SubTrigger data-testid="mass-subtrigger">
		<WeightIcon class="size-4" />
		Mass Status
	</DropdownMenu.SubTrigger>
	<DropdownMenu.SubContent class="w-48" data-testid="mass-submenu">
		{#each MASS_OPTIONS as o (o.value)}
			<DropdownMenu.Item onSelect={() => setMass(o.value)}>
				{@render dot(o.color)}
				{o.label}
				<span class="ml-auto text-muted-foreground">{o.hint ?? ''}</span>
				{@render check(
					connection.mass_status === o.value ||
						(o.value === 'stable' && connection.mass_status === null),
				)}
			</DropdownMenu.Item>
		{/each}
	</DropdownMenu.SubContent>
</DropdownMenu.Sub>

<DropdownMenu.Sub>
	<DropdownMenu.SubTrigger data-testid="size-subtrigger">
		<ShipIcon class="size-4" />
		Ship Size
	</DropdownMenu.SubTrigger>
	<DropdownMenu.SubContent class="w-48" data-testid="size-submenu">
		{#if lockingType}
			<div class="px-2 py-1 text-[10px] text-muted-foreground" data-testid="size-locked-hint">
				Set by {lockingType.signature}
			</div>
		{/if}
		<!-- Nobody has read the hole yet, which is not the same as it taking a battleship. -->
		<DropdownMenu.Item disabled={lockingType !== null} onSelect={() => setSize(null)}>
			<span class="inline-flex w-6 justify-center font-mono text-[10px] text-muted-foreground">
				?
			</span>
			Unknown
			<span class="ml-auto"></span>
			{@render check(connection.size === null)}
		</DropdownMenu.Item>
		{#each SIZE_OPTIONS as o (o.value)}
			<DropdownMenu.Item disabled={lockingType !== null} onSelect={() => setSize(o.value)}>
				<span class="inline-flex w-6 justify-center font-mono text-[10px] text-muted-foreground">
					{o.letter}
				</span>
				{o.label}
				<span class="ml-auto"></span>
				{@render check(connection.size === o.value)}
			</DropdownMenu.Item>
		{/each}
	</DropdownMenu.SubContent>
</DropdownMenu.Sub>

<DropdownMenu.Sub>
	<DropdownMenu.SubTrigger data-testid="type-subtrigger">
		<WaypointsIcon class="size-4" />
		Connection type
	</DropdownMenu.SubTrigger>
	<DropdownMenu.SubContent class="w-40" data-testid="type-submenu">
		<DropdownMenu.Item onSelect={() => setKind('wormhole')}>
			Wormhole
			<span class="ml-auto"></span>
			{@render check(connection.kind === 'wormhole')}
		</DropdownMenu.Item>
		<DropdownMenu.Item onSelect={() => setKind('stargate')}>
			Stargate
			{#if connection.kind !== 'stargate'}
				<TriangleAlertIcon class="ml-auto size-3.5 text-amber-500" />
			{:else}
				<span class="ml-auto"></span>
				{@render check(true)}
			{/if}
		</DropdownMenu.Item>
	</DropdownMenu.SubContent>
</DropdownMenu.Sub>

<DropdownMenu.Separator />
<DropdownMenu.Item variant="destructive" onSelect={removeConnection}>
	<Trash2Icon class="size-4" />
	Remove
</DropdownMenu.Item>
