<script lang="ts">
	// Right-clicking empty canvas: add, tidy, or wipe the map.
	import BrushCleaningIcon from '@lucide/svelte/icons/brush-cleaning';
	import BugIcon from '@lucide/svelte/icons/bug';
	import EraserIcon from '@lucide/svelte/icons/eraser';
	import PlusIcon from '@lucide/svelte/icons/plus';
	import Trash2Icon from '@lucide/svelte/icons/trash-2';

	import { toast } from 'svelte-sonner';

	import { errorMessage } from '$lib/api/client';
	import * as DropdownMenu from '$lib/components/ui/dropdown-menu';
	import { confirmDanger } from '$lib/confirm.svelte';
	import { NODE_W } from '$lib/map/geometry';
	import type { MapState, Menu } from '../../state/map-state.svelte';

	let { map, menu }: { map: MapState; menu: Menu } = $props();

	function addSystem() {
		map.linkFrom = null;
		// Centred on the click, so the new system lands where the map was right-clicked.
		const w = map.camera.toWorld(menu.x, menu.y);
		map.searchAnchor = { x: w.x - NODE_W / 2, y: w.y - map.nodeH / 2 };
		map.paletteOpen = true;
	}

	/** Ask first: what goes is a branch nobody is looking at. */
	function cleanMap() {
		map.cleanPrompt = true;
	}

	async function clearMap() {
		const sure = await confirmDanger({
			title: 'Clear the map?',
			body: 'This removes all systems except home and pinned ones.',
			action: 'Clear map',
		});
		if (sure) map.systems.clearAll();
	}

	// Loaded on demand, so none of it reaches a built app.
	const dev = import.meta.env.DEV;

	function stressChain() {
		toast.promise(
			import('../../state/debug').then((debug) => debug.seedStressChain(map)),
			{
				loading: 'Building a stress-test chain…',
				success: 'Stress-test chain built',
				error: (err) => `debug: ${errorMessage(err)}`,
			},
		);
	}
</script>

<DropdownMenu.Item onSelect={addSystem}>
	<PlusIcon class="size-4" />
	Add solar system
</DropdownMenu.Item>
{#if map.selected.size > 0}
	<DropdownMenu.Item onSelect={() => map.removeSelection()}>
		<Trash2Icon class="size-4" />
		Delete selection
	</DropdownMenu.Item>
{/if}
{#if map.systems.orphaned.length > 0}
	<DropdownMenu.Item onSelect={cleanMap} data-testid="clean-map">
		<BrushCleaningIcon class="size-4" />
		Clean map ({map.systems.orphaned.length})
	</DropdownMenu.Item>
{/if}
<DropdownMenu.Item onSelect={clearMap}>
	<EraserIcon class="size-4" />
	Clear map
</DropdownMenu.Item>
{#if dev}
	<DropdownMenu.Separator />
	<DropdownMenu.Sub>
		<DropdownMenu.SubTrigger data-testid="debug-subtrigger">
			<BugIcon class="size-4" />
			Debug
		</DropdownMenu.SubTrigger>
		<DropdownMenu.SubContent data-testid="debug-submenu">
			<DropdownMenu.Item onSelect={stressChain}>Add a stress-test chain</DropdownMenu.Item>
		</DropdownMenu.SubContent>
	</DropdownMenu.Sub>
{/if}
