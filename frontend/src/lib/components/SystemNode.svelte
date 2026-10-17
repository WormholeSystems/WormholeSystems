<script lang="ts">
	// One placed system. Five styling channels stack on it: status border, selected
	// background, active ring, hover outline, and the pink glow and tint of a rally point.
	import type { MapCharacter } from '$lib/api/types/MapCharacter';
	import type { MapSystemView } from '$lib/api/types/MapSystemView';
	import type { SigCounts } from '$lib/map/grouping';
	import { Button } from '$lib/components/ui/button';
	import { Input } from '$lib/components/ui/input';
	import * as Popover from '$lib/components/ui/popover';
	import ClassBadge from '$lib/components/ClassBadge.svelte';
	import { isWormholeClass } from '$lib/map/classes';
	import { NODE_W } from '$lib/map/geometry';
	import { statusColor } from '$lib/map/status';
	import HeaderIcons from './system-node/HeaderIcons.svelte';
	import PilotsRow from './system-node/PilotsRow.svelte';
	import RegionRow from './system-node/RegionRow.svelte';
	import StaticsRow from './system-node/StaticsRow.svelte';
	import Hint from '$lib/components/Hint.svelte';

	let {
		node,
		nodeH,
		selected,
		active = false,
		highlighted = false,
		pos,
		sigCounts,
		connectionCount,
		pilots = [],
		showThreat = true,
		onselect,
		ondown,
		onlink,
		linkable = true,
		editable = true,
		onmenu,
		onsavealias,
		draggable = true,
		signatureId = null,
	}: {
		node: MapSystemView;
		nodeH: number;
		selected: boolean;
		active?: boolean;
		/** Pointed at from a side panel row, so it can be found without reading labels. */
		highlighted?: boolean;
		pos: { x: number; y: number };
		sigCounts: SigCounts;
		connectionCount: number;
		pilots?: MapCharacter[];
		showThreat?: boolean;
		onselect: (ev: PointerEvent) => void;
		ondown: (ev: PointerEvent) => void;
		onlink: (ev: PointerEvent) => void;
		/** An automatic layout owns the positions, so the drag handle goes away. */
		draggable?: boolean;
		/** A viewer draws no connections, so the handle that starts one is not offered. */
		linkable?: boolean;
		/** Same for the alias editor a double-click opens. */
		editable?: boolean;
		/** The scanner id an unmapped hole is known by, until it has a system. */
		signatureId?: string | null;
		onmenu: (ev: MouseEvent) => void;
		onsavealias: (alias: string | null, occupier: string | null) => void;
	} = $props();

	// A hole nobody has been through: drawn as a node so the chain can be laid out and named,
	// dashed so it never reads as somewhere you can actually go.
	const mapped = $derived(node.kind === 'system' ? node : null);
	const ghost = $derived(mapped === null);

	// Suppressed while active, so the amber ring is not fighting the threat ring.
	const threatRing = $derived(
		showThreat &&
			!active &&
			(mapped?.threat_level === 'high' || mapped?.threat_level === 'critical')
			? mapped.threat_level
			: null,
	);
	const showStatics = $derived(
		isWormholeClass(mapped?.wormhole_class_id ?? null) && (mapped?.statics.length ?? 0) > 0,
	);
	const unmapped = $derived(Math.max(0, sigCounts.wormholes - connectionCount));

	let editorOpen = $state(false);
	// The editor is mounted the first time it is asked for and kept after, so most nodes
	// never build one at all while an opened one still animates closed.
	let editorMounted = $state(false);
	let editAlias = $state('');
	let editOccupier = $state('');

	function openEditor() {
		editAlias = node.alias ?? '';
		editOccupier = mapped?.occupying_group ?? '';
		editorMounted = true;
		editorOpen = true;
	}

	function saveEditor() {
		onsavealias(editAlias.trim() || null, editOccupier.trim() || null);
		editorOpen = false;
	}
</script>

