<script lang="ts">
	// The settings screen's own access table, given grants and no actions. What a visitor
	// sees here is the component the app renders, not a second drawing of it.
	import type { AccessEntry } from '$lib/api/types/AccessEntry';
	import AccessTable from '$lib/components/AccessTable.svelte';
	import MapPanelHeader from '$lib/components/map-panel/MapPanelHeader.svelte';
	import { ROLE_HELP, ROLE_LABEL, ROLES_ASCENDING } from '$lib/map/roles';
	import { DEMO_ACCESS } from './demo-chain';

	const entries: AccessEntry[] = DEMO_ACCESS;
</script>

<div class="flex flex-col gap-8">
	<div class="overflow-hidden rounded border border-border bg-card">
		<MapPanelHeader>
			Access · home.map
			{#snippet actions()}
				<span class="font-mono text-[10px] text-muted-foreground">{entries.length}</span>
			{/snippet}
		</MapPanelHeader>
		<AccessTable {entries} />
	</div>

	<ul class="grid gap-4 sm:grid-cols-2">
		{#each ROLES_ASCENDING as role (role)}
			<li class="flex flex-col gap-1">
				<span class="font-mono text-[10px] tracking-[0.15em] text-muted-foreground uppercase">
					{ROLE_LABEL[role]}
				</span>
				<span class="text-sm text-muted-foreground">{ROLE_HELP[role]}</span>
			</li>
		{/each}
	</ul>
</div>
