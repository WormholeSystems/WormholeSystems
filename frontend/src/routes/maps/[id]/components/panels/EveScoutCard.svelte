<script lang="ts">
	// The public wormholes out of Thera and Turnur, as EVE Scout's scouts have them. One hub
	// at a time: a Thera hole is not an alternative route to a Turnur one.
	// The jump count is measured through your own chain, which is the reason to look, so rows
	// sort by it by default.
	import ExternalLinkIcon from '@lucide/svelte/icons/external-link';
	import PlusIcon from '@lucide/svelte/icons/plus';

	import type { SystemSearchResult } from '$lib/api/types/SystemSearchResult';
	import { ticking } from '$lib/now.svelte';
	import { systemResolver } from '$lib/resolve-cache.svelte';
	import { sortedBy, sortState } from '$lib/sort-state.svelte';
	import ClassBadge from '$lib/components/ClassBadge.svelte';
	import EveImage from '$lib/components/EveImage.svelte';
	import MapPanel from '$lib/components/map-panel/MapPanel.svelte';
	import MapPanelContent from '$lib/components/map-panel/MapPanelContent.svelte';
	import MapPanelHeader from '$lib/components/map-panel/MapPanelHeader.svelte';
	import SortHeader from '$lib/components/SortHeader.svelte';
	import RouteOriginBadge from './RouteOriginBadge.svelte';
	import SystemMenu from '../system-menu/SystemMenu.svelte';
	import { Button } from '$lib/components/ui/button';
	import * as Tabs from '$lib/components/ui/tabs';
	import * as Tooltip from '$lib/components/ui/tooltip';
	import { timeAgo } from '$lib/format';
	import { jumpTone } from '$lib/routing/algorithm';
	import { cn } from '$lib/utils';
	import { clearHover, hoverSystem } from '../../state/map-hover';
	import {
		EVESCOUT_COLUMNS,
		EVESCOUT_COMPARATORS,
		buildEveScoutRows,
		eveScoutTiebreak,
		latestUpdate,
		ttl,
		ttlTone,
		type EveScoutColumn,
		type EveScoutRow,
	} from './evescout-rows';
	import { routeBatch } from '../../state/route-batch.svelte';
	import type { MapState } from '../../state/map-state.svelte';
	import RoutePopover from './RoutePopover.svelte';
	import Hint from '$lib/components/Hint.svelte';

	let { map }: { map: MapState } = $props();

	type Hub = 'Thera' | 'Turnur';

	const HUBS: Hub[] = ['Thera', 'Turnur'];

	const connections = $derived(map.eveScout);
	let hub = $state<Hub>('Thera');
	const sort = sortState('evescout-sort', EVESCOUT_COLUMNS, { column: 'jumps', direction: 'asc' });
	const clock = ticking(60_000);
	const now = $derived(clock.current);

	// The hubs are resolved along with the far sides, so a row whose destination is the other
	// hub still renders.
	const systems = systemResolver;
	$effect(() => {
		systems.ensure(connections.map((c) => c.solar_system_id));
	});

	const hubRows = $derived(connections.filter((c) => c.hub === hub));
	const hubSolarSystemId = $derived(hubRows[0]?.hub_solar_system_id ?? null);
	// One search from the origin covers every row. The hub itself is excluded as a stepping
	// stone: a route to a Thera hole that goes through Thera measures the wrong thing.
	// svelte-ignore state_referenced_locally -- the map instance is stable for this mount.
	const batch = routeBatch(map, () => hubRows.map((c) => c.solar_system_id), {
		extraIgnored: () => hubRows.map((c) => c.hub_solar_system_id),
	});
	const routes = $derived(batch.routes);

	const counts = $derived({
		Thera: connections.filter((c) => c.hub === 'Thera').length,
		Turnur: connections.filter((c) => c.hub === 'Turnur').length,
	});

	/** The most recent scout report in the active hub's list. */
	const updated = $derived.by(() => {
		const stamp = latestUpdate(hubRows);
		return stamp === null ? null : timeAgo(stamp, now);
	});

	const sorted = $derived(
		sortedBy(
			buildEveScoutRows(hubRows, (id) => systems.get(id), routes),
			sort.current,
			EVESCOUT_COMPARATORS,
			eveScoutTiebreak,
		),
	);

	function hover(row: EveScoutRow, on: boolean) {
		if (on) hoverSystem(map, row.connection.solar_system_id, row.route);
		else clearHover(map);
	}

	// EVE serves a faction's logo from the corporations endpoint keyed by the faction id,
	// so anything that is not an alliance uses the corporation one.
	function sovKind(sov: NonNullable<SystemSearchResult['sovereignty']>) {
		return sov.kind === 'alliance' ? 'alliance' : 'corporation';
	}