<!-- svelte-ignore a11y_no_static_element_interactions -->
<div
	data-testid="system-node"
	data-status={node.status}
	data-threat={threatRing}
	data-ghost={ghost ? 'true' : undefined}
	data-rally={node.is_rally ? 'true' : undefined}
	class="group/node absolute flex flex-col justify-center rounded border px-2 py-0.5 text-[11px] leading-tight transition-colors duration-200
		{ghost ? 'border-dashed' : ''}
		{selected
		? 'bg-amber-100 dark:bg-amber-900'
		: node.is_rally
			? 'bg-pink-50 dark:bg-pink-950'
			: 'bg-card'}
		{node.is_rally ? 'shadow-[0_0_10px_2px] shadow-pink-500/60' : 'shadow-sm'}
		{active ? 'z-10 ring-2 ring-amber-500 ring-offset-2 ring-offset-background' : ''}
		{highlighted ? 'z-20 outline-2 outline-yellow-500' : ''}
		{threatRing ? 'ring-2' : ''}
		hover:z-20 hover:outline-2 hover:outline-yellow-500"
	style:--tw-ring-color={threatRing ? `var(--color-threat-${threatRing})` : null}
	style:border-color={statusColor(node.status)}
	style:width="{NODE_W}px"
	style:height="{pilots.length > 0 ? nodeH + 20 : nodeH}px"
	style:left="{pos.x}px"
	style:top="{pos.y}px"
	onpointerdown={onselect}
	oncontextmenu={onmenu}
	ondblclick={() => editable && openEditor()}
>
	{#if !node.is_pinned && draggable}
		<!-- svelte-ignore a11y_no_static_element_interactions -->
		<div
			class="absolute top-[1px] left-1/2 hidden h-2 w-12 -translate-x-1/2 -translate-y-1/2 cursor-move rounded border border-neutral-300 bg-white group-hover/node:z-50 group-hover/node:block dark:border-neutral-600 dark:bg-neutral-700"
			data-testid="drag-handle"
			onpointerdown={(ev) => {
				ev.stopPropagation();
				ondown(ev);
			}}
		></div>
	{/if}
	<!-- No connection handle on a ghost: a connection out of it would claim the unknown
	     system on its far side leads somewhere, which nobody knows yet. -->
	{#if !ghost && linkable}
		<!-- svelte-ignore a11y_no_static_element_interactions -->
		<div
			class="absolute top-1/2 left-full hidden h-4 w-4 -translate-x-1/2 -translate-y-1/2 cursor-pointer rounded-full border border-neutral-300 bg-white group-hover/node:block hover:block dark:border-neutral-600 dark:bg-neutral-700"
			data-testid="connection-handle"
			onpointerdown={onlink}
		></div>
	{/if}

	<div class="flex min-w-0 items-center gap-1">
		<ClassBadge
			classId={mapped?.wormhole_class_id ?? null}
			security={mapped?.security_status ?? null}
			class="shrink-0 font-medium"
		/>
		{#if node.alias}
			<span class="shrink-0 font-medium text-foreground">{node.alias}</span>
			<span class="truncate text-muted-foreground">
				{mapped?.name ?? signatureId ?? 'Unmapped'}
			</span>
		{:else if mapped}
			<span class="truncate font-medium text-foreground">{mapped.name}</span>
		{:else if signatureId}
			<span class="truncate font-medium text-muted-foreground" data-testid="ghost-signature">
				{signatureId}
			</span>
		{:else}
			<span class="truncate font-medium text-muted-foreground italic">Unmapped</span>
		{/if}
		{#if mapped?.occupying_group}
			<!-- Truncated like the name: a long one would push the icons out of the node. The
			     full text is in its title, and on the system's info card. -->
			<Hint text={mapped.occupying_group} clippedOnly focusable={false}>
				{#snippet child({ props })}
					<span class="truncate text-muted-foreground" {...props} data-testid="node-occupier">
						({mapped.occupying_group})
					</span>
				{/snippet}
			</Hint>
		{/if}

		<HeaderIcons {node} {sigCounts} {unmapped} />
	</div>

	{#if mapped && showStatics}
		<StaticsRow statics={mapped.statics} />
	{:else}
		<RegionRow region={mapped?.region ?? null} />
	{/if}

	{#if pilots.length > 0}
		<PilotsRow {pilots} />
	{/if}

	{#if editorMounted}
		<Popover.Root bind:open={editorOpen}>
			<Popover.Trigger class="pointer-events-none absolute inset-x-0 top-0" tabindex={-1} />
			<Popover.Content
				class="flex w-56 flex-col gap-2 p-2"
				onpointerdown={(ev: PointerEvent) => ev.stopPropagation()}
				ondblclick={(ev: MouseEvent) => ev.stopPropagation()}
			>
				<Input
					placeholder="Alias"
					bind:value={editAlias}
					onkeydown={(ev) => ev.key === 'Enter' && saveEditor()}
				/>
				{#if !ghost}
					<Input
						placeholder="Occupier alias"
						bind:value={editOccupier}
						onkeydown={(ev) => ev.key === 'Enter' && saveEditor()}
					/>
				{/if}
				<Button size="sm" onclick={saveEditor}>Save</Button>
			</Popover.Content>
		</Popover.Root>
	{/if}
</div>
