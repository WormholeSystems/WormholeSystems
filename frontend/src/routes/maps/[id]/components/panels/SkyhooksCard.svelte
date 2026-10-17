<script lang="ts">
	// Raidable skyhooks: which ones are open, which are about to be, and how far away.
	// The whole list is shown rather than a slice near you, because a two-hour window is long
	// enough to fly a long way for. Sorted by distance by default.
	import { toSearchResult } from '$lib/map/system';

	import { createQuery } from '@tanstack/svelte-query';

	import { q } from '$lib/api/queries';
	import { ticking } from '$lib/now.svelte';
	import { sortedBy, sortState } from '$lib/sort-state.svelte';
	import type { PlanetKind } from '$lib/api/types/PlanetKind';
	import type { Skyhook } from '$lib/api/types/Skyhook';
	import type { SystemSearchResult } from '$lib/api/types/SystemSearchResult';
	import ClassBadge from '$lib/components/ClassBadge.svelte';
	import MapPanel from '$lib/components/map-panel/MapPanel.svelte';
	import MapPanelContent from '$lib/components/map-panel/MapPanelContent.svelte';
	import MapPanelHeader from '$lib/components/map-panel/MapPanelHeader.svelte';
	import SortHeader from '$lib/components/SortHeader.svelte';
	import SovereigntyBadge from '$lib/components/SovereigntyBadge.svelte';
	import RouteOriginBadge from './RouteOriginBadge.svelte';
	import * as Tabs from '$lib/components/ui/tabs';
	import { Button } from '$lib/components/ui/button';
	import { Separator } from '$lib/components/ui/separator';
	import * as Tooltip from '$lib/components/ui/tooltip';
	import { jumpTone } from '$lib/routing/algorithm';
	import {
		describe,
		formatDuration,
		formatWindow,
		statusDot,
		statusText,
		type SkyhookStatus,
	} from '$lib/skyhooks/timer';
	import { cn } from '$lib/utils';
	import { clearHover, hoverSystem } from '../../state/map-hover';
	import {
		buildSkyhookRows,
		liveSkyhookRows,
		skyhookCounts,
		skyhookTiebreak,
		SKYHOOK_COLUMNS,
		SKYHOOK_COMPARATORS,
		type SkyhookColumn,
		type SkyhookRow,
	} from './skyhook-rows';
	import { routeBatch } from '../../state/route-batch.svelte';
	import type { MapState } from '../../state/map-state.svelte';
	import SystemMenu from '../system-menu/SystemMenu.svelte';
	import RoutePopover from './RoutePopover.svelte';
	import Hint from '$lib/components/Hint.svelte';

	let { map }: { map: MapState } = $props();

	// The server mirrors ESI every five minutes, so the query's own interval asks no more
	// often than that.
	const skyhooksQuery = createQuery(() => q.skyhooks());
	const skyhooks = $derived(skyhooksQuery.data ?? []);
	// Timers are read to the minute, so half a minute keeps them honest without churn.
	const clock = ticking(30_000);
	const now = $derived(clock.current);
	// Skyhooks only go on reagent planets, so lava and ice is the whole vocabulary. Shown one
	// kind at a time, because the reagent is what the trip was for.
	let kind = $state<PlanetKind>('lava');
	// Closed skyhooks are never listed: the window is over, so there is nothing to go and do.
	let shown = $state<string[]>(['upcoming', 'open', 'closing']);
	const sort = sortState(null, SKYHOOK_COLUMNS, { column: 'jumps', direction: 'asc' });

	// One search from the origin covers every skyhook, rather than one per row.
	// svelte-ignore state_referenced_locally -- the map instance is stable for this mount.
	const batch = routeBatch(map, () => skyhooks.map((s) => s.solar_system_id));

	const rows = $derived(buildSkyhookRows(skyhooks, now, batch.routes));
	const live = $derived(liveSkyhookRows(rows, shown));
	const counts = $derived(skyhookCounts(live));

	/**
	 * "Other" should stay empty forever (skyhooks only go on reagent planets), but it is still
	 * counted so one that cannot be displayed shows up instead of vanishing.
	 */
	const choices = $derived([
		{ key: 'lava' as const, label: 'Lava' },
		{ key: 'ice' as const, label: 'Ice' },
		...(counts.other > 0 ? [{ key: 'other' as const, label: 'Other' }] : []),
	]);

	const sorted = $derived(
		sortedBy(
			live.filter((r) => r.skyhook.planet_kind === kind),
			sort.current,
			SKYHOOK_COMPARATORS,
			skyhookTiebreak,
		),
	);

	/**
	 * Built from the payload rather than resolved: a skyhook carries everything the menu
	 * needs, so no row waits on a second request before it can be right-clicked.
	 */
	function systemOf(skyhook: Skyhook): SystemSearchResult {
		return toSearchResult({
			id: skyhook.solar_system_id,
			name: skyhook.system_name,
			security: skyhook.security_status,
			region: skyhook.region,
			region_id: skyhook.region_id,
			constellation_id: skyhook.constellation_id,
			sovereignty: skyhook.sovereignty ?? null,
		});
	}

	function hover(row: SkyhookRow, on: boolean) {
		if (on) hoverSystem(map, row.skyhook.solar_system_id, row.route);
		else clearHover(map);
	}

	const FILTERS: { key: SkyhookStatus; label: string }[] = [
		{ key: 'upcoming', label: 'Upcoming' },
		{ key: 'open', label: 'Raidable now' },
		{ key: 'closing', label: 'Closing within 15m' },
	];
