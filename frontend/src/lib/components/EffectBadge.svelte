<script lang="ts">
	// The wormhole effect badge: the effect's own icon from the game's HUD (a lettered circle
	// in its color for one without), opening a popover listing every modifier at this
	// system's class. Modifiers are fetched on open, not on mount, or a list of
	// wormholes would fire one request per row for a table nobody asked to see.
	import { createQuery } from '@tanstack/svelte-query';

	import { browser } from '$app/environment';
	import { page } from '$app/state';
	import { q } from '$lib/api/queries';
	import EffectModifiers from '$lib/components/EffectModifiers.svelte';
	import { effectIcon } from '$lib/map/effect-icons';
	import * as Popover from '$lib/components/ui/popover';
	import * as Tooltip from '$lib/components/ui/tooltip';

	let {
		name,
		wormholeClassId,
		/**
		 * Whether clicking opens the modifier table. Off in list rows, whose own click wins,
		 * and it shows its name in a tooltip instead.
		 */
		detail = true,
	}: { name: string; wormholeClassId: number; detail?: boolean } = $props();

	const token = $derived(name.toLowerCase().replaceAll(' ', '-'));
	const letter = $derived(name === 'Wolf-Rayet Star' ? 'W' : name.charAt(0).toUpperCase());
	const darkText = $derived(name === 'Cataclysmic Variable');
	// Letters instead of pictures for whoever turned the icons off in their settings.
	const icon = $derived((page.data.preferences?.effect_icons ?? true) ? effectIcon(name) : null);

	let open = $state(false);

	const modsQuery = createQuery(() => ({
		...q.effectModifiers(name, wormholeClassId),
		enabled: browser && open,
	}));
	const mods = $derived(modsQuery.data ?? []);
	// Drifter and Thera classes have no effect strength of their own to show.
	const classLabel = $derived(
		wormholeClassId >= 1 && wormholeClassId <= 6 ? `Class ${wormholeClassId}` : null,
	);
</script>

{#snippet circle(size: number)}
	<!-- The HUD's icons glow on black, so they keep a dark disc in the light theme too. -->
	{#if icon}
		<span
			class="flex shrink-0 items-center justify-center overflow-hidden rounded-full bg-neutral-950"
			style:width="{size}px"
			style:height="{size}px"
		>
			<img src={icon} alt="" class="size-full scale-125" />
		</span>
	{:else}
		<span
			class="flex shrink-0 items-center justify-center rounded-full font-semibold {darkText
				? 'text-neutral-950'
				: 'text-white'}"
			style:width="{size}px"
			style:height="{size}px"
			style:font-size="{Math.round(size * 0.6)}px"
			style="background-color: var(--color-{token})"
		>
			{letter}
		</span>
	{/if}
{/snippet}

{#if !detail}
	<Tooltip.Root>
		<Tooltip.Trigger aria-label={name} class="flex">
			{@render circle(16)}
		</Tooltip.Trigger>
		<Tooltip.Content>{name}</Tooltip.Content>
	</Tooltip.Root>
{:else}
	<Popover.Root bind:open>
		<Popover.Trigger
			aria-label={name}
			class="flex shrink-0 cursor-pointer rounded-full"
			onpointerdown={(ev: PointerEvent) => ev.stopPropagation()}
		>
			{@render circle(16)}
		</Popover.Trigger>
		<Popover.Content class="w-64 p-0" onpointerdown={(ev: PointerEvent) => ev.stopPropagation()}>
			<div class="flex items-center gap-2 border-b border-border px-3 py-2">
				{@render circle(24)}
				<span class="font-heading text-sm font-semibold">{name}</span>
				{#if classLabel}
					<span class="ml-auto text-[10px] text-muted-foreground">{classLabel}</span>
				{/if}
			</div>
			<div class="px-3 py-2">
				<EffectModifiers modifiers={mods} loading={modsQuery.isPending} />
			</div>
		</Popover.Content>
	</Popover.Root>
{/if}
