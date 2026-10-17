<script lang="ts">
	// The undo tree as a popover: newest first, rails and forks drawn per row, with the map's
	// current position marked and scrolled into view.
	import HistoryIcon from '@lucide/svelte/icons/history';

	import { Button } from '$lib/components/ui/button';
	import * as Popover from '$lib/components/ui/popover';
	import { historyRows } from '$lib/map/history-tree';
	import { timeAgo } from '$lib/format';
	import { cn } from '$lib/utils';
	import type { MapState } from '../../state/map-state.svelte';
	import Hint from '$lib/components/Hint.svelte';

	let { map }: { map: MapState } = $props();

	const canWrite = $derived(map.canWrite);
	const rows = $derived(historyRows(map.history.entries));

	// The trunk runs oldest-first, so the map's position is near the bottom of a long history.
	// Binding the marker fires this once the popover's rows are in the DOM, with no timer.
	let headLabel = $state<HTMLElement | null>(null);
	$effect(() => {
		headLabel?.scrollIntoView({ block: 'nearest' });
	});
</script>

<Popover.Root>
	<Popover.Trigger>
		{#snippet child({ props })}
			<Button {...props} variant="ghost" size="icon" class="size-7" data-testid="history-button">
				<HistoryIcon />
			</Button>
		{/snippet}
	</Popover.Trigger>
	<Popover.Content class="w-96 p-0" align="end">
		<div class="border-b border-border/50 px-3 py-2 text-xs font-medium">
			History
			<span class="ml-1 font-normal text-muted-foreground">newest first</span>
		</div>
		{#if rows.length === 0}
			<p class="px-3 py-6 text-center text-xs text-muted-foreground">Nothing yet.</p>
		{:else}
			<ul class="max-h-80 overflow-y-auto py-1" data-testid="history-list">
				{#each rows as row (row.entry.id)}
					{@const entry = row.entry}
					{@const isHead = entry.id === map.history.data?.head_event_id}
					{@const navigable = entry.is_step && canWrite && !isHead}
					<li>
						<Hint
							text={entry.is_step
								? isHead
									? 'The map is here'
									: entry.applied
										? 'Rewind the map to this point'
										: 'Return to this branch'
								: 'Recorded automatically; not part of undo'}
						>
							{#snippet child({ props })}
								<button
									type="button"
									class={cn(
										'flex w-full items-stretch gap-0 text-left text-xs',
										navigable && 'hover:bg-accent',
										!navigable && 'cursor-default',
										isHead && 'bg-accent/60',
									)}
									data-testid="history-row"
									data-applied={entry.applied}
									data-depth={row.depth}
									data-forks={row.forks}
									data-head={isHead}
									disabled={!navigable}
									{...props}
									onclick={() => map.history.gotoEvent(entry.id)}
								>
									<!-- A rail for each line still open above this row, then this row's own
						     dot. Every line is centred in a 16px cell, so a branch's connector
						     meets the rail it left exactly. -->
									{#each row.rails as passing, i (i)}
										<span class="relative w-4 shrink-0">
											{#if passing}
												<span
													class="absolute inset-y-0 left-1/2 w-px -translate-x-1/2 bg-foreground/25"
												></span>
											{/if}
										</span>
									{/each}
									<span class="relative w-4 shrink-0">
										{#if row.railUp}
											<span
												class="absolute top-0 bottom-1/2 left-1/2 w-px -translate-x-1/2 bg-foreground/25"
											></span>
										{/if}
										{#if row.railDown}
											<span
												class="absolute top-1/2 bottom-0 left-1/2 w-px -translate-x-1/2 bg-foreground/25"
											></span>
										{/if}
										{#if row.forks}
											<!-- Where this line left the one it branched from. -->
											<span
												class="absolute top-1/2 right-1/2 h-px w-4 -translate-y-1/2 bg-foreground/25"
											></span>
										{/if}
										<span
											class={cn(
												'absolute top-1/2 left-1/2 size-1.5 -translate-x-1/2 -translate-y-1/2 rounded-full ring-2 ring-popover',
												isHead
													? 'bg-amber-400'
													: !entry.is_step
														? 'bg-transparent ring-0'
														: entry.applied
															? 'bg-foreground/60'
															: 'bg-muted-foreground/40',
											)}
										></span>
									</span>
									<span class="flex flex-1 items-baseline gap-2 py-1.5 pr-3 min-w-0">
										<span
											class={cn(
												'flex-1 truncate',
												!entry.is_step && 'text-muted-foreground italic',
												entry.is_step && !entry.applied && 'text-muted-foreground',
											)}
										>
											<span class="text-muted-foreground"
												>{entry.character_name ?? 'WormholeSystems'}</span
											>
											{entry.label}
										</span>
										{#if isHead}
											<span
												bind:this={headLabel}
												class="shrink-0 font-mono text-[10px] tracking-wider text-amber-400 uppercase"
											>
												here
											</span>
										{:else}
											<span class="shrink-0 text-muted-foreground">{timeAgo(entry.created_at)}</span
											>
										{/if}
									</span>
								</button>
							{/snippet}
						</Hint>
					</li>
				{/each}
			</ul>
			{#if canWrite && map.history.data?.head_event_id != null}
				<div class="border-t border-border/50 p-2">
					<Button
						variant="ghost"
						size="sm"
						class="w-full text-xs"
						data-testid="history-rewind"
						onclick={() => map.history.gotoEvent(null)}
					>
						Rewind to the start
					</Button>
				</div>
			{/if}
		{/if}
	</Popover.Content>
</Popover.Root>
