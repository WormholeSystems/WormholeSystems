<script lang="ts">
	// One solar system result row, shared by every system picker: class, name, region, then the
	// holder cell (statics and effect for J-space, sovereignty for k-space). The cells are grid
	// items and the parent owns the track sizes, see `pickers/columns.ts`. The holder cell draws
	// statics and effects exactly as a map node does, since the glyphs say where a hole leads.
	import type { SystemSearchResult } from '$lib/api/types/SystemSearchResult';
	import EffectBadge from '$lib/components/EffectBadge.svelte';
	import SovereigntyBadge from '$lib/components/SovereigntyBadge.svelte';
	import ClassBadge from '$lib/components/ClassBadge.svelte';
	import StaticDetails from '$lib/components/StaticDetails.svelte';
	import * as Tooltip from '$lib/components/ui/tooltip';
	import { destClassMeta } from '$lib/map/classes';

	let { system }: { system: SystemSearchResult } = $props();

	const sov = $derived(system.sovereignty);
</script>

<ClassBadge
	classId={system.wormhole_class_id}
	security={system.security}
	class="truncate text-xs"
/>
<span class="min-w-0 truncate text-foreground">{system.name}</span>
<span class="min-w-0 truncate text-xs text-muted-foreground">{system.region}</span>
{#if system.statics.length > 0 || system.effect_name}
	<span class="flex items-center justify-end gap-1">
		{#each system.statics as st (st.code)}
			{@const dest = destClassMeta(st.dest_class)}
			<Tooltip.Root>
				<Tooltip.Trigger
					class="cursor-help text-[10px] font-medium"
					data-testid="row-static"
					style="color: var(--color-{dest.token})"
				>
					{dest.short}
				</Tooltip.Trigger>
				<Tooltip.Content class="p-0" side="bottom">
					<StaticDetails static={st} />
				</Tooltip.Content>
			</Tooltip.Root>
		{/each}
		{#if system.effect_name}
			<EffectBadge
				name={system.effect_name}
				wormholeClassId={system.wormhole_class_id ?? 0}
				detail={false}
			/>
		{/if}
	</span>
{:else if sov}
	<!-- Logo only; the holder's ticker and name are one hover away. -->
	<span class="flex items-center justify-end">
		<SovereigntyBadge sovereignty={sov} />
	</span>
{:else}
	<span></span>
{/if}
