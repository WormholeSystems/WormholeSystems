<script lang="ts">
	// The map's chrome strip. Everything here is about the map as a whole or about the viewer,
	// never about one system.
	import ArrowLeftIcon from '@lucide/svelte/icons/arrow-left';
	import EyeIcon from '@lucide/svelte/icons/eye';
	import LayoutGridIcon from '@lucide/svelte/icons/layout-grid';
	import RadarIcon from '@lucide/svelte/icons/radar';
	import SearchIcon from '@lucide/svelte/icons/search';
	import Redo2Icon from '@lucide/svelte/icons/redo-2';
	import CircleHelpIcon from '@lucide/svelte/icons/circle-help';
	import SettingsIcon from '@lucide/svelte/icons/settings';
	import BrushCleaningIcon from '@lucide/svelte/icons/brush-cleaning';
	import TriangleAlertIcon from '@lucide/svelte/icons/triangle-alert';
	import Undo2Icon from '@lucide/svelte/icons/undo-2';
	import type { UpdateMapUserSettings } from '$lib/api/types/UpdateMapUserSettings';
	import { solarSystemId } from '$lib/map/system';

	import { page } from '$app/state';

	import { Badge } from '$lib/components/ui/badge';
	import { Button } from '$lib/components/ui/button';
	import { Separator } from '$lib/components/ui/separator';
	import * as Tooltip from '$lib/components/ui/tooltip';
	import ClassBadge from '$lib/components/ClassBadge.svelte';
	import { cn } from '$lib/utils';
	import type { MapState } from '../../state/map-state.svelte';
	import HistoryPopover from '../overlays/HistoryPopover.svelte';
	import StaleConnectionsPopover from '../connection/StaleConnectionsPopover.svelte';
	import TrackingSettings from '../tracking/TrackingSettings.svelte';

	let { map, onstarttour }: { map: MapState; onstarttour?: () => void } = $props();

	const canWrite = $derived(map.canWrite);
	// Somebody following a share link has no pilot, so the pilot warnings have nobody to be
	// about.
	const watching = $derived(page.data.me == null);

	// Resolved against the map's own systems for the class chip; a pilot outside the chain
	// still gets their system id.
	const pilot = $derived(map.characters.mine.find((c) => c.is_active) ?? null);
	const pilotSystem = $derived(
		map.systems.all.find((s) => solarSystemId(s) === pilot?.solar_system_id) ?? null,
	);

	const socketLabel = {
		connecting: 'Connecting to the live feed',
		open: 'Live: changes from other pilots arrive automatically',
		reconnecting: 'Disconnected. Retrying, the map may be out of date',
		revoked: 'You no longer have access to this map',
	} satisfies Record<typeof map.socket, string>;

	function toggleSetting(key: 'tracking_allowed' | 'show_threat_level') {
		const current = map.userSettings;
		if (!current) return;
		const patch: UpdateMapUserSettings = {};
		patch[key] = !current[key];
		map.patchUserSettings(patch);
	}
</script>

