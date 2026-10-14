<script lang="ts">
	// What "clean map" is about to do. Removing a branch is easy to regret and hard to read
	// off the canvas, so this names every system going and says what keeps the rest.
	import ClassBadge from '$lib/components/ClassBadge.svelte';
	import { Button } from '$lib/components/ui/button';
	import * as Dialog from '$lib/components/ui/dialog';
	import type { MapState } from '../../state/map-state.svelte';
	import { solarSystemId } from '$lib/map/system';

	let { map, open = $bindable() }: { map: MapState; open: boolean } = $props();

	const going = $derived(map.systems.orphaned);
	const anchors = $derived(map.systems.all.filter((s) => s.is_pinned || s.is_home));
	const signatures = $derived(
		map.signatures.all.filter((sig) => going.some((s) => solarSystemId(s) === sig.solar_system_id))
			.length,
	);

	function clean() {
		map.systems.removeOrphaned();
		open = false;
	}
</script>

<Dialog.Root bind:open>
	<Dialog.Content class="sm:max-w-md" data-testid="clean-map-dialog">
		<Dialog.Header>
			<Dialog.Title>Clean the map?</Dialog.Title>
			<Dialog.Description>
				{going.length === 1 ? 'One system' : `${going.length} systems`} nothing reaches any more, from
				{anchors.length === 1 ? 'the one anchor' : `the ${anchors.length} anchors`} on this map. Everything
				still hanging off a pinned or home system stays.
			</Dialog.Description>
		</Dialog.Header>

		<ul class="max-h-64 overflow-y-auto border border-border/60" data-testid="clean-list">
			{#each going as node (node.id)}
				{@const mapped = node.kind === 'system' ? node : null}
				<li
					class="flex items-center gap-2 border-b border-border/40 px-2 py-1.5 text-xs last:border-b-0"
				>
					<ClassBadge
						classId={mapped?.wormhole_class_id ?? null}
						security={mapped?.security_status ?? null}
					/>
					{#if node.alias}
						<span class="font-medium">{node.alias}</span>
					{/if}
					<span class="truncate {node.alias ? 'text-muted-foreground' : 'font-medium'}">
						{mapped?.name ?? 'Unmapped'}
					</span>
					<span class="ml-auto shrink-0 truncate text-muted-foreground">{mapped?.region ?? ''}</span
					>
				</li>
			{/each}
		</ul>

		<p class="text-xs text-muted-foreground">
			Their connections go with them{signatures > 0
				? `, along with ${signatures === 1 ? 'the signature' : `${signatures} signatures`} scanned in them`
				: ''}. Notes, status and occupier are kept, and come back if the system is placed again. One
			undo puts all of it back.
		</p>

		<Dialog.Footer>
			<Button variant="ghost" onclick={() => (open = false)}>Cancel</Button>
			<Button variant="destructive" onclick={clean} data-testid="clean-map-confirm">
				Clean {going.length}
				{going.length === 1 ? 'system' : 'systems'}
			</Button>
		</Dialog.Footer>
	</Dialog.Content>
</Dialog.Root>
