<script lang="ts">
	// Settings for the screen size being arranged: what it is called, the window width it
	// takes over at, and the grid its tiles snap to.
	import RotateCcwIcon from '@lucide/svelte/icons/rotate-ccw';
	import TrashIcon from '@lucide/svelte/icons/trash-2';

	import { Button } from '$lib/components/ui/button';
	import { Input } from '$lib/components/ui/input';
	import { Slider } from '$lib/components/ui/slider';
	import {
		MAX_COLUMNS,
		ROW_HEIGHT_RANGE,
		type BreakpointLayout,
		type BreakpointMeta,
	} from '../../state/layout/registry';
	import type { MapState } from '../../state/map-state.svelte';

	let {
		map,
		bp,
		layout,
		breakpoints,
	}: {
		map: MapState;
		bp: BreakpointMeta;
		layout: BreakpointLayout;
		breakpoints: BreakpointMeta[];
	} = $props();

	// The narrowest size covers every window below the next one, so its start is fixed.
	const fixedStart = $derived(bp.key === 'xs');
	const next = $derived(breakpoints.find((other) => other.minWidth > bp.minWidth));
	const range = $derived(
		next ? `${bp.minWidth}px to ${next.minWidth - 1}px` : `${bp.minWidth}px and wider`,
	);

	let widthError = $state('');

	function rename(value: string) {
		const label = value.trim();
		if (label.length === 0 || label.length > 32 || label === bp.label) return;
		map.panels.updateBreakpoint(bp.key, { label });
	}

	function move(value: number | null) {
		widthError = '';
		if (value === null || value === bp.minWidth) return;
		if (!Number.isInteger(value) || value < 1 || value > 10_000) {
			widthError = 'Pick a width between 1 and 10000';
			return;
		}
		if (breakpoints.some((other) => other.key !== bp.key && other.minWidth === value)) {
			widthError = 'Another size already starts there';
			return;
		}
		map.panels.updateBreakpoint(bp.key, { minWidth: value });
	}
</script>

<div class="flex flex-col gap-4 p-3" data-testid="breakpoint-settings">
	<div class="flex gap-2">
		<label class="flex min-w-0 flex-1 flex-col gap-1">
			<span class="text-[11px] text-muted-foreground">Name</span>
			<Input
				value={bp.label}
				maxlength={32}
				class="h-8 text-xs"
				data-testid="breakpoint-label"
				onchange={(e) => rename(e.currentTarget.value)}
			/>
		</label>
		<label class="flex w-24 flex-col gap-1">
			<span class="text-[11px] text-muted-foreground">From (px)</span>
			<Input
				type="number"
				value={bp.minWidth}
				min={1}
				max={10000}
				disabled={fixedStart}
				class="h-8 text-xs"
				data-testid="breakpoint-width"
				onchange={(e) => move(e.currentTarget.valueAsNumber)}
			/>
		</label>
	</div>
	<p class="-mt-2 text-[11px] text-muted-foreground">
		{#if widthError}
			<span class="text-destructive">{widthError}</span>
		{:else}
			Used for windows {range}.
		{/if}
	</p>

	<div class="flex flex-col gap-2">
		<div class="flex items-center justify-between text-xs">
			<span class="text-muted-foreground">Columns</span>
			<span class="font-mono tabular-nums" data-testid="grid-cols">{layout.cols}</span>
		</div>
		<Slider
			type="single"
			min={1}
			max={MAX_COLUMNS}
			step={1}
			value={layout.cols}
			aria-label="Columns"
			onValueChange={(v) => map.panels.setGrid(bp.key, { cols: v })}
		/>
	</div>
	<div class="flex flex-col gap-2">
		<div class="flex items-center justify-between text-xs">
			<span class="text-muted-foreground">Row height</span>
			<span class="font-mono tabular-nums" data-testid="grid-row-height">
				{layout.row_height}px
			</span>
		</div>
		<Slider
			type="single"
			min={ROW_HEIGHT_RANGE.min}
			max={ROW_HEIGHT_RANGE.max}
			step={10}
			value={layout.row_height}
			aria-label="Row height"
			onValueChange={(v) => map.panels.setGrid(bp.key, { rowHeight: v })}
		/>
	</div>

	<div class="flex items-center justify-between">
		<Button
			variant="ghost"
			size="sm"
			data-testid="breakpoint-reset"
			onclick={() => map.panels.reset(bp.key)}
		>
			<RotateCcwIcon />
			Reset arrangement
		</Button>
		{#if bp.custom}
			<Button
				variant="ghost"
				size="sm"
				class="text-destructive hover:text-destructive"
				data-testid="breakpoint-remove"
				onclick={() => map.panels.removeBreakpoint(bp.key)}
			>
				<TrashIcon />
				Remove
			</Button>
		{/if}
	</div>
</div>
