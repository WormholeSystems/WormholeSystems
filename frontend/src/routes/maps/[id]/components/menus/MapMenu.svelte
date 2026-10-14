<script lang="ts">
	// Right-clicking empty canvas: add, tidy, or wipe the map.
	import BrushCleaningIcon from '@lucide/svelte/icons/brush-cleaning';
	import BugIcon from '@lucide/svelte/icons/bug';
	import ChevronRightIcon from '@lucide/svelte/icons/chevron-right';
	import EraserIcon from '@lucide/svelte/icons/eraser';
	import PlusIcon from '@lucide/svelte/icons/plus';
	import Trash2Icon from '@lucide/svelte/icons/trash-2';

	import { toast } from 'svelte-sonner';

	import { errorMessage } from '$lib/api/client';
	import { confirmDanger } from '$lib/confirm.svelte';
	import { NODE_W } from '$lib/map/geometry';
	import type { MapState, Menu } from '../../state/map-state.svelte';
	import { item, panel, sub } from './chrome';

	let { map, menu }: { map: MapState; menu: Menu } = $props();

	function close() {
		map.closeMenu();
	}

	function addSystem() {
		map.linkFrom = null;
		// Centred on the click, so the new system lands where the map was right-clicked.
		const w = map.camera.toWorld(menu.x, menu.y);
		map.searchAnchor = { x: w.x - NODE_W / 2, y: w.y - map.nodeH / 2 };
		map.paletteOpen = true;
		close();
	}

	function deleteSelection() {
		const ids = [...map.selected];
		map.selected = new Set();
		map.systems.remove(ids);
		close();
	}

	/** Ask first: what goes is a branch nobody is looking at. */
	function cleanMap() {
		map.cleanPrompt = true;
		close();
	}

	async function clearMap() {
		close();
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
		close();
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

<button class={item} onclick={addSystem}>
	<PlusIcon class="size-4" />
	Add solar system
</button>
{#if map.selected.size > 0}
	<button class={item} onclick={deleteSelection}>
		<Trash2Icon class="size-4" />
		Delete selection
	</button>
{/if}
{#if map.systems.orphaned.length > 0}
	<button class={item} onclick={cleanMap} data-testid="clean-map">
		<BrushCleaningIcon class="size-4" />
		Clean map ({map.systems.orphaned.length})
	</button>
{/if}
<button class={item} onclick={clearMap}>
	<EraserIcon class="size-4" />
	Clear map
</button>
{#if dev}
	<div class="my-0.5 border-t border-border"></div>
	<div class={sub} data-testid="debug-subtrigger">
		<BugIcon class="size-4" />
		Debug
		<ChevronRightIcon class="ml-auto size-3" />
		<div class={panel} data-testid="debug-submenu">
			<button class={item} onclick={stressChain}>Add a stress-test chain</button>
		</div>
	</div>
{/if}
