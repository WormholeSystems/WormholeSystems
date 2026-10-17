<script lang="ts">
	// Closest-systems Find: pick a condition, get the nearest matches from the shared origin.
	// The Stations condition takes two optional filters, owner (a corporation or a whole
	// faction) and service, intersected at the station level; matching rows expand to the
	// concrete stations.
	import BuildingIcon from '@lucide/svelte/icons/building-2';
	import ChevronDownIcon from '@lucide/svelte/icons/chevron-down';
	import ChevronRightIcon from '@lucide/svelte/icons/chevron-right';
	import ChevronsUpDownIcon from '@lucide/svelte/icons/chevrons-up-down';

	import * as Command from '$lib/components/ui/command';
	import * as Popover from '$lib/components/ui/popover';
	import * as Select from '$lib/components/ui/select';
	import DestinationMenu from '../../system-menu/DestinationMenu.svelte';
	import SystemMenu from '../../system-menu/SystemMenu.svelte';
	import EveImage from '$lib/components/EveImage.svelte';
	import SystemRow from '../../pickers/SystemRow.svelte';
	import { SYSTEM_CELLS_4, SYSTEM_LIST_ACTIONS, SYSTEM_ROW } from '../../pickers/columns';
	import { jumpTone as badgeTone, type RouteGraph } from '$lib/routing/algorithm';
	import { findMatcher } from '$lib/routing/find-conditions';
	import type { MapState } from '../../../state/map-state.svelte';
	import {
		byFaction,
		factionOptions,
		matchesOwner,
		stationFilter,
		type OwnerPick,
	} from './find-stations';
	import RoutePopover from '../RoutePopover.svelte';
	import Hint from '$lib/components/Hint.svelte';

	let {
		map,
		graph,
		origin,
		active,
	}: {
		map: MapState;
		graph: RouteGraph | null;
		origin: number | null;
		/** Its tab is showing. It stays mounted behind the other tabs to keep its picks. */
		active: boolean;
	} = $props();

	const serviceOptions = $derived(map.route.serviceOptions);
	const corporationOptions = $derived(map.route.corporationOptions);

	const CONDITIONS = [
		{ value: 'observatories', label: 'Jove Observatories' },
		{ value: 'station', label: 'Stations' },
		{ value: 'highsec', label: 'High Security' },
		{ value: 'lowsec', label: 'Low Security' },
		{ value: 'nullsec', label: 'Null Security' },
	];
	let condition = $state('observatories');
	let findLimit = $state('15');
	const conditionLabel = $derived(
		CONDITIONS.find((c) => c.value === condition)?.label ?? 'Pick one',
	);

	// The optional station filters. Nothing picked means every NPC station counts.
	let owner = $state<OwnerPick>(null);
	let ownerOpen = $state(false);
	let ownerSearch = $state('');
	let serviceValue = $state('any');
	const serviceId = $derived(serviceValue === 'any' ? null : Number(serviceValue));

	const factions = $derived(factionOptions(corporationOptions));
	const sortedCorps = $derived(byFaction(corporationOptions));
	const ownerQuery = $derived(ownerSearch.trim().toLowerCase());
	const visibleFactions = $derived(
		ownerQuery ? factions.filter((f) => f.name.toLowerCase().includes(ownerQuery)) : factions,
	);
	const visibleCorps = $derived(
		ownerQuery ? sortedCorps.filter((corp) => matchesOwner(corp, ownerQuery)) : sortedCorps,
	);
	const ownerLabel = $derived.by(() => {
		const picked = owner;
		if (picked === null) return 'Any owner';
		if (picked.kind === 'faction') return factions.find((f) => f.id === picked.id)?.name ?? '…';
		return corporationOptions.find((c) => c.id === picked.id)?.name ?? '…';
	});
	const serviceLabel = $derived(
		serviceOptions.find((svc) => svc.id === serviceId)?.name ?? 'Any service',
	);
	function pickOwner(pick: OwnerPick) {
		owner = pick;
		ownerOpen = false;
		ownerSearch = '';
	}

	// What the picked filters agree on; null when nothing is picked, where every NPC
	// station matches and there is no station list to expand.
	const pickedStations = $derived(
		condition === 'station'
			? stationFilter(owner, serviceId, corporationOptions, serviceOptions)
			: null,
	);

	// Station lists collapse by default: a filter can match a dozen stations per system,
	// which would bury the jump-ordered results.
	let expandedFind = $state<Set<number>>(new Set());
	$effect(() => {
		void condition;
		void owner;
		void serviceValue;
		void origin;
		expandedFind = new Set();
	});
	function toggleFindRow(id: number) {
		const next = new Set(expandedFind);
		if (!next.delete(id)) next.add(id);
		expandedFind = next;
	}

	const findResults = $derived.by(() => {
		if (!active || !graph || origin === null) return [];
		const picked = pickedStations;
		const matches = picked
			? (id: number) => picked.systems.has(id)
			: findMatcher(condition, {
					jove: map.route.joveSystems,
					stations: map.route.stationSystems,
					security: map.route.security,
				});
		return map.route.search(origin)?.closest(matches, Number(findLimit)) ?? [];
	});