</script>

{#snippet heading(key: EveScoutColumn, label: string, extra = '')}
	<SortHeader column={key} sort={sort.current} onsort={sort.toggle} class={extra}>
		<span>{label}</span>
	</SortHeader>
{/snippet}

<MapPanel testid="evescout-card">
	<MapPanelHeader>
		<span class="inline-flex items-center gap-2">
			EVE Scout
			{#if updated}
				<span class="font-mono text-[10px] text-muted-foreground/60" data-testid="evescout-updated">
					{updated}
				</span>
			{/if}
		</span>
		{#snippet actions()}
			<RouteOriginBadge {map} />
			<Tabs.Root
				value={hub}
				onValueChange={(v) => v && (hub = v as Hub)}
				class="w-fit"
				data-testid="evescout-hubs"
			>
				<Tabs.List variant="line" class="h-6">
					{#each HUBS as name (name)}
						<Tabs.Trigger
							value={name}
							class="px-1 text-[10px]"
							data-testid="evescout-hub-{name.toLowerCase()}"
						>
							{name}
							{#if counts[name] > 0}
								<span class="font-mono text-amber-400">{counts[name]}</span>
							{/if}
						</Tabs.Trigger>
					{/each}
				</Tabs.List>
			</Tabs.Root>
			{#if map.canWrite && hubSolarSystemId !== null}
				<Tooltip.Root>
					<Tooltip.Trigger>
						{#snippet child({ props })}
							<Button
								{...props}
								variant="ghost"
								size="icon"
								class="size-6"
								aria-label="Add all {hub} holes to the map"
								data-testid="evescout-add"
								onclick={() => map.systems.addEveScout(hubSolarSystemId)}
							>
								<PlusIcon />
							</Button>
						{/snippet}
					</Tooltip.Trigger>
					<Tooltip.Content>Add all {hub} holes to the map</Tooltip.Content>
				</Tooltip.Root>
			{/if}
			<Tooltip.Root>
				<Tooltip.Trigger>
					{#snippet child({ props })}
						<Button
							{...props}
							variant="ghost"
							size="icon"
							class="size-6"
							href="https://www.eve-scout.com/"
							target="_blank"
							rel="noopener noreferrer"
							aria-label="Open EVE Scout"
						>
							<ExternalLinkIcon />
						</Button>
					{/snippet}
				</Tooltip.Trigger>
				<Tooltip.Content>Open eve-scout.com</Tooltip.Content>
			</Tooltip.Root>
		{/snippet}
	</MapPanelHeader>

	<MapPanelContent>
		<div class="mt-1 flex flex-col">
			<div
				class="flex items-center gap-2 px-3 pb-1 text-[10px] tracking-wider text-muted-foreground uppercase"
			>
				{@render heading('system', 'System', 'min-w-0 flex-1')}
				{@render heading('region', 'Region', 'min-w-0 flex-1')}
				<span class="w-4 shrink-0"></span>
				{@render heading('signature', 'Sig', 'w-14 shrink-0')}
				{@render heading('type', 'WH', 'w-10 shrink-0')}
				{@render heading('jumps', 'J', 'w-10 shrink-0 justify-end')}
				{@render heading('ttl', 'TTL', 'w-10 shrink-0 justify-end')}
			</div>

			{#if sorted.length === 0}
				<p class="px-3 py-4 text-xs text-muted-foreground" data-testid="evescout-empty">
					{connections.length === 0
						? 'No public connections reported right now.'
						: `Nothing out of ${hub} right now.`}
				</p>
			{:else}
				{#each sorted as row (row.connection.hub_signature + row.connection.signature)}
					{@const c = row.connection}
					{#snippet line()}
						<!-- svelte-ignore a11y_no_static_element_interactions -->
						<div
							class="flex items-center gap-2 border-b border-border/30 px-3 py-1 text-xs last:border-b-0 hover:bg-muted/30"
							data-testid="evescout-row"
							data-system={row.system?.name ?? c.solar_system_id}
							onmouseenter={() => hover(row, true)}
							onmouseleave={() => hover(row, false)}
						>
							<span class="flex min-w-0 flex-1 items-center gap-1.5">
								<ClassBadge
									classId={row.system?.wormhole_class_id ?? null}
									security={row.system?.security ?? 0}
									class="shrink-0 text-[10px]"
								/>
								<Hint text={row.system?.name} clippedOnly focusable={false}>
									{#snippet child({ props })}
										<span class="truncate" {...props}>
											{row.system?.name ?? '…'}
										</span>
									{/snippet}
								</Hint>
							</span>

							<Hint text={row.system?.region} clippedOnly focusable={false}>
								{#snippet child({ props })}
									<span
										class="min-w-0 flex-1 truncate font-mono text-[10px] text-muted-foreground"
										{...props}
									>
										{row.system?.region ?? ''}
									</span>
								{/snippet}
							</Hint>

							<span class="flex w-4 shrink-0 justify-center">
								{#if row.system?.sovereignty}
									{@const sov = row.system.sovereignty}
									<Tooltip.Root>
										<Tooltip.Trigger class="flex">
											<EveImage
												kind={sovKind(sov)}
												id={sov.id}
												class="size-4 shrink-0 rounded-sm"
											/>
										</Tooltip.Trigger>
										<Tooltip.Content class="flex items-center gap-2">
											<EveImage kind={sovKind(sov)} id={sov.id} class="size-6 rounded-sm" />
											{sov.name}
											{#if 'ticker' in sov}({sov.ticker}){/if}
										</Tooltip.Content>
									</Tooltip.Root>
								{:else if row.system?.effect_name}
									{@const effect = row.system.effect_name}
									<Hint text={effect} focusable={false} label>
										{#snippet child({ props })}
											<span class="font-mono text-[9px] text-muted-foreground/70" {...props}>
												{effect.slice(0, 1)}
											</span>
										{/snippet}
									</Hint>
								{/if}
							</span>

							<!-- The hub-side signature is what you warp to; the far side's own id is on
								     the tooltip, for the scan on arrival. -->
							<Tooltip.Root>
								<Tooltip.Trigger
									class="w-14 shrink-0 cursor-help text-left font-mono text-[10px] text-muted-foreground"
									data-testid="evescout-signature"
								>
									{c.hub_signature || '---'}
								</Tooltip.Trigger>
								<Tooltip.Content class="flex flex-col gap-0.5 text-xs">
									<span>{c.hub_signature || '?'} in {c.hub}</span>
									<span class="text-muted-foreground">
										{c.signature || '?'} on the far side
									</span>
									{#if c.max_ship_size}
										<span class="text-muted-foreground">Up to {c.max_ship_size}</span>
									{/if}
								</Tooltip.Content>
							</Tooltip.Root>

							<span class="w-10 shrink-0 font-mono text-[10px] text-muted-foreground">
								{c.wormhole_type ?? '---'}
							</span>

							<span class="w-10 shrink-0 text-right">
								{#if row.route}
									<RoutePopover {map} steps={row.route.route}>
										<span
											class={cn(
												'cursor-pointer font-medium tabular-nums',
												jumpTone(row.jumps ?? 0),
											)}
											data-testid="evescout-jumps">{row.jumps}j</span
										>
									</RoutePopover>
								{:else}
									<span class="text-[10px] whitespace-nowrap text-muted-foreground/60">--</span>
								{/if}
							</span>

							<span
								class={cn(
									'w-10 shrink-0 text-right font-mono text-[10px] font-semibold tabular-nums',
									ttlTone(c.remaining_hours),
								)}
								data-testid="evescout-ttl"
							>
								{ttl(c.remaining_hours)}
							</span>
						</div>
					{/snippet}

					{#if row.system}
						<SystemMenu system={row.system}>{@render line()}</SystemMenu>
					{:else}
						{@render line()}
					{/if}
				{/each}
			{/if}
		</div>
	</MapPanelContent>
</MapPanel>
