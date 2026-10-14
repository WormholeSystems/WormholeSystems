<script lang="ts">
	// Everything that acts on the layout as a whole; per-tile controls live on the tiles.
	import { copyText, readText } from '$lib/clipboard';

	import ClipboardCopyIcon from '@lucide/svelte/icons/clipboard-copy';
	import ClipboardPasteIcon from '@lucide/svelte/icons/clipboard-paste';
	import LaptopIcon from '@lucide/svelte/icons/laptop';
	import LayoutGridIcon from '@lucide/svelte/icons/layout-grid';
	import MonitorIcon from '@lucide/svelte/icons/monitor';
	import MoreHorizontalIcon from '@lucide/svelte/icons/more-horizontal';
	import PlusIcon from '@lucide/svelte/icons/plus';
	import RotateCcwIcon from '@lucide/svelte/icons/rotate-ccw';
	import SaveIcon from '@lucide/svelte/icons/save';
	import SmartphoneIcon from '@lucide/svelte/icons/smartphone';
	import TabletIcon from '@lucide/svelte/icons/tablet';
	import XIcon from '@lucide/svelte/icons/x';

	import { Badge } from '$lib/components/ui/badge';
	import { Button } from '$lib/components/ui/button';
	import * as Dialog from '$lib/components/ui/dialog';
	import * as DropdownMenu from '$lib/components/ui/dropdown-menu';
	import * as Popover from '$lib/components/ui/popover';
	import { Separator } from '$lib/components/ui/separator';
	import * as Tooltip from '$lib/components/ui/tooltip';
	import { cn } from '$lib/utils';
	import { decodeLayout, encodeLayout } from '../../state/layout/layout-codec';
	import {
		BREAKPOINTS,
		PANELS,
		type BreakpointKey,
		resolveLayouts,
	} from '../../state/layout/registry';
	import type { MapState } from '../../state/map-state.svelte';

	let { map }: { map: MapState } = $props();

	let pasteError = $state('');

	const hidden = $derived(map.userSettings?.hidden_panels ?? []);
	const hideable = $derived(PANELS.filter((p) => p.removable));

	const ICONS = {
		xs: SmartphoneIcon,
		sm: TabletIcon,
		md: LaptopIcon,
		lg: MonitorIcon,
	} as const;

	async function copy() {
		// The clipboard is the whole result here, so a refused write has to say so rather
		// than look like nothing happened.
		await copyText(
			encodeLayout({
				breakpoints: resolveLayouts(map.panels.draft),
				hidden: map.userSettings?.hidden_panels ?? [],
			}),
			{ success: 'Layout copied' },
		);
	}

	async function paste() {
		pasteError = '';
		const text = await readText();
		if (text === null) return;
		const data = decodeLayout(text);
		if (data === null) {
			pasteError = 'That does not look like a layout.';
			return;
		}
		map.panels.set(resolveLayouts(data.breakpoints));
		if (data.hidden) {
			const hidden = data.hidden;
			map.patchSettingsLocal((s) => ({ ...s, hidden_panels: hidden }));
		}
	}
</script>

