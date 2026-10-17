<script lang="ts">
	// The jump log in a popover, with the manual add/edit form in a second one nested inside.
	import ChevronRightIcon from '@lucide/svelte/icons/chevron-right';
	import EllipsisVerticalIcon from '@lucide/svelte/icons/ellipsis-vertical';
	import MoveLeftIcon from '@lucide/svelte/icons/move-left';
	import MoveRightIcon from '@lucide/svelte/icons/move-right';
	import PencilIcon from '@lucide/svelte/icons/pencil';
	import PencilLineIcon from '@lucide/svelte/icons/pencil-line';
	import PlusIcon from '@lucide/svelte/icons/plus';
	import Trash2Icon from '@lucide/svelte/icons/trash-2';

	import { createQuery } from '@tanstack/svelte-query';

	import { q } from '$lib/api/queries';
	import type { ConnectionJump } from '$lib/api/types/ConnectionJump';
	import type { MapConnection } from '$lib/api/types/MapConnection';
	import type { MapSystemView } from '$lib/api/types/MapSystemView';
	import * as DropdownMenu from '$lib/components/ui/dropdown-menu';
	import * as Popover from '$lib/components/ui/popover';
	import * as Tooltip from '$lib/components/ui/tooltip';
	import EveImage from '$lib/components/EveImage.svelte';
	import { timeAgoShort, utcShort } from '$lib/format';
	import { tickingMs } from '$lib/now.svelte';
	import { formatKt } from '$lib/map/mass';
	import { solarSystemId } from '$lib/map/system';
	import type { MapState } from '../../state/map-state.svelte';
	import JumpForm from './JumpForm.svelte';
	import Hint from '$lib/components/Hint.svelte';

	let {
		map,
		connection,
		source,
		target,
		canWrite,
	}: {
		map: MapState;
		connection: MapConnection;
		source: MapSystemView;
		target: MapSystemView;
		canWrite: boolean;
	} = $props();

	let logOpen = $state(false);
	// Fetched on open; invalidating while closed is a no-op that just marks it stale.
	const jumpsQuery = createQuery(() => ({
		...q.listConnectionJumps(map.mapId, connection.id),
		enabled: logOpen,
	}));
	const jumps = $derived(jumpsQuery.data ?? []);
	function refreshLog() {
		map.connections.refreshJumps(connection.id);
	}
	// New transits arrive with the map refetch: the counters changing is the signal that
	// the log behind them moved.
	$effect(() => {
		void connection.jumps_count;
		void connection.jumps_mass_sum;
		refreshLog();
	});

	function isOutbound(jump: ConnectionJump): boolean {
		return jump.from_solar_system_id === solarSystemId(source);
	}

	const clock = tickingMs();

	let formOpen = $state(false);
	let editing = $state<ConnectionJump | null>(null);

	function openAdd() {
		editing = null;
		formOpen = true;
	}

	function openEdit(jump: ConnectionJump) {
		editing = jump;
		// Deferred: the closing dropdown's focus restore would dismiss a popover opened in the
		// same tick.
		setTimeout(() => (formOpen = true));
	}

	function formDone() {
		formOpen = false;
		refreshLog();
	}

	function deleteJump(jump: ConnectionJump) {
		void map.connections.removeJump(jump.id).then(refreshLog, () => {});
	}
</script>