{#snippet toggle(
	label: string,
	on: boolean,
	Icon: typeof EyeIcon,
	key: 'tracking_allowed' | 'show_threat_level',
	testid: string,
)}
	<Tooltip.Root>
		<Tooltip.Trigger>
			{#snippet child({ props })}
				<!-- The hairline is what says "on" while the pointer is over it: ghost's hover
				     repaints the icon, so colour alone cannot carry the state. -->
				<Button
					{...props}
					variant="ghost"
					size="icon"
					class={cn(
						'relative size-7',
						on
							? 'text-foreground hover:text-foreground'
							: 'text-muted-foreground/50 hover:text-muted-foreground',
					)}
					aria-pressed={on}
					data-testid={testid}
					onclick={() => toggleSetting(key)}
				>
					<Icon />
					{#if on}
						<span class="absolute inset-x-1.5 bottom-0.5 h-px bg-current"></span>
					{/if}
				</Button>
			{/snippet}
		</Tooltip.Trigger>
		<Tooltip.Content>{label}: {on ? 'on' : 'off'}</Tooltip.Content>
	</Tooltip.Root>
{/snippet}

<div
	class="flex h-10 items-center gap-2 border-b border-border/50 bg-muted/30 px-3"
	data-testid="status-bar"
>
	<!-- A watcher has no map list, no saved arrangement and no settings: the chain is the
	     whole page for them. -->
	{#if !watching}
		<a
			href="/maps"
			class="flex items-center gap-1.5 text-sm text-muted-foreground transition-colors hover:text-foreground"
		>
			<ArrowLeftIcon class="size-4" />
			Maps
		</a>
		<Separator orientation="vertical" class="h-4" />
	{/if}
	<span class="truncate text-sm font-medium" data-testid="status-bar-name">
		{map.data?.map.name ?? '...'}
	</span>
	<Button
		variant="outline"
		size="sm"
		class="h-7 gap-2 text-muted-foreground"
		data-testid="palette-trigger"
		onclick={() => (map.paletteOpen = true)}
	>
		<SearchIcon />
		Search
		<kbd class="rounded border border-border/60 px-1 font-mono text-[10px]">⌘K</kbd>
	</Button>

	<div class="flex flex-1 items-center justify-center gap-2">
		{#if watching}
			<Tooltip.Root>
				<Tooltip.Trigger>
					{#snippet child({ props })}
						<Badge {...props} variant="outline" class="gap-1 text-muted-foreground">
							<EyeIcon />
							Watching
						</Badge>
					{/snippet}
				</Tooltip.Trigger>
				<Tooltip.Content class="max-w-64">
					You are following a link to this map. It updates as the chain is scanned, but nothing here
					can be changed without an account that has access to it.
				</Tooltip.Content>
			</Tooltip.Root>
		{:else if map.data && !map.data.character_has_access}
			<Tooltip.Root>
				<Tooltip.Trigger>
					{#snippet child({ props })}
						<Badge {...props} variant="outline" class="gap-1 border-amber-600/40 text-amber-500">
							<TriangleAlertIcon />
							Limited access
						</Badge>
					{/snippet}
				</Tooltip.Trigger>
				<Tooltip.Content class="max-w-64">
					You can see this map through another of your characters, but {pilot?.name ??
						'the active character'} has no access of its own. Location sharing and waypoints will not
					work for them.
				</Tooltip.Content>
			</Tooltip.Root>
		{/if}

		{#if map.systems.orphaned.length > 0 && canWrite}
			<!-- What a collapsed hole leaves behind: a branch nothing reaches any more. -->
			<Badge
				variant="outline"
				class="cursor-pointer gap-1 border-amber-500/40 text-amber-500"
				data-testid="orphaned-badge"
				onclick={() => (map.cleanPrompt = true)}
			>
				<BrushCleaningIcon />
				{map.systems.orphaned.length} adrift
			</Badge>
		{/if}

		<StaleConnectionsPopover {map} />
	</div>

	{#if pilot?.online && pilot.solar_system_id !== null}
		<span class="hidden items-center gap-1.5 text-xs text-muted-foreground lg:flex">
			{#if pilotSystem?.kind === 'system'}
				<ClassBadge
					classId={pilotSystem.wormhole_class_id}
					security={pilotSystem.security_status}
				/>
				<span class="text-foreground">{pilotSystem.name}</span>
			{:else}
				<span>Outside the chain</span>
			{/if}
		</span>
	{/if}

	<Tooltip.Root>
		<Tooltip.Trigger>
			{#snippet child({ props })}
				<span
					{...props}
					data-testid="socket-dot"
					data-state={map.socket}
					class={cn(
						'size-2 rounded-full',
						map.socket === 'open' && 'bg-emerald-500',
						map.socket === 'connecting' && 'bg-amber-500',
						map.socket === 'reconnecting' && 'animate-pulse bg-red-500',
						map.socket === 'revoked' && 'bg-red-500',
					)}
				></span>
			{/snippet}
		</Tooltip.Trigger>
		<Tooltip.Content>{socketLabel[map.socket]}</Tooltip.Content>
	</Tooltip.Root>

	<Separator orientation="vertical" class="h-4" />

	<Tooltip.Root>
		<Tooltip.Trigger>
			{#snippet child({ props })}
				<Button
					{...props}
					variant="ghost"
					size="icon"
					class="size-7"
					data-testid="tour-button"
					onclick={() => onstarttour?.()}
				>
					<CircleHelpIcon />
				</Button>
			{/snippet}
		</Tooltip.Trigger>
		<Tooltip.Content>Show me around</Tooltip.Content>
	</Tooltip.Root>

	<Separator orientation="vertical" class="h-4" />

	{#if map.userSettings}
		{@render toggle(
			'Share location',
			map.userSettings.tracking_allowed,
			EyeIcon,
			'tracking_allowed',
			'tracking-toggle',
		)}
		<TrackingSettings {map} />
		{@render toggle(
			'Threat rings',
			map.userSettings.show_threat_level,
			RadarIcon,
			'show_threat_level',
			'threat-toggle',
		)}
	{/if}

	{#if canWrite}
		<Separator orientation="vertical" class="h-4" />
	{/if}
	<span class="flex items-center gap-2" data-testid="history-controls">
		{#if canWrite}
			<Tooltip.Root>
				<Tooltip.Trigger>
					{#snippet child({ props })}
						<Button
							{...props}
							variant="ghost"
							size="icon"
							class="size-7"
							data-testid="undo-button"
							disabled={!map.history.canUndo}
							onclick={() => map.history.undo()}
						>
							<Undo2Icon />
						</Button>
					{/snippet}
				</Tooltip.Trigger>
				<Tooltip.Content>
					{map.history.headEntry ? `Undo: ${map.history.headEntry.label}` : 'Nothing to undo'}
				</Tooltip.Content>
			</Tooltip.Root>
			<Tooltip.Root>
				<Tooltip.Trigger>
					{#snippet child({ props })}
						<Button
							{...props}
							variant="ghost"
							size="icon"
							class="size-7"
							data-testid="redo-button"
							disabled={!map.history.canRedo}
							onclick={() => map.history.redo()}
						>
							<Redo2Icon />
						</Button>
					{/snippet}
				</Tooltip.Trigger>
				<Tooltip.Content>
					{map.history.redoEntry ? `Redo: ${map.history.redoEntry.label}` : 'Nothing to redo'}
				</Tooltip.Content>
			</Tooltip.Root>
		{/if}
		{#if !watching}
			<HistoryPopover {map} />
		{/if}
	</span>

	{#if !watching}
		<Separator orientation="vertical" class="h-4" />
		<Tooltip.Root>
			<Tooltip.Trigger>
				{#snippet child({ props })}
					<Button
						{...props}
						variant="ghost"
						size="icon"
						class={cn('size-7', map.panels.editing && 'bg-accent text-foreground')}
						aria-pressed={map.panels.editing}
						data-testid="layout-toggle"
						onclick={() =>
							map.panels.editing ? map.panels.exitEdit() : (map.panels.editing = true)}
					>
						<LayoutGridIcon />
					</Button>
				{/snippet}
			</Tooltip.Trigger>
			<Tooltip.Content>
				{map.panels.editing ? 'Done arranging panels' : 'Arrange the side panels'}
			</Tooltip.Content>
		</Tooltip.Root>

		<Tooltip.Root>
			<Tooltip.Trigger>
				{#snippet child({ props })}
					<Button
						{...props}
						href="/maps/{map.mapId}/settings"
						variant="ghost"
						size="icon"
						class="size-7"
						data-testid="settings-link"
					>
						<SettingsIcon />
					</Button>
				{/snippet}
			</Tooltip.Trigger>
			<Tooltip.Content>Map settings</Tooltip.Content>
		</Tooltip.Root>
	{/if}
</div>
