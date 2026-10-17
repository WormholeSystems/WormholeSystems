<script lang="ts">
	// What has died in the chain lately: whether anything is hunting here, and whether it was
	// worth anything.
	import FilterIcon from '@lucide/svelte/icons/list-filter';
	import { solarSystemId, toSearchResult } from '$lib/map/system';
	import type { KillmailScope } from '$lib/api/types/KillmailScope';

	import { createQuery } from '@tanstack/svelte-query';

	import { q } from '$lib/api/queries';
	import { KILLMAIL_FILTERS } from '$lib/map/killmails';
	import { ticking } from '$lib/now.svelte';
	import type { MapKillmail } from '$lib/api/types/MapKillmail';
	import type { SystemSearchResult } from '$lib/api/types/SystemSearchResult';
	import ClassBadge from '$lib/components/ClassBadge.svelte';
	import EveImage from '$lib/components/EveImage.svelte';
	import MapPanel from '$lib/components/map-panel/MapPanel.svelte';
	import MapPanelContent from '$lib/components/map-panel/MapPanelContent.svelte';
	import MapPanelHeader from '$lib/components/map-panel/MapPanelHeader.svelte';
	import SystemMenu from '../system-menu/SystemMenu.svelte';
	import { Button } from '$lib/components/ui/button';
	import * as DropdownMenu from '$lib/components/ui/dropdown-menu';
	import * as Tooltip from '$lib/components/ui/tooltip';
	import { formatIsk, iskSeverity, timeAgo } from '$lib/format';
	import { cn } from '$lib/utils';
	import { clearHover, hoverSystem } from '../../state/map-hover';
	import {
		ISK_TONE,
		crowdLabel,
		crowdTone,
		partyName,
		partyOrg,
		systemKey,
	} from './killmail-presentation';
	import type { MapState } from '../../state/map-state.svelte';
	import Hint from '$lib/components/Hint.svelte';

	let { map }: { map: MapState } = $props();

	// The list itself arrives by push; this is only the clock.
	const clock = ticking(30_000);
	const now = $derived(clock.current);

	const filter = $derived(map.userSettings?.killmail_filter ?? 'all');
	// Only a card that follows the selection cares what is selected. Any other filter
	// leaves it out of the key, so clicking around the map doesn't refetch the list.
	const followsSelection = $derived(filter === 'selected_system');
	const selected = $derived(
		followsSelection && map.activeSystem ? solarSystemId(map.activeSystem) : null,
	);
	const awaitingSelection = $derived(followsSelection && selected === null);

	// Adding a system is as much a change as a fresh kill arriving, so the systems are part
	// of the key; a live kill refetches through the same filter rather than landing as is.
	const killsQuery = createQuery(() => ({
		...q.mapKillmails(map.mapId, filter, selected, systemKey(map.systems.all)),
		// With nothing selected there is nothing to ask for.
		enabled: !awaitingSelection,
	}));
	const kills = $derived(killsQuery.data ?? []);

	function setFilter(value: KillmailScope) {
		map.patchUserSettings({ killmail_filter: value });
	}

	/** The row's system in the shape the context menu wants, from the payload. */
	function systemOf(kill: MapKillmail): SystemSearchResult {
		return toSearchResult({
			id: kill.solar_system_id,
			name: kill.system_name,
			security: kill.security_status,
			region: kill.region,
			wormhole_class_id: kill.wormhole_class_id ?? null,
		});
	}

	/** The map's own name for the system, when it has one. */
	function aliasOf(kill: MapKillmail): string | null {
		return map.systems.all.find((s) => solarSystemId(s) === kill.solar_system_id)?.alias ?? null;
	}

	function hover(kill: MapKillmail, on: boolean) {
		if (on) hoverSystem(map, kill.solar_system_id);
		else clearHover(map);
	}

	const zkill = (id: number) => `https://zkillboard.com/kill/${id}/`;
</script>

