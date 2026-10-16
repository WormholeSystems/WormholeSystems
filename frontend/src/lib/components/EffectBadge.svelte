<script lang="ts">
	// The wormhole effect badge: a lettered colored circle that opens a popover listing every
	// modifier at this system's class. Modifiers are fetched on open, not on mount, or a list of
	// wormholes would fire one request per row for a table nobody asked to see.
	import ArrowDownIcon from '@lucide/svelte/icons/arrow-down';
	import ArrowUpIcon from '@lucide/svelte/icons/arrow-up';

	import { createQuery } from '@tanstack/svelte-query';

	import { browser } from '$app/environment';
	import { q } from '$lib/api/queries';
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
		<Popover.Content
			class="w-56 p-2 text-[11px]"
			onpointerdown={(ev: PointerEvent) => ev.stopPropagation()}
		>
			<div class="mb-1 flex items-center gap-1.5 font-medium">
				<span
					class="flex size-[14px] items-center justify-center rounded-full text-[9px] font-semibold {darkText
						? 'text-neutral-950'
						: 'text-white'}"
					style:background-color="var(--color-{token})"
				>
					{letter}
				</span>
				{name}
			</div>
			{#each mods as m (m.stat + m.kind)}
				<div class="flex items-center justify-between gap-2">
					<span class="text-muted-foreground">{m.stat}</span>
					<span class="flex items-center gap-0.5">
						{m.value}
						{#if m.kind === 'buff'}
							<ArrowUpIcon class="size-3 text-green-500" />
						{:else}
							<ArrowDownIcon class="size-3 text-red-500" />
						{/if}
					</span>
				</div>
			{/each}
		</Popover.Content>
	</Popover.Root>
{/if}
