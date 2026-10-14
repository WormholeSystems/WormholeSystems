<script lang="ts">
	// Created is the earliest and Updated the latest across the connection and its linked
	// signatures.
	import HeartIcon from '@lucide/svelte/icons/heart';

	import type { MapConnection } from '$lib/api/types/MapConnection';
	import type { Signature } from '$lib/api/types/Signature';
	import * as Tooltip from '$lib/components/ui/tooltip';
	import { timeAgo, utcShort } from '$lib/format';
	import { LIFETIME_OPTIONS, MASS_OPTIONS } from '$lib/map/connection-status';
	import { describeRemaining, lifetimeDeadline } from '$lib/map/lifetime';
	import { tickingMs } from '$lib/now.svelte';

	let { connection, sigs }: { connection: MapConnection; sigs: Signature[] } = $props();

	const clock = tickingMs();
	const HOUR_MS = 3_600_000;

	const typeMeta = $derived(
		connection.kind === 'stargate'
			? { label: 'Stargate', text: 'text-sky-500', dot: 'bg-sky-500' }
			: { label: 'Wormhole', text: 'text-foreground', dot: 'bg-neutral-500' },
	);
	// The shared vocabulary names the states; a status readout paints the healthy ones green
	// rather than neutral.
	const lifetimeMeta = $derived.by(() => {
		const option =
			LIFETIME_OPTIONS.find((o) => o.value === connection.time_status) ?? LIFETIME_OPTIONS[0];
		const label = option.label;
		switch (option.value) {
			case 'eol':
				return { label, text: 'text-purple-500', dot: 'bg-purple-500' };
			case 'critical':
				return { label, text: 'text-red-500', dot: 'bg-red-500' };
			case 'expired':
				return {
					label: `${label} (${option.hint})`,
					text: 'text-red-500',
					dot: option.dot,
				};
			default:
				return { label, text: 'text-green-500', dot: 'bg-green-500' };
		}
	});
	// Every wormhole gets a countdown: a mark's replaces the threshold it stands for, and
	// before one the class lifetime bounds what is left.
	const deadline = $derived(lifetimeDeadline(connection));
	const lifetimeLeft = $derived.by(() => {
		if (deadline === null) return null;
		const left = describeRemaining(deadline.at, clock.current);
		const overdue = deadline.at <= clock.current;
		return {
			label: deadline.estimated && !overdue ? `≈ ${left}` : left,
			text: lifetimeTone(deadline.at - clock.current),
			source: deadline.estimated
				? "Estimated from the hole's age, which starts when it reached the map."
				: 'Counted from when the hole was marked.',
		};
	});
	// The same thresholds a mark stands for, so an estimate reads in the mark's colour once
	// it crosses them.
	function lifetimeTone(remainingMs: number): string {
		if (remainingMs < HOUR_MS) return 'text-red-500';
		if (remainingMs < 4 * HOUR_MS) return 'text-purple-500';
		return 'text-green-500';
	}
	const massMeta = $derived.by(() => {
		const option = MASS_OPTIONS.find((o) => o.value === connection.mass_status);
		if (!option) return { label: 'Unknown', text: 'text-muted-foreground', dot: 'bg-neutral-500' };
		switch (option.value) {
			case 'reduced':
				return { label: option.label, text: 'text-amber-500', dot: 'bg-amber-500' };
			case 'critical':
				return { label: option.label, text: 'text-red-500', dot: 'bg-red-500' };
			default:
				return { label: option.label, text: 'text-green-500', dot: 'bg-green-500' };
		}
	});

	const createdMs = $derived(
		Math.min(Date.parse(connection.created_at), ...sigs.map((s) => Date.parse(s.created_at))),
	);
	const updatedMs = $derived(
		Math.max(Date.parse(connection.updated_at), ...sigs.map((s) => Date.parse(s.updated_at))),
	);

	function ago(ms: number): string {
		return timeAgo(ms, new Date(clock.current));
	}

	const degraded = $derived(connection.time_status !== null && connection.time_status !== 'stable');
	const lifetimeSince = $derived(
		connection.time_status_updated_at === null
			? null
			: Date.parse(connection.time_status_updated_at),
	);
</script>

<div class="space-y-1">
	<div class="border-b pb-1 text-xs font-medium text-foreground">Status</div>
	<div class="grid grid-cols-2 divide-y truncate text-xs text-muted-foreground *:py-1">
		<div class="col-span-full grid grid-cols-subgrid">
			<span>Type</span>
			<span class="flex items-center justify-end gap-1.5 text-right {typeMeta.text}">
				<span class="inline-block size-2 rounded-full {typeMeta.dot}"></span>
				{typeMeta.label}
			</span>
		</div>
		<div class="col-span-full grid grid-cols-subgrid">
			<span>Lifetime</span>
			<span
				class="flex items-center justify-end gap-1.5 text-right {lifetimeMeta.text}"
				data-testid="popover-lifetime"
			>
				<span class="inline-block size-2 shrink-0 rounded-full {lifetimeMeta.dot}"></span>
				{#if degraded && lifetimeSince !== null}
					<Tooltip.Root>
						<Tooltip.Trigger class="cursor-help">{lifetimeMeta.label}</Tooltip.Trigger>
						<Tooltip.Content>
							Since {utcShort(lifetimeSince)} ({ago(lifetimeSince)})
						</Tooltip.Content>
					</Tooltip.Root>
				{:else}
					<span>{lifetimeMeta.label}</span>
				{/if}
			</span>
		</div>
		{#if lifetimeLeft}
			<div class="col-span-full grid grid-cols-subgrid">
				<span>Lifetime left</span>
				<Tooltip.Root>
					<Tooltip.Trigger
						class="cursor-help text-right tabular-nums {lifetimeLeft.text}"
						data-testid="popover-lifetime-detail"
					>
						{lifetimeLeft.label}
					</Tooltip.Trigger>
					<Tooltip.Content>{lifetimeLeft.source}</Tooltip.Content>
				</Tooltip.Root>
			</div>
		{/if}
		<div class="col-span-full grid grid-cols-subgrid">
			<span>Mass Status</span>
			<span class="flex items-center justify-end gap-1.5 text-right {massMeta.text}">
				<span class="inline-block size-2 rounded-full {massMeta.dot}"></span>
				{massMeta.label}
			</span>
		</div>
		{#if connection.preserve_mass}
			<div class="col-span-full grid grid-cols-subgrid" data-testid="popover-preserve-mass">
				<span>Preserve mass</span>
				<span class="flex items-center justify-end gap-1.5 text-right text-emerald-500">
					<HeartIcon class="size-3" />
					Yes
				</span>
			</div>
		{/if}
		<div class="col-span-full grid grid-cols-subgrid">
			<span>Created</span>
			<Tooltip.Root>
				<Tooltip.Trigger class="cursor-help text-right">{ago(createdMs)}</Tooltip.Trigger>
				<Tooltip.Content>{utcShort(createdMs)}</Tooltip.Content>
			</Tooltip.Root>
		</div>
		<div class="col-span-full grid grid-cols-subgrid">
			<span>Updated</span>
			<Tooltip.Root>
				<Tooltip.Trigger class="cursor-help text-right">{ago(updatedMs)}</Tooltip.Trigger>
				<Tooltip.Content>{utcShort(updatedMs)}</Tooltip.Content>
			</Tooltip.Root>
		</div>
	</div>
</div>