<MapPanel testid="killmails-card">
	<MapPanelHeader>
		<span class="inline-flex items-center gap-2">
			Killmails
			{#if kills.length > 0}
				<span class="font-mono text-amber-400">{kills.length}</span>
			{/if}
		</span>
		{#snippet actions()}
			<DropdownMenu.Root>
				<DropdownMenu.Trigger>
					{#snippet child({ props })}
						<Button
							{...props}
							variant="ghost"
							size="icon"
							class="size-6"
							aria-label="Filter killmails"
							data-testid="killmail-filter"
						>
							<FilterIcon />
						</Button>
					{/snippet}
				</DropdownMenu.Trigger>
				<DropdownMenu.Content align="end">
					<DropdownMenu.Group>
						{#each KILLMAIL_FILTERS as option (option.value)}
							<DropdownMenu.CheckboxItem
								checked={filter === option.value}
								onCheckedChange={() => setFilter(option.value)}
								data-testid="killmail-filter-{option.value}"
							>
								{option.label}
							</DropdownMenu.CheckboxItem>
						{/each}
					</DropdownMenu.Group>
				</DropdownMenu.Content>
			</DropdownMenu.Root>
		{/snippet}
	</MapPanelHeader>

	<MapPanelContent>
		{#if awaitingSelection}
			<p class="px-3 py-4 text-xs text-muted-foreground" data-testid="killmails-select-hint">
				Select a system on the map to see its killmails.
			</p>
		{:else if kills.length === 0}
			<p class="px-3 py-4 text-xs text-muted-foreground" data-testid="killmails-empty">
				{followsSelection
					? 'Nothing has died in this system in the last week.'
					: 'Nothing has died in these systems in the last week.'}
			</p>
		{:else}
			<div class="@container flex flex-col">
				{#each kills as kill (kill.id)}
					{@const alias = aliasOf(kill)}
					{#snippet row()}
						<!-- svelte-ignore a11y_no_static_element_interactions -->
						<div
							class="flex items-center gap-2 border-b border-border/30 px-3 py-1 text-xs last:border-b-0 hover:bg-muted/30"
							data-testid="killmail-row"
							data-kill={kill.id}
							onmouseenter={() => hover(kill, true)}
							onmouseleave={() => hover(kill, false)}
						>
							<Tooltip.Root>
								<Tooltip.Trigger class="flex shrink-0 items-center gap-1">
									{#if kill.victim.ship_type_id}
										<EveImage kind="type" id={kill.victim.ship_type_id} class="size-5 rounded" />
									{/if}
									{#if kill.victim.character_id}
										<EveImage
											kind="character"
											id={kill.victim.character_id}
											class="size-5 rounded"
										/>
									{/if}
								</Tooltip.Trigger>
								<Tooltip.Content class="flex flex-col gap-0.5">
									<span>{partyName(kill.victim)}</span>
									<span class="text-muted-foreground">
										lost {kill.victim.ship_name ?? 'a ship'}
									</span>
									{#if partyOrg(kill.victim)}
										<span class="text-muted-foreground">{partyOrg(kill.victim)}</span>
									{/if}
								</Tooltip.Content>
							</Tooltip.Root>

							<Hint text={kill.victim.ship_name ?? ''} clippedOnly focusable={false}>
								{#snippet child({ props })}
									<span class="min-w-0 flex-1 truncate" {...props}>
										{kill.victim.ship_name ?? 'Unknown ship'}
										{#if kill.victim.alliance_ticker ?? kill.victim.corporation_ticker}
											<span class="font-mono text-[10px] text-muted-foreground">
												[{kill.victim.alliance_ticker ?? kill.victim.corporation_ticker}]
											</span>
										{/if}
									</span>
								{/snippet}
							</Hint>

							<span class="flex min-w-0 flex-1 items-center gap-1.5">
								<ClassBadge
									classId={kill.wormhole_class_id === null ? null : Number(kill.wormhole_class_id)}
									security={kill.security_status}
									class="shrink-0 text-[10px]"
								/>
								<Hint text={`${kill.system_name} · ${kill.region}`} clippedOnly focusable={false}>
									{#snippet child({ props })}
										<span class="min-w-0 truncate" {...props}>
											{#if alias}<span class="font-medium">{alias}</span> ·
											{/if}{kill.system_name}
										</span>
									{/snippet}
								</Hint>
							</span>

							<!-- Who did it. The icon and the count are min-content and left-aligned, so
								     every row lines up on the icon rather than on a number whose width
								     depends on how many attackers there were. -->
							<Tooltip.Root>
								<Tooltip.Trigger
									class="flex min-w-0 shrink-0 items-center gap-1.5 @min-[620px]:w-40 @min-[760px]:w-56"
								>
									{#if kill.final_blow.ship_type_id}
										<EveImage
											kind="type"
											id={kill.final_blow.ship_type_id}
											class="size-4 shrink-0 rounded"
										/>
									{/if}
									<span
										class={cn('shrink-0 font-mono text-[10px] tabular-nums', crowdTone(kill))}
										data-testid="killmail-attackers"
									>
										{kill.attacker_count}
									</span>
									<!-- Only where the card is wide enough to say it without crowding. -->
									<span
										class="hidden min-w-0 truncate text-left text-[10px] text-muted-foreground @min-[620px]:inline"
										data-testid="killmail-aggressor"
									>
										{kill.final_blow.character_name ?? (kill.is_npc ? 'NPCs' : 'Unknown')}
									</span>
									<span
										class="hidden min-w-0 truncate text-left font-mono text-[10px] text-muted-foreground/60 @min-[760px]:inline"
										data-testid="killmail-aggressor-org"
									>
										{kill.final_blow.alliance_ticker ?? kill.final_blow.corporation_ticker ?? ''}
									</span>
								</Tooltip.Trigger>
								<Tooltip.Content class="flex flex-col gap-0.5">
									<span>{crowdLabel(kill)}</span>
									{#if !kill.is_npc}
										<span class="text-muted-foreground">
											Final blow: {partyName(kill.final_blow)}
											{#if kill.final_blow.ship_name}({kill.final_blow.ship_name}){/if}
										</span>
										{#if partyOrg(kill.final_blow)}
											<span class="text-muted-foreground">{partyOrg(kill.final_blow)}</span>
										{/if}
									{/if}
								</Tooltip.Content>
							</Tooltip.Root>

							<Hint text={'Open on zKillboard'}>
								{#snippet child({ props })}
									<a
										class={cn(
											'hidden w-14 shrink-0 text-right font-mono text-[10px] tabular-nums hover:underline @min-[380px]:inline',
											ISK_TONE[iskSeverity(kill.total_value)],
										)}
										href={zkill(kill.id)}
										target="_blank"
										rel="noopener"
										data-testid="killmail-value"
										{...props}
									>
										{formatIsk(kill.total_value) ?? '--'}
									</a>
								{/snippet}
							</Hint>

							<span
								class="hidden w-14 shrink-0 text-right font-mono text-[10px] whitespace-nowrap text-muted-foreground/60 @min-[480px]:inline"
								data-testid="killmail-age"
							>
								{timeAgo(kill.time, now)}
							</span>
						</div>
					{/snippet}

					<SystemMenu system={systemOf(kill)}>{@render row()}</SystemMenu>
				{/each}
			</div>
		{/if}
	</MapPanelContent>
</MapPanel>
