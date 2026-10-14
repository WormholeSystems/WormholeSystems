<script lang="ts">
	// Remaining-mass bar with 10%/50% ticks, the jump log in a nested popover, and the manual
	// log form in a third. The bar is an estimate from tracked hull masses; the manual
	// mass-status flag is independent of it.
	import type { MapConnection } from '$lib/api/types/MapConnection';
	import type { MapSystemView } from '$lib/api/types/MapSystemView';
	import type { SignatureTypeInfo } from '$lib/api/types/SignatureTypeInfo';
	import MassBar from '$lib/components/MassBar.svelte';
	import * as Tooltip from '$lib/components/ui/tooltip';
	import { formatKt, remainingMass } from '$lib/map/mass';
	import type { MapState } from '../../state/map-state.svelte';
	import JumpLogPopover from './JumpLogPopover.svelte';

	let {
		map,
		connection,
		source,
		target,
		physics,
		canWrite,
	}: {
		map: MapState;
		connection: MapConnection;
		source: MapSystemView;
		target: MapSystemView;
		physics: SignatureTypeInfo | null;
		canWrite: boolean;
	} = $props();

	const remaining = $derived(remainingMass(physics?.total_mass ?? null, connection.jumps_mass_sum));
</script>

<div class="space-y-1" data-testid="mass-tracking">
	<div class="flex items-center justify-between border-b pb-1 text-xs font-medium text-foreground">
		<Tooltip.Root>
			<Tooltip.Trigger class="cursor-help">Mass (estimate)</Tooltip.Trigger>
			<Tooltip.Content>
				Only tracked pilots are counted automatically; the total mass varies by ±10% in game.
			</Tooltip.Content>
		</Tooltip.Root>

		<JumpLogPopover {map} {connection} {source} {target} {canWrite} />
	</div>

	<div class="grid grid-cols-2 divide-y text-xs text-muted-foreground *:py-1">
		{#if remaining !== null}
			<div class="col-span-full">
				<MassBar remainingPercent={remaining.percent} />
			</div>
			<div class="col-span-full grid grid-cols-subgrid">
				<span>Remaining</span>
				<span class="text-right tabular-nums" data-testid="mass-remaining">
					≈ {formatKt(remaining.kg)} ({Math.round(remaining.percent)}%)
				</span>
			</div>
		{/if}
		<div class="col-span-full grid grid-cols-subgrid">
			<span>Jumped</span>
			<span class="text-right tabular-nums" data-testid="mass-jumped">
				{formatKt(connection.jumps_mass_sum)}
			</span>
		</div>
	</div>
</div>
