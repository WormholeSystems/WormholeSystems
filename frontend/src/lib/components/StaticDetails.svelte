<script lang="ts">
	// Wormhole physics for one static, the tooltip/popover body (legacy StaticDetails).
	import type { Static } from '$lib/api/types/Static';
	import { destClassMeta } from '$lib/map/classes';
	import { formatKt, jumpMassLetter } from '$lib/map/mass';

	let { static: st }: { static: Static } = $props();

	const dest = $derived(destClassMeta(st.dest_class));
	// Header shows the full destination form: HS/LS/NS for k-space, C-number otherwise.
	const destFull = $derived(dest.isKnownSpace ? dest.token.toUpperCase() : dest.short);

	function kt(kg: number | null): string {
		return kg === null ? '—' : `${formatKt(kg)} kt`;
	}
</script>

<div class="min-w-40">
	<div class="border-b border-border/50 px-3 py-2">
		<span class="font-mono text-xs font-medium">
			{st.code} →
			<span class="uppercase" style:color="var(--color-{dest.token})">{destFull}</span>
		</span>
	</div>
	<div class="flex flex-col gap-1 px-3 py-2 text-[11px]">
		<div class="flex justify-between gap-4">
			<span class="text-muted-foreground">Total Mass</span>
			<span>{kt(st.total_mass)}</span>
		</div>
		<div class="flex justify-between gap-4">
			<span class="text-muted-foreground">Max Jump Mass</span>
			<span>{kt(st.max_jump_mass)}</span>
		</div>
		<div class="flex justify-between gap-4">
			<span class="text-muted-foreground">Ship Size</span>
			<span>{jumpMassLetter(st.max_jump_mass)}</span>
		</div>
		<div class="flex justify-between gap-4">
			<span class="text-muted-foreground">Lifetime</span>
			<span>{st.lifetime_hours === null ? '—' : `${st.lifetime_hours}h`}</span>
		</div>
		<div class="flex justify-between gap-4">
			<span class="text-muted-foreground">Sig Strength</span>
			<span>{st.signature_strength === null ? 'unknown' : `${st.signature_strength}%`}</span>
		</div>
	</div>
</div>
