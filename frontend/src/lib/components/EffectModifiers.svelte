<script lang="ts">
	// A wormhole effect's buffs and debuffs, each with the game client's icon for the stat,
	// so the list reads at a glance. Shared by the effect badge's popover and the system's
	// info card.
	import type { EffectModifier } from '$lib/api/types/EffectModifier';
	import { statIcon } from '$lib/map/effect-icons';
	import { cn } from '$lib/utils';
	import Hint from '$lib/components/Hint.svelte';

	let {
		modifiers,
		columns = 1,
		loading = false,
	}: {
		modifiers: EffectModifier[];
		/** Two side by side where the panel is wide enough to spare them. */
		columns?: 1 | 2;
		loading?: boolean;
	} = $props();

	const groups = $derived(
		[
			{ kind: 'buff', label: 'Buffs', rows: modifiers.filter((m) => m.kind === 'buff') },
			{ kind: 'debuff', label: 'Debuffs', rows: modifiers.filter((m) => m.kind !== 'buff') },
		].filter((g) => g.rows.length > 0),
	);
</script>

{#if loading && modifiers.length === 0}
	<p class="text-[11px] text-muted-foreground">Loading…</p>
{:else}
	<div class="flex flex-col gap-2" data-testid="effect-modifiers">
		{#each groups as group (group.kind)}
			<div class="flex flex-col gap-0.5">
				<span class="text-[10px] tracking-wider text-muted-foreground uppercase">
					{group.label}
				</span>
				<div class={cn('grid gap-x-4 gap-y-0.5', columns === 2 && 'grid-cols-2')}>
					{#each group.rows as m (m.stat + m.kind)}
						{@const icon = statIcon(m.stat)}
						<div class="flex min-w-0 items-center gap-2 text-[11px]" data-testid="effect-modifier">
							<!-- The client's icons are drawn for a dark background, so they keep one in
							     the light theme too. -->
							<span
								class="flex size-5 shrink-0 items-center justify-center rounded-sm bg-neutral-900"
							>
								{#if icon}
									<img src={icon} alt="" class="size-5" loading="lazy" />
								{/if}
							</span>
							<Hint text={m.stat} clippedOnly focusable={false}>
								{#snippet child({ props })}
									<span class="min-w-0 flex-1 truncate" {...props}>{m.stat}</span>
								{/snippet}
							</Hint>
							<span
								class="shrink-0 font-medium tabular-nums {m.kind === 'buff'
									? 'text-green-500'
									: 'text-red-500'}"
							>
								{m.value}
							</span>
						</div>
					{/each}
				</div>
			</div>
		{/each}
	</div>
{/if}
