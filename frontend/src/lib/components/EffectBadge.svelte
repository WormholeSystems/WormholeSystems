<script lang="ts">
	// The wormhole effect badge: a lettered colored circle that opens a popover listing every
	// modifier at this system's class. Modifiers are fetched on open, not on mount, or a list of
	// wormholes would fire one request per row for a table nobody asked to see.
	import { createQuery } from '@tanstack/svelte-query';

	import { browser } from '$app/environment';
	import { q } from '$lib/api/queries';
	import EffectModifiers from '$lib/components/EffectModifiers.svelte';
	import * as Popover from '$lib/components/ui/popover';
	import * as Tooltip from '$lib/components/ui/tooltip';

	let {
		name,
		wormholeClassId,
		/**
		 * Whether clicking opens the modifier table. Off in list rows, whose own click wins;
		 * the row then has to sit inside a `Tooltip.Provider`.
		 */
		detail = true,
	}: { name: string; wormholeClassId: number; detail?: boolean } = $props();

	const token = $derived(name.toLowerCase().replaceAll(' ', '-'));
	const letter = $derived(name === 'Wolf-Rayet Star' ? 'W' : name.charAt(0).toUpperCase());
	const darkText = $derived(name === 'Cataclysmic Variable');

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

{#snippet circle()}
	<span
		class="flex size-[14px] shrink-0 items-center justify-center rounded-full text-[9px] font-semibold {darkText
			? 'text-neutral-950'
			: 'text-white'}"
		style="background-color: var(--color-{token})"
	>
		{letter}
	</span>
{/snippet}

{#if !detail}
	<!-- The row around it provides the tooltip context, so a long list mounts one provider
	     per row rather than one per badge. -->
	<Tooltip.Root>
		<Tooltip.Trigger aria-label={name} class="flex">
			{@render circle()}
		</Tooltip.Trigger>
		<Tooltip.Content>{name}</Tooltip.Content>
	</Tooltip.Root>
{:else}
	<Popover.Root bind:open>
		<Popover.Trigger
			aria-label={name}
			class="flex size-[14px] shrink-0 cursor-pointer items-center justify-center rounded-full text-[9px] font-semibold {darkText
				? 'text-neutral-950'
				: 'text-white'}"
			style="background-color: var(--color-{token})"
			onpointerdown={(ev: PointerEvent) => ev.stopPropagation()}
		>
			{letter}
		</Popover.Trigger>
		<Popover.Content class="w-64 p-0" onpointerdown={(ev: PointerEvent) => ev.stopPropagation()}>
			<div class="flex items-center gap-2 border-b border-border px-3 py-2">
				<span
					class="flex size-[18px] shrink-0 items-center justify-center rounded-full text-[10px] font-semibold {darkText
						? 'text-neutral-950'
						: 'text-white'}"
					style:background-color="var(--color-{token})"
				>
					{letter}
				</span>
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
