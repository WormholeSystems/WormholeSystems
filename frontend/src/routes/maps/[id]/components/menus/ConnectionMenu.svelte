<script lang="ts">
	// Right-clicking a connection: its degradable statuses, its kind, and removal.
	import ChevronRightIcon from '@lucide/svelte/icons/chevron-right';
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
	import type { MapSystemView } from '$lib/api/types/MapSystemView';
	import type { MassStatus } from '$lib/api/types/MassStatus';
	import type { TimeStatus } from '$lib/api/types/TimeStatus';
	import type { WormholeSize } from '$lib/api/types/WormholeSize';
	import { copyText } from '$lib/clipboard';
	import { LIFETIME_OPTIONS, MASS_OPTIONS, SIZE_OPTIONS } from '$lib/map/connection-status';
	import { sizeForJumpMass } from '$lib/map/mass';
	import { typeById } from '$lib/map/signatures';
	import { formatBookmark, formatsFromNaming } from '$lib/naming/bookmark';
	import type { MapState } from '../../state/map-state.svelte';
	import { item, panel, sub } from './chrome';

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

	function close() {
		map.closeMenu();
	}

	/**
	 * The bookmark for each end, as it would be written in game. A bookmark is made while
	 * standing at the hole, so the name for one end carries the signature read on the
	 * other: the far side is what the name has to describe.
	 */
	const bookmarks = $derived.by(() => {
		const placed = (id: number) => map.systems.all.find((s) => s.id === id) ?? null;
		const from = placed(connection.from_system);
		const to = placed(connection.to_system);
		if (from === null || to === null) return [];
		return [bookmarkFor(from, to), bookmarkFor(to, from)].filter((name) => name !== null);
	});

	/** Null for a ghost: an unflown hole has no system on the far side to name. */
	function bookmarkFor(end: MapSystemView, opposite: MapSystemView): string | null {
		if (end.kind !== 'system') return null;
		const sig =
			opposite.kind === 'system'
				? (map.signatures.all.find(
						(s) => s.connection_id === cid && s.solar_system_id === opposite.solar_system_id,
					) ?? null)
				: null;
		return formatBookmark(
			{
				alias: end.alias,
				name: end.name,
				region: end.region,
				wormholeClassId: end.wormhole_class_id,
				security: end.security_status,
				occupier: end.occupying_group,
			},
			{
				signatureId: sig?.signature_id ?? null,
				size: connection.size,
				massStatus: connection.mass_status,
				timeStatus: connection.time_status,
				wormholeCode:
					catalog && sig ? (typeById(catalog, sig.signature_type_id)?.signature ?? null) : null,
			},
			formatsFromNaming(map.naming),
			opposite.alias,
		);
	}

	function copyBookmark(text: string) {
		void copyText(text, { success: 'Bookmark copied' });
		close();
	}

	function setKind(kind: ConnectionType) {
		map.connections.patch(cid, { kind });
		close();
	}

	function setMass(mass: MassStatus) {
		map.connections.patch(cid, { mass_status: mass });
		close();
	}

	function setLifetime(time: TimeStatus) {
		map.connections.patch(cid, { time_status: time });
		close();
	}

	function setSize(size: WormholeSize | null) {
		map.connections.patch(cid, { size });
		close();
	}

	function removeConnection() {
		map.connections.remove(cid);
		close();
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
	<div class={sub} data-testid="copy-name-subtrigger">
		<CopyIcon class="size-4" />
		Copy name
		<ChevronRightIcon class="ml-auto size-3" />
		<div class={panel} data-testid="copy-name-submenu">
			{#each bookmarks as name, i (i)}
				<button class={item} onclick={() => copyBookmark(name)}>{name}</button>
			{/each}
		</div>
	</div>
{/if}

<div class={sub} data-testid="lifetime-subtrigger">
	<ClockIcon class="size-4" />
	Lifetime
	<ChevronRightIcon class="ml-auto size-3" />
	<div class={panel} data-testid="lifetime-submenu">
		{#each LIFETIME_OPTIONS as o (o.value)}
			<button class={item} onclick={() => setLifetime(o.value)}>
				{@render dot(o.color, o.value === 'expired')}
				{o.label}
				<span class="ml-auto text-muted-foreground">{o.hint ?? ''}</span>
				{@render check(
					connection.time_status === o.value ||
						(o.value === 'stable' && connection.time_status === null),
				)}
			</button>
		{/each}
	</div>
</div>

<div class={sub} data-testid="mass-subtrigger">
	<WeightIcon class="size-4" />
	Mass Status
	<ChevronRightIcon class="ml-auto size-3" />
	<div class={panel} data-testid="mass-submenu">
		{#each MASS_OPTIONS as o (o.value)}
			<button class={item} onclick={() => setMass(o.value)}>
				{@render dot(o.color)}
				{o.label}
				<span class="ml-auto text-muted-foreground">{o.hint ?? ''}</span>
				{@render check(
					connection.mass_status === o.value ||
						(o.value === 'stable' && connection.mass_status === null),
				)}
			</button>
		{/each}
	</div>
</div>

<div class={sub} data-testid="size-subtrigger">
	<ShipIcon class="size-4" />
	Ship Size
	<ChevronRightIcon class="ml-auto size-3" />
	<div class={panel} data-testid="size-submenu">
		{#if lockingType}
			<div class="px-3 py-1 text-[10px] text-muted-foreground" data-testid="size-locked-hint">
				Set by {lockingType.signature}
			</div>
		{/if}
		<!-- Nobody has read the hole yet, which is not the same as it taking a battleship. -->
		<button
			class="{item} disabled:cursor-default disabled:opacity-50 disabled:hover:bg-transparent"
			disabled={lockingType !== null}
			onclick={() => setSize(null)}
		>
			<span class="inline-flex w-6 justify-center font-mono text-[10px] text-muted-foreground">
				?
			</span>
			Unknown
			<span class="ml-auto"></span>
			{@render check(connection.size === null)}
		</button>
		{#each SIZE_OPTIONS as o (o.value)}
			<button
				class="{item} disabled:cursor-default disabled:opacity-50 disabled:hover:bg-transparent"
				disabled={lockingType !== null}
				onclick={() => setSize(o.value)}
			>
				<span class="inline-flex w-6 justify-center font-mono text-[10px] text-muted-foreground">
					{o.letter}
				</span>
				{o.label}
				<span class="ml-auto"></span>
				{@render check(connection.size === o.value)}
			</button>
		{/each}
	</div>
</div>

<div class={sub} data-testid="type-subtrigger">
	<WaypointsIcon class="size-4" />
	Connection type
	<ChevronRightIcon class="ml-auto size-3" />
	<div class={panel} data-testid="type-submenu">
		<button class={item} onclick={() => setKind('wormhole')}>
			Wormhole
			<span class="ml-auto"></span>
			{@render check(connection.kind === 'wormhole')}
		</button>
		<button class={item} onclick={() => setKind('stargate')}>
			Stargate
			{#if connection.kind !== 'stargate'}
				<TriangleAlertIcon class="ml-auto size-3.5 text-amber-500" />
			{:else}
				<span class="ml-auto"></span>
				{@render check(true)}
			{/if}
		</button>
	</div>
</div>

<div class="my-0.5 border-t border-border"></div>
<button class="{item} text-destructive hover:text-destructive" onclick={removeConnection}>
	<Trash2Icon class="size-4" />
	Remove
</button>