<Popover.Root bind:open={logOpen}>
	<Popover.Trigger
		class="flex items-center gap-0.5 rounded font-normal text-muted-foreground transition-colors hover:text-foreground"
		data-testid="jump-log-trigger"
	>
		{connection.jumps_count} jumps
		<ChevronRightIcon class="size-3" />
	</Popover.Trigger>
	<Popover.Content class="w-96 p-0" side="right" align="start" data-testid="jump-log">
		<div class="max-h-64 overflow-y-auto px-3">
			<div class="grid grid-cols-[auto_auto_1fr_auto_auto_auto_auto] divide-y divide-border/40">
				<div
					class="sticky top-0 z-10 col-span-full grid grid-cols-subgrid gap-x-3 bg-popover py-1.5 text-[10px] font-medium tracking-wider text-muted-foreground uppercase"
				>
					<span class="col-span-3">Ship</span>
					<span>Pilot</span>
					<span class="text-right">kt</span>
					<span class="text-right">Age</span>
					{#if canWrite}
						<Hint text={'Log jump manually'}>
							{#snippet child({ props })}
								<button
									class="flex items-center justify-end text-muted-foreground transition-colors hover:text-foreground"
									{...props}
									aria-label="Log jump manually"
									data-testid="log-jump"
									onclick={openAdd}
								>
									<PlusIcon class="size-3" />
								</button>
							{/snippet}
						</Hint>
					{:else}
						<span></span>
					{/if}
				</div>

				{#if jumps.length === 0}
					<div class="col-span-full py-3 text-center text-xs text-muted-foreground">
						No jumps logged yet
					</div>
				{/if}
				{#each jumps as jump (jump.id)}
					<div
						class="group col-span-full grid grid-cols-subgrid items-center gap-x-3 py-1.5 transition-colors hover:bg-muted/30"
						data-testid="jump-row"
					>
						{#if isOutbound(jump)}
							<MoveRightIcon class="size-3 shrink-0 text-sky-500" />
						{:else}
							<MoveLeftIcon class="size-3 shrink-0 text-purple-500" />
						{/if}
						<div class="size-5 shrink-0">
							{#if jump.ship_type_id !== null}
								<EveImage kind="type" id={jump.ship_type_id} class="size-5 rounded" />
							{/if}
						</div>
						<span class="min-w-0 truncate text-xs text-foreground">
							{jump.ship_type_name ?? 'Unknown'}
						</span>
						{#if jump.character_id !== null}
							<span class="flex items-center gap-1">
								<EveImage
									kind="character"
									id={jump.character_id}
									class="size-4 shrink-0 rounded-full"
								/>
								<span class="max-w-20 truncate text-[10px] text-muted-foreground">
									{jump.character_name}
								</span>
							</span>
						{:else}
							<div class="flex items-center gap-1 text-[10px] text-muted-foreground italic">
								<PencilLineIcon class="size-3 shrink-0" />
								manual
							</div>
						{/if}
						<span
							class="text-right font-mono text-[10px] whitespace-nowrap text-foreground/80 tabular-nums"
						>
							{formatKt(jump.mass)}
						</span>
						<Tooltip.Root>
							<Tooltip.Trigger
								class="cursor-help text-right font-mono text-[10px] whitespace-nowrap text-muted-foreground tabular-nums"
							>
								{timeAgoShort(jump.created_at, new Date(clock.current))}
							</Tooltip.Trigger>
							<Tooltip.Content>{utcShort(jump.created_at)}</Tooltip.Content>
						</Tooltip.Root>
						{#if canWrite}
							<DropdownMenu.Root>
								<Hint text={'Jump actions'}>
									{#snippet child({ props })}
										<DropdownMenu.Trigger
											class="flex items-center justify-end text-muted-foreground hover:text-foreground"
											{...props}
											aria-label="Jump actions"
										>
											<EllipsisVerticalIcon class="size-3" />
										</DropdownMenu.Trigger>
									{/snippet}
								</Hint>
								<DropdownMenu.Content side="right" align="start">
									<DropdownMenu.Item class="text-xs" onclick={() => openEdit(jump)}>
										<PencilIcon class="size-3.5" />
										Edit
									</DropdownMenu.Item>
									<DropdownMenu.Item
										class="text-xs text-destructive focus:text-destructive"
										onclick={() => deleteJump(jump)}
									>
										<Trash2Icon class="size-3.5" />
										Delete
									</DropdownMenu.Item>
								</DropdownMenu.Content>
							</DropdownMenu.Root>
						{:else}
							<span></span>
						{/if}
					</div>
				{/each}
			</div>
		</div>
		<div class="flex items-center justify-between border-t bg-muted/30 px-3 py-2 text-xs">
			<span class="font-medium text-foreground">
				Total
				{#if connection.jumps_count > jumps.length}
					<span class="font-normal text-muted-foreground">
						· latest {jumps.length} of {connection.jumps_count} shown
					</span>
				{/if}
			</span>
			<span class="font-mono text-foreground tabular-nums">
				{formatKt(connection.jumps_mass_sum)} kt
			</span>
		</div>

		<Popover.Root bind:open={formOpen}>
			<Popover.Trigger class="pointer-events-none absolute top-0 right-0" tabindex={-1} />
			<Popover.Content
				class="w-72 p-3"
				side="right"
				align="start"
				data-testid="jump-form"
				onpointerdown={(ev: PointerEvent) => ev.stopPropagation()}
			>
				<JumpForm
					{map}
					{connection}
					{source}
					{target}
					{editing}
					ondone={formDone}
					oncancel={() => (formOpen = false)}
				/>
			</Popover.Content>
		</Popover.Root>
	</Popover.Content>
</Popover.Root>
