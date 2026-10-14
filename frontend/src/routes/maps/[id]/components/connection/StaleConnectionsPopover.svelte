<script lang="ts">
	// Holes critical or expired for over an hour: list them, and offer to sweep them all at once.
	import TriangleAlertIcon from '@lucide/svelte/icons/triangle-alert';

	import { Badge } from '$lib/components/ui/badge';
	import { Button } from '$lib/components/ui/button';
	import * as Popover from '$lib/components/ui/popover';
	import type { MapState } from '../../state/map-state.svelte';

	let { map }: { map: MapState } = $props();

	const canWrite = $derived(map.canWrite);
</script>

{#if map.connections.stale.length > 0}
	<Popover.Root>
		<Popover.Trigger>
			{#snippet child({ props })}
				<Badge
					{...props}
					variant="outline"
					class="cursor-pointer gap-1 border-red-600/40 text-red-500"
					data-testid="stale-badge"
				>
					<TriangleAlertIcon />
					{map.connections.stale.length} stale
				</Badge>
			{/snippet}
		</Popover.Trigger>
		<Popover.Content class="w-80 p-0" align="center">
			<div class="border-b border-border/50 px-3 py-2 text-xs">
				Critical or expired for over an hour, so probably long gone.
			</div>
			<ul class="max-h-64 overflow-y-auto py-1" data-testid="stale-list">
				{#each map.connections.stale as s (s.connection_id)}
					<li class="px-3 py-1 text-xs">
						{s.from_name}
						<span class="text-muted-foreground">to</span>
						{s.to_name}
					</li>
				{/each}
			</ul>
			{#if canWrite}
				<div class="border-t border-border/50 p-2">
					<Button
						variant="destructive"
						size="sm"
						class="w-full"
						data-testid="clean-stale"
						onclick={() => map.connections.cleanStale()}
					>
						Remove {map.connections.stale.length === 1 ? 'it' : 'them'}
					</Button>
				</div>
			{/if}
		</Popover.Content>
	</Popover.Root>
{/if}