</script>

{#snippet heading(key: SkyhookColumn, label: string, extra = '')}
	<SortHeader column={key} sort={sort.current} onsort={sort.toggle} class={extra}>
		<span>{label}</span>
	</SortHeader>
{/snippet}

<MapPanel testid="skyhooks-card">
	<MapPanelHeader>
		<span class="inline-flex items-center gap-2">
			Raidable Skyhooks
			<span class="font-mono text-amber-400">{live.length}</span>
		</span>
		{#snippet actions()}
			<RouteOriginBadge {map} />
			<!-- Tabs rather than a toggle group: a toggle's "on" background is the same colour
				     as its hover, so the selected one was only distinguishable by accident. -->
			<Tabs.Root
				value={kind}
				onValueChange={(v) => v && (kind = v as PlanetKind)}
				class="w-fit"
				data-testid="skyhook-kinds"
			>
				<Tabs.List variant="line" class="h-6">
					{#each choices as choice (choice.key)}
						<Tabs.Trigger
							value={choice.key}
							class="px-1 text-[10px]"
							data-testid="skyhook-kind-{choice.key}"
						>
							{choice.label}
							{#if counts[choice.key] > 0}
								<span class="font-mono text-amber-400">{counts[choice.key]}</span>
							{/if}
						</Tabs.Trigger>
					{/each}
				</Tabs.List>
			</Tabs.Root>
			<Separator orientation="vertical" class="mx-0.5 data-[orientation=vertical]:h-4" />
			<div class="flex items-center gap-0.5" data-testid="skyhook-filters">
				{#each FILTERS as filter (filter.key)}
					{@const on = shown.includes(filter.key)}
					<Tooltip.Root>
						<Tooltip.Trigger>
							{#snippet child({ props })}
								<Button
									{...props}
									variant="ghost"
									size="icon"
									class="size-6"
									aria-pressed={on}
									aria-label={filter.label}
									data-testid="skyhook-filter-{filter.key}"
									onclick={() => {
										shown = on ? shown.filter((k) => k !== filter.key) : [...shown, filter.key];
									}}
								>
									<span
										class={cn(
											'inline-block size-2 rounded-full',
											on ? statusDot(filter.key) : 'bg-muted-foreground/30',
										)}
									></span>
								</Button>
							{/snippet}
						</Tooltip.Trigger>
						<Tooltip.Content>{on ? `Hide ${filter.label}` : `Show ${filter.label}`}</Tooltip.Content
						>
					</Tooltip.Root>
				{/each}
			</div>
		{/snippet}
	</MapPanelHeader>

	<MapPanelContent>
		<div class="mt-1 flex flex-col">
			<div
				class="flex items-center gap-2 px-3 pb-1 text-[10px] tracking-wider text-muted-foreground uppercase"
			>
				<span class="w-2 shrink-0"></span>
				{@render heading('planet', 'Planet', 'min-w-0 flex-1')}
				{@render heading('region', 'Region', 'min-w-0 flex-1')}
				<span class="w-4 shrink-0"></span>
				{@render heading('jumps', 'J', 'w-10 shrink-0 justify-end')}
				{@render heading('timer', 'Timer', 'w-16 shrink-0 justify-end')}
			</div>

			{#if sorted.length === 0}
				<p class="px-3 py-4 text-xs text-muted-foreground" data-testid="skyhooks-empty">
					{skyhooks.length === 0
						? 'No raidable skyhooks right now.'
						: 'Nothing matches the current filters.'}
				</p>
			{:else}
				{#each sorted as row (row.skyhook.planet_id)}
					{#snippet line()}
						<!-- svelte-ignore a11y_no_static_element_interactions -->
						<div
							class="flex items-center gap-2 border-b border-border/30 px-3 py-1 text-xs last:border-b-0 hover:bg-muted/30"
							data-testid="skyhook-row"
							data-planet={row.skyhook.planet_name}
							data-status={row.status}
							onmouseenter={() => hover(row, true)}
							onmouseleave={() => hover(row, false)}
						>
							<span
								class={cn('size-2 shrink-0 rounded-full', statusDot(row.status))}
								aria-label={row.status}
							></span>

							<span class="flex min-w-0 flex-1 items-center gap-1.5">
								<ClassBadge
									classId={null}
									security={row.skyhook.security_status}
									class="shrink-0 text-[10px]"
								/>
								<Hint text={row.skyhook.planet_name} clippedOnly focusable={false}>
									{#snippet child({ props })}
										<span class="truncate" {...props}>{row.skyhook.planet_name}</span>
									{/snippet}
								</Hint>
							</span>

							<Hint text={row.skyhook.region} clippedOnly focusable={false}>
								{#snippet child({ props })}
									<span
										class="min-w-0 flex-1 truncate font-mono text-[10px] text-muted-foreground"
										{...props}>{row.skyhook.region}</span
									>
								{/snippet}
							</Hint>

							<span class="flex w-4 shrink-0 justify-center">
								{#if row.skyhook.sovereignty}
									<SovereigntyBadge sovereignty={row.skyhook.sovereignty} />
								{/if}
							</span>

							<span class="w-10 shrink-0 text-right">
								{#if row.route}
									<RoutePopover {map} steps={row.route.route}>
										<span
											class={cn(
												'cursor-pointer font-medium tabular-nums',
												jumpTone(row.jumps ?? 0),
											)}
											data-testid="skyhook-jumps">{row.jumps}j</span
										>
									</RoutePopover>
								{:else}
									<span class="text-[10px] whitespace-nowrap text-muted-foreground/60">--</span>
								{/if}
							</span>

							<Tooltip.Root>
								<Tooltip.Trigger
									class={cn(
										'w-16 shrink-0 cursor-help text-right font-mono text-[10px] font-semibold tabular-nums',
										statusText(row.status),
									)}
									data-testid="skyhook-timer"
								>
									{formatDuration(row.untilMs)}
								</Tooltip.Trigger>
								<Tooltip.Content class="flex flex-col gap-0.5">
									<span>{describe({ status: row.status, untilMs: row.untilMs })}</span>
									<span class="font-mono text-[10px] text-muted-foreground">
										{formatWindow(row.skyhook)}
									</span>
								</Tooltip.Content>
							</Tooltip.Root>
						</div>
					{/snippet}

					<SystemMenu system={systemOf(row.skyhook)}>{@render line()}</SystemMenu>
				{/each}
			{/if}
		</div>
	</MapPanelContent>
</MapPanel>