<Tooltip.Provider delayDuration={300}>
	<div class="fixed bottom-6 left-1/2 z-50 -translate-x-1/2" data-testid="layout-toolbar">
		<div
			class="flex items-center gap-2 rounded-2xl border border-border/60 bg-card/95 p-1.5 shadow-xl backdrop-blur-md"
		>
			<Tooltip.Root>
				<Tooltip.Trigger>
					{#snippet child({ props })}
						<Button
							{...props}
							variant="ghost"
							size="icon"
							class={cn('size-9 rounded-xl', map.panels.dirty && 'text-destructive')}
							data-testid="layout-exit"
							onclick={() => map.panels.exitEdit()}
						>
							<XIcon />
						</Button>
					{/snippet}
				</Tooltip.Trigger>
				<Tooltip.Content>
					{map.panels.dirty ? 'Exit (unsaved changes)' : 'Done arranging'}
				</Tooltip.Content>
			</Tooltip.Root>

			<Separator orientation="vertical" class="h-7" />

			<!-- Each breakpoint keeps its own arrangement, so you edit them one at a time. -->
			<div class="flex items-center gap-0.5 rounded-xl border border-border/60 bg-muted/40 p-0.5">
				{#each BREAKPOINTS as bp (bp.key)}
					{@const Icon = ICONS[bp.key]}
					{@const active = map.panels.breakpoint === bp.key}
					<Tooltip.Root>
						<Tooltip.Trigger>
							{#snippet child({ props })}
								<button
									{...props}
									type="button"
									class={cn(
										'flex h-8 items-center gap-1.5 rounded-lg px-2 text-xs transition-colors',
										active
											? 'bg-background font-medium text-foreground shadow-sm'
											: 'text-muted-foreground hover:text-foreground',
									)}
									data-testid="breakpoint-{bp.key}"
									aria-pressed={active}
									onclick={() => (map.panels.breakpoint = bp.key as BreakpointKey)}
								>
									<Icon class="size-4 shrink-0" />
									{#if active}<span class="whitespace-nowrap">{bp.label}</span>{/if}
								</button>
							{/snippet}
						</Tooltip.Trigger>
						<Tooltip.Content>
							<p class="font-medium">{bp.label}</p>
							<p class="text-xs text-muted-foreground">
								{bp.minWidth}px and wider
							</p>
						</Tooltip.Content>
					</Tooltip.Root>
				{/each}
			</div>

			<Separator orientation="vertical" class="h-7" />

			<Popover.Root>
				<Popover.Trigger>
					{#snippet child({ props })}
						<Button
							{...props}
							variant="ghost"
							size="icon"
							class="relative size-9 rounded-xl"
							data-testid="card-library"
						>
							<LayoutGridIcon />
							{#if hidden.length > 0}
								<Badge
									class="absolute -top-1 -right-1 size-4 justify-center rounded-full p-0 text-[10px] tabular-nums"
								>
									{hidden.length}
								</Badge>
							{/if}
						</Button>
					{/snippet}
				</Popover.Trigger>
				<Popover.Content side="top" align="center" class="w-80 p-0">
					<div class="border-b px-3 py-2.5">
						<p class="text-sm font-medium">Panels</p>
						<p class="text-xs text-muted-foreground">Add one back to the layout.</p>
					</div>
					<div class="max-h-80 overflow-y-auto p-1.5">
						{#each hideable as panel (panel.id)}
							{@const isHidden = hidden.includes(panel.id)}
							{#if isHidden}
								<button
									type="button"
									class="flex w-full items-start gap-3 rounded-lg p-2 text-left transition-colors hover:bg-muted"
									data-testid="add-{panel.id}"
									onclick={() => map.panels.showPanel(panel.id)}
								>
									<span class="min-w-0 flex-1">
										<span class="block text-sm font-medium">{panel.label}</span>
										<span class="block text-xs text-muted-foreground">{panel.description}</span>
									</span>
									<PlusIcon class="mt-0.5 size-4 shrink-0 text-muted-foreground" />
								</button>
							{:else}
								<div class="flex items-start gap-3 rounded-lg p-2 opacity-50">
									<span class="min-w-0 flex-1">
										<span class="block text-sm font-medium">{panel.label}</span>
										<span class="block text-xs text-muted-foreground">{panel.description}</span>
									</span>
									<span
										class="mt-0.5 font-mono text-[10px] tracking-wider text-muted-foreground uppercase"
									>
										On layout
									</span>
								</div>
							{/if}
						{/each}
					</div>
				</Popover.Content>
			</Popover.Root>

			<Separator orientation="vertical" class="h-7" />

			<div class="flex items-center gap-1">
				<Button
					class="h-9 gap-1.5 rounded-xl"
					disabled={!map.panels.dirty}
					data-testid="layout-save"
					onclick={() => map.panels.save()}
				>
					<SaveIcon />
					Save
				</Button>

				<DropdownMenu.Root>
					<DropdownMenu.Trigger>
						{#snippet child({ props })}
							<Button
								{...props}
								variant="ghost"
								size="icon"
								class="size-9 rounded-xl"
								data-testid="layout-more"
							>
								<MoreHorizontalIcon />
							</Button>
						{/snippet}
					</DropdownMenu.Trigger>
					<DropdownMenu.Content side="top" align="end" class="w-52">
						<DropdownMenu.Group>
							<DropdownMenu.Item
								data-testid="layout-reset"
								onSelect={() => map.panels.reset(map.panels.breakpoint)}
							>
								<RotateCcwIcon />
								Reset this size
							</DropdownMenu.Item>
							<DropdownMenu.Separator />
							<DropdownMenu.Item data-testid="layout-copy" onSelect={copy}>
								<ClipboardCopyIcon />
								Copy layout
							</DropdownMenu.Item>
							<DropdownMenu.Item data-testid="layout-paste" onSelect={paste}>
								<ClipboardPasteIcon />
								Paste layout
							</DropdownMenu.Item>
						</DropdownMenu.Group>
					</DropdownMenu.Content>
				</DropdownMenu.Root>
			</div>
		</div>
		{#if pasteError}
			<p class="mt-2 text-center text-xs text-destructive" data-testid="layout-paste-error">
				{pasteError}
			</p>
		{/if}
	</div>
</Tooltip.Provider>

<Dialog.Root bind:open={map.panels.exitPrompt}>
	<Dialog.Content class="sm:max-w-md" data-testid="layout-exit-prompt">
		<Dialog.Header>
			<Dialog.Title>Save your layout changes?</Dialog.Title>
			<Dialog.Description>
				You have rearranged panels without saving. Discarding puts the arrangement back to the last
				one you saved.
			</Dialog.Description>
		</Dialog.Header>
		<Dialog.Footer class="sm:justify-between">
			<Button variant="ghost" onclick={() => (map.panels.exitPrompt = false)}>Keep editing</Button>
			<div class="flex gap-2">
				<Button
					variant="outline"
					data-testid="layout-discard"
					onclick={() => map.panels.resolveExit(false)}>Discard</Button
				>
				<Button data-testid="layout-save-exit" onclick={() => map.panels.resolveExit(true)}>
					Save
				</Button>
			</div>
		</Dialog.Footer>
	</Dialog.Content>
</Dialog.Root>