</script>

<div class="flex flex-col">
	<div class={SYSTEM_LIST_ACTIONS}>
		<div class="col-span-full flex items-center gap-1.5 border-b border-border/30 p-2">
			<Select.Root type="single" bind:value={condition}>
				<Select.Trigger class="h-7 min-w-0 flex-1 text-xs" data-testid="find-condition">
					<span class="truncate">{conditionLabel}</span>
				</Select.Trigger>
				<Select.Content>
					<Select.Group>
						<Select.GroupHeading>Features</Select.GroupHeading>
						{#each CONDITIONS.slice(0, 2) as c (c.value)}
							<Select.Item value={c.value} label={c.label}>{c.label}</Select.Item>
						{/each}
					</Select.Group>
					<Select.Group>
						<Select.GroupHeading>Security</Select.GroupHeading>
						{#each CONDITIONS.slice(2) as c (c.value)}
							<Select.Item value={c.value} label={c.label}>{c.label}</Select.Item>
						{/each}
					</Select.Group>
				</Select.Content>
			</Select.Root>
			<Select.Root type="single" bind:value={findLimit}>
				<Select.Trigger class="h-7 w-14 shrink-0 text-xs" data-testid="find-limit">
					{findLimit}
				</Select.Trigger>
				<Select.Content>
					<Select.Group>
						{#each ['5', '10', '15', '25', '50'] as n (n)}
							<Select.Item value={n} label={n}>{n}</Select.Item>
						{/each}
					</Select.Group>
				</Select.Content>
			</Select.Root>
		</div>
		{#if condition === 'station'}
			<div class="col-span-full flex items-center gap-1.5 border-b border-border/30 p-2">
				<Popover.Root bind:open={ownerOpen}>
					<Popover.Trigger
						class="flex h-7 min-w-0 flex-1 items-center justify-between gap-1.5 rounded-md border border-input bg-input/20 px-2 text-xs whitespace-nowrap transition-colors focus-visible:border-ring focus-visible:ring-2 focus-visible:ring-ring/30 dark:bg-input/30 dark:hover:bg-input/50 {owner
							? ''
							: 'text-muted-foreground'}"
						data-testid="find-owner"
					>
						<span class="truncate">{ownerLabel}</span>
						<ChevronsUpDownIcon class="size-3.5 shrink-0 opacity-50" />
					</Popover.Trigger>
					<Popover.Content class="w-80 p-0" align="start">
						<Command.Root shouldFilter={false}>
							<Command.Input
								placeholder="Search owners or factions…"
								bind:value={ownerSearch}
								data-testid="find-owner-search"
								class="h-8 text-xs"
							/>
							<Command.List class="max-h-64">
								{#if owner !== null}
									<Command.Item value="any" class="text-xs" onSelect={() => pickOwner(null)}>
										Any owner
									</Command.Item>
								{/if}
								{#if visibleFactions.length > 0}
									<Command.Group heading="Factions">
										{#each visibleFactions as faction (faction.id)}
											<Command.Item
												value="faction-{faction.id}"
												class="text-xs"
												onSelect={() => pickOwner({ kind: 'faction', id: faction.id })}
											>
												<EveImage kind="faction" id={faction.id} class="size-4 rounded-sm" />
												<span class="truncate">{faction.name}</span>
											</Command.Item>
										{/each}
									</Command.Group>
								{/if}
								{#if visibleCorps.length > 0}
									<Command.Group heading="Corporations">
										{#each visibleCorps as corp (corp.id)}
											<Command.Item
												value="corp-{corp.id}"
												class="text-xs"
												onSelect={() => pickOwner({ kind: 'corp', id: corp.id })}
											>
												{#if corp.faction}
													<EveImage kind="faction" id={corp.faction.id} class="size-4 rounded-sm" />
												{:else}
													<EveImage kind="corporation" id={corp.id} class="size-4 rounded-sm" />
												{/if}
												<span class="truncate">{corp.name}</span>
											</Command.Item>
										{/each}
									</Command.Group>
								{/if}
								{#if visibleFactions.length === 0 && visibleCorps.length === 0}
									<p class="px-2 py-2 text-xs text-muted-foreground">No owner by that name.</p>
								{/if}
							</Command.List>
						</Command.Root>
					</Popover.Content>
				</Popover.Root>
				<Select.Root type="single" bind:value={serviceValue}>
					<Select.Trigger
						class="h-7 min-w-0 flex-1 text-xs {serviceId === null ? 'text-muted-foreground' : ''}"
						data-testid="find-service"
					>
						<span class="truncate">{serviceLabel}</span>
					</Select.Trigger>
					<Select.Content>
						<Select.Item value="any" label="Any service">Any service</Select.Item>
						{#each serviceOptions as svc (svc.id)}
							<Select.Item value={String(svc.id)} label={svc.name}>{svc.name}</Select.Item>
						{/each}
					</Select.Content>
				</Select.Root>
			</div>
		{/if}
		{#if origin === null}
			<p
				class="col-span-full p-3 text-center font-mono text-[10px] tracking-wider text-muted-foreground/60 uppercase"
			>
				Pick an origin, or select a system
			</p>
		{:else if findResults.length === 0}
			<p
				class="col-span-full p-3 text-center font-mono text-[10px] tracking-wider text-muted-foreground/60 uppercase"
			>
				No systems found
			</p>
		{/if}
		{#each findResults as result (result.id)}
			{@const r = map.systemInfo(result.id)}
			{@const stations = pickedStations?.stationsBySystem.get(result.id) ?? []}
			{@const expandable = stations.length > 0}
			<!-- The role/tabindex pair is conditional (button only when there is something
			     to expand), which the static a11y check cannot follow. -->
			<!-- svelte-ignore a11y_no_noninteractive_tabindex -->
			<div
				class="{SYSTEM_ROW} border-b border-border/30 px-3 py-1 text-xs hover:bg-muted/30 {expandable
					? 'cursor-pointer'
					: ''}"
				data-testid="find-row"
				role={expandable ? 'button' : 'listitem'}
				tabindex={expandable ? 0 : undefined}
				aria-expanded={expandable ? expandedFind.has(result.id) : undefined}
				onclick={() => expandable && toggleFindRow(result.id)}
				onkeydown={(ev) => {
					if (expandable && (ev.key === 'Enter' || ev.key === ' ')) {
						ev.preventDefault();
						toggleFindRow(result.id);
					}
				}}
				onmouseenter={() => (map.route.hoverPath = result.route.map((s) => s.id))}
				onmouseleave={() => (map.route.hoverPath = null)}
			>
				{#if r}
					<SystemMenu system={r} class={SYSTEM_CELLS_4}>
						<SystemRow system={r} />
					</SystemMenu>
				{:else}
					<span class="col-span-4 truncate text-muted-foreground">{result.id}</span>
				{/if}
				<RoutePopover {map} steps={result.route}>
					<span class="cursor-pointer text-xs font-medium {badgeTone(result.jumps)}">
						{result.jumps}j
					</span>
				</RoutePopover>
				{#if expandable}
					<span
						class="flex items-center gap-0.5 text-[10px] text-muted-foreground"
						data-testid="find-stations-indicator"
					>
						{#if expandedFind.has(result.id)}
							<ChevronDownIcon class="size-3" />
						{:else}
							<ChevronRightIcon class="size-3" />
						{/if}
						{stations.length}
					</span>
				{:else}
					<span></span>
				{/if}
			</div>
			{#if expandedFind.has(result.id)}
				{#each stations as station (station.id)}
					<DestinationMenu destinationId={station.id} class="col-span-full">
						<!-- Hovering a station keeps its system's route highlighted. -->
						<div
							class="col-span-full flex items-center gap-2 border-b border-border/20 py-0.5 pr-3 pl-5 text-[11px] text-muted-foreground hover:bg-muted/20"
							data-testid="find-station"
							role="listitem"
							onmouseenter={() => (map.route.hoverPath = result.route.map((step) => step.id))}
							onmouseleave={() => (map.route.hoverPath = null)}
						>
							<BuildingIcon class="size-3 shrink-0 text-muted-foreground/60" />
							<Hint text={station.name} clippedOnly focusable={false}>
								{#snippet child({ props })}
									<span class="truncate" {...props}>{station.name}</span>
								{/snippet}
							</Hint>
						</div>
					</DestinationMenu>
				{/each}
			{/if}
		{/each}
	</div>
</div>
