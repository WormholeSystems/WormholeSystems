<script lang="ts">
	// Physics of the resolved wormhole type, from the enriched signature catalog.
	import type { SignatureTypeInfo } from '$lib/api/types/SignatureTypeInfo';
	import { formatKt, jumpMassLetter } from '$lib/map/mass';

	let { type }: { type: SignatureTypeInfo } = $props();
</script>

<div class="space-y-1" data-testid="popover-properties">
	<div class="border-b pb-1 text-xs font-medium text-foreground">Properties</div>
	<div class="grid grid-cols-2 divide-y truncate text-xs text-muted-foreground *:py-1">
		{#if type.total_mass != null}
			<div class="col-span-full grid grid-cols-subgrid">
				<span>Total Mass</span>
				<span class="text-right">{formatKt(type.total_mass)} kt</span>
			</div>
		{/if}
		{#if type.max_jump_mass != null}
			<div class="col-span-full grid grid-cols-subgrid">
				<span>Maximum Jump Mass</span>
				<span class="text-right">{formatKt(type.max_jump_mass)} kt</span>
			</div>
		{/if}
		{#if type.lifetime_hours != null}
			<div class="col-span-full grid grid-cols-subgrid">
				<span>Maximum Lifetime</span>
				<span class="text-right">{type.lifetime_hours}h</span>
			</div>
		{/if}
		<div class="col-span-full grid grid-cols-subgrid">
			<span>Ship Size</span>
			<span class="text-right">{jumpMassLetter(type.max_jump_mass ?? null)}</span>
		</div>
		{#if type.signature_strength != null}
			<div class="col-span-full grid grid-cols-subgrid">
				<span>Sig Strength</span>
				<span class="text-right">{type.signature_strength}%</span>
			</div>
		{/if}
	</div>
</div>
