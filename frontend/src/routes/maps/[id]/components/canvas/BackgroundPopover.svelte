<script lang="ts">
	// The picture behind this viewer's map, set from the map itself: drop or pick a file,
	// see it, swap it, take it away, and say whether it moves with the systems or stays
	// put. The same setting the display page edits, reached without leaving the chain.
	import ImageIcon from '@lucide/svelte/icons/image';
	import ImageUpIcon from '@lucide/svelte/icons/image-up';
	import LoaderCircleIcon from '@lucide/svelte/icons/loader-circle';
	import Trash2Icon from '@lucide/svelte/icons/trash-2';

	import type { BackgroundMode } from '$lib/api/types/BackgroundMode';
	import { Button } from '$lib/components/ui/button';
	import * as Popover from '$lib/components/ui/popover';
	import * as Tooltip from '$lib/components/ui/tooltip';
	import { ACCEPTED_IMAGE_TYPES, BACKGROUND_MODES, takePickedFile } from '$lib/map/background';
	import { cn } from '$lib/utils';
	import type { MapState } from '../../state/map-state.svelte';
	import Hint from '$lib/components/Hint.svelte';

	let { map }: { map: MapState } = $props();

	// A guest on a share link has no settings row to keep a picture on.
	const canUpload = $derived(map.userSettings !== null);
	const image = $derived(map.userSettings?.background_image_url ?? null);
	const mode = $derived(map.userSettings?.background_image_mode ?? 'grid');

	let fileInput = $state<HTMLInputElement | null>(null);
	let dragging = $state(false);

	function onFilePicked(event: Event) {
		const file = takePickedFile(event);
		if (file) map.background.upload(file);
	}

	function onDrop(event: DragEvent) {
		event.preventDefault();
		dragging = false;
		if (map.background.pending) return;
		const file = event.dataTransfer?.files?.[0];
		if (file) map.background.upload(file);
	}

	function setMode(next: BackgroundMode) {
		if (next !== mode) map.patchUserSettings({ background_image_mode: next });
	}
</script>

<Popover.Root>
	<Hint text="Background image">
		{#snippet child({ props: hint })}
			<Popover.Trigger {...hint}>
				{#snippet child({ props })}
					<button
						{...props}
						class={cn(
							'flex items-center px-2.5 py-1 hover:bg-accent hover:text-foreground',
							image ? 'text-foreground' : 'text-muted-foreground',
						)}
						aria-label="Background image"
						data-testid="background-button"
					>
						<ImageIcon class="size-4" />
					</button>
				{/snippet}
			</Popover.Trigger>
		{/snippet}
	</Hint>
	<Popover.Content class="w-72" side="top" align="end" data-testid="background-popover">
		<div class="flex flex-col gap-3">
			<div class="flex items-center justify-between">
				<span class="text-xs font-medium">Background</span>
				{#if image}
					<Button
						variant="ghost"
						size="sm"
						class="h-6 gap-1 px-2 text-xs text-muted-foreground hover:text-destructive"
						disabled={map.background.pending}
						onclick={() => map.background.remove()}
						data-testid="background-remove"
					>
						<Trash2Icon class="size-3.5" />
						Remove
					</Button>
				{/if}
			</div>

			{#if canUpload}
				<input
					bind:this={fileInput}
					type="file"
					accept={ACCEPTED_IMAGE_TYPES}
					class="hidden"
					data-testid="background-input"
					onchange={onFilePicked}
				/>
				<!-- svelte-ignore a11y_no_static_element_interactions -->
				<div
					class={cn(
						'group relative flex aspect-video cursor-pointer items-center justify-center overflow-hidden border border-dashed transition-colors',
						dragging
							? 'border-foreground bg-accent'
							: 'border-border hover:border-foreground/40 hover:bg-muted/40',
					)}
					role="button"
					tabindex="0"
					aria-label={image ? 'Replace background image' : 'Choose background image'}
					data-testid="background-dropzone"
					onclick={() => !map.background.pending && fileInput?.click()}
					onkeydown={(e) => e.key === 'Enter' && fileInput?.click()}
					ondragover={(e) => {
						e.preventDefault();
						dragging = true;
					}}
					ondragleave={() => (dragging = false)}
					ondrop={onDrop}
				>
					{#if image}
						<img
							src={image}
							alt=""
							class="absolute inset-0 size-full object-cover"
							data-testid="background-preview"
						/>
						<div
							class="absolute inset-0 flex flex-col items-center justify-center gap-1 bg-black/55 text-white opacity-0 transition-opacity group-hover:opacity-100"
						>
							<ImageUpIcon class="size-5" />
							<span class="text-xs font-medium">Replace image</span>
						</div>
					{:else}
						<div class="flex flex-col items-center gap-1.5 text-muted-foreground">
							<ImageUpIcon class="size-5" />
							<span class="text-xs font-medium text-foreground">Drop a file or click</span>
							<span class="text-[10px]">PNG, JPG, GIF or WebP, up to 8 MiB</span>
						</div>
					{/if}
					{#if map.background.pending}
						<div class="absolute inset-0 flex items-center justify-center bg-background/70">
							<LoaderCircleIcon class="size-5 animate-spin" />
						</div>
					{/if}
				</div>
			{:else}
				<p
					class="border border-dashed border-border px-3 py-4 text-center text-xs text-muted-foreground"
				>
					Sign in to put a picture behind this map.
				</p>
			{/if}

			{#if image}
				<div class="flex flex-col gap-1.5">
					<span class="text-[11px] font-medium text-muted-foreground">Position</span>
					<div class="grid grid-cols-2 gap-px bg-border" data-testid="background-modes">
						{#each BACKGROUND_MODES as option (option.value)}
							<Tooltip.Root>
								<Tooltip.Trigger>
									{#snippet child({ props })}
										<button
											{...props}
											type="button"
											class={cn(
												'px-2 py-1.5 text-xs transition-colors',
												mode === option.value
													? 'bg-accent text-foreground'
													: 'bg-card text-muted-foreground hover:text-foreground',
											)}
											aria-pressed={mode === option.value}
											data-testid="background-mode-{option.value}"
											onclick={() => setMode(option.value)}
										>
											{option.label}
										</button>
									{/snippet}
								</Tooltip.Trigger>
								<Tooltip.Content>{option.hint}</Tooltip.Content>
							</Tooltip.Root>
						{/each}
					</div>
				</div>
			{/if}
		</div>
	</Popover.Content>
</Popover.Root>
