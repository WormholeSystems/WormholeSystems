<script lang="ts">
	// A screen size the built-in four don't cover, say an ultrawide. It starts from the
	// arrangement of the size closest to it and becomes the one being arranged.
	import PlusIcon from '@lucide/svelte/icons/plus';

	import { Button } from '$lib/components/ui/button';
	import { Input } from '$lib/components/ui/input';
	import * as Popover from '$lib/components/ui/popover';
	import * as Tooltip from '$lib/components/ui/tooltip';
	import {
		MAX_BREAKPOINTS,
		startingColumns,
		type BreakpointMeta,
	} from '../../state/layout/registry';
	import type { MapState } from '../../state/map-state.svelte';

	let { map, breakpoints }: { map: MapState; breakpoints: BreakpointMeta[] } = $props();

	let open = $state(false);
	let label = $state('');
	let width = $state<number | null>(null);

	const full = $derived(breakpoints.length >= MAX_BREAKPOINTS);
	const error = $derived.by(() => {
		if (width === null) return null;
		if (!Number.isInteger(width) || width < 1 || width > 10_000)
			return 'Pick a width between 1 and 10000';
		if (breakpoints.some((bp) => bp.minWidth === width)) return 'Another size already starts there';
		return null;
	});
	const canAdd = $derived(!full && label.trim().length > 0 && width !== null && error === null);

	function add() {
		if (!canAdd || width === null) return;
		map.panels.addBreakpoint({
			label,
			minWidth: width,
			cols: startingColumns(width),
			rowHeight: 100,
		});
		label = '';
		width = null;
		open = false;
	}
</script>

<Popover.Root bind:open>
	<Tooltip.Root>
		<Tooltip.Trigger>
			{#snippet child({ props: tooltipProps })}
				<Popover.Trigger {...tooltipProps}>
					{#snippet child({ props })}
						<button
							{...props}
							type="button"
							class="flex size-8 items-center justify-center rounded-lg text-muted-foreground transition-colors hover:text-foreground disabled:opacity-40"
							aria-label="Add a screen size"
							disabled={full}
							data-testid="breakpoint-add"
						>
							<PlusIcon class="size-4" />
						</button>
					{/snippet}
				</Popover.Trigger>
			{/snippet}
		</Tooltip.Trigger>
		<Tooltip.Content>
			{full ? `At most ${MAX_BREAKPOINTS} sizes` : 'Add a screen size'}
		</Tooltip.Content>
	</Tooltip.Root>
	<Popover.Content side="top" align="center" class="w-72 p-0">
		<form
			class="flex flex-col gap-3 p-3"
			onsubmit={(e) => {
				e.preventDefault();
				add();
			}}
		>
			<p class="text-sm font-medium">Add a screen size</p>
			<div class="flex gap-2">
				<label class="flex min-w-0 flex-1 flex-col gap-1">
					<span class="text-[11px] text-muted-foreground">Name</span>
					<Input
						bind:value={label}
						placeholder="Ultrawide"
						maxlength={32}
						class="h-8 text-xs"
						data-testid="breakpoint-new-label"
					/>
				</label>
				<label class="flex w-24 flex-col gap-1">
					<span class="text-[11px] text-muted-foreground">From (px)</span>
					<Input
						type="number"
						bind:value={width}
						placeholder="2560"
						min={1}
						max={10000}
						class="h-8 text-xs"
						data-testid="breakpoint-new-width"
					/>
				</label>
			</div>
			{#if error}
				<p class="text-[11px] text-destructive">{error}</p>
			{:else if width}
				<p class="text-[11px] text-muted-foreground">
					Starts with {startingColumns(width)} columns; you can change that after.
				</p>
			{/if}
			<Button type="submit" size="sm" disabled={!canAdd} data-testid="breakpoint-new-add">
				Add
			</Button>
		</form>
	</Popover.Content>
</Popover.Root>
