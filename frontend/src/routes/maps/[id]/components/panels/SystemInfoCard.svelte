<script lang="ts">
	import { createQuery } from '@tanstack/svelte-query';

	import { q } from '$lib/api/queries';
	import type { MapSystemView } from '$lib/api/types/MapSystemView';
	import * as Popover from '$lib/components/ui/popover';
	import EffectModifiers from '$lib/components/EffectModifiers.svelte';
	import EveImage from '$lib/components/EveImage.svelte';
	import MapPanel from '$lib/components/map-panel/MapPanel.svelte';
	import MapPanelContent from '$lib/components/map-panel/MapPanelContent.svelte';
	import MapPanelHeader from '$lib/components/map-panel/MapPanelHeader.svelte';
	import StaticDetails from '$lib/components/StaticDetails.svelte';
	import ClassBadge from '$lib/components/ClassBadge.svelte';
	import { destClassMeta, effectTextColor, isWormholeClass } from '$lib/map/classes';
	import {
		dotlanRegionMapUrl,
		dotlanSystemUrl,
		zkillboardSystemUrl,
	} from '$lib/map/external-links';

	let { system }: { system: MapSystemView } = $props();

	// Everything past the node's own name is looked up from a system it may not be yet.
	const mapped = $derived(system.kind === 'system' ? system : null);

	const isWormhole = $derived(isWormholeClass(mapped?.wormhole_class_id ?? null));

	const effectColor = $derived(effectTextColor(mapped?.effect_name ?? null));

	const modsQuery = createQuery(() => ({
		...q.effectModifiers(mapped?.effect_name ?? '', mapped?.wormhole_class_id ?? 0),
		enabled: Boolean(mapped?.effect_name),
	}));
	const mods = $derived(mapped?.effect_name ? (modsQuery.data ?? []) : []);

	const dotlanUrl = $derived.by(() => {
		if (!mapped) return null;
		return isWormhole
			? dotlanSystemUrl(mapped.name)
			: dotlanRegionMapUrl(mapped.region, mapped.name);
	});
</script>

<MapPanel testid="system-info">
	<MapPanelHeader>System</MapPanelHeader>
	<MapPanelContent>
		<div class="border-b border-border/50 px-3 py-3">
			<div class="flex items-center gap-2">
				<ClassBadge
					classId={mapped?.wormhole_class_id ?? null}
					security={mapped?.security_status ?? null}
				/>
				<span class="truncate text-sm font-medium">
					{#if system.alias && mapped}
						{system.alias} <span class="text-muted-foreground">({mapped.name})</span>
					{:else if mapped}
						{mapped.name}
					{:else if system.alias}
						{system.alias} <span class="text-muted-foreground">(unmapped)</span>
					{:else}
						<span class="text-muted-foreground">Unmapped system</span>
					{/if}
				</span>
				{#if mapped?.effect_name}
					<span class="shrink-0 text-[10px] {effectColor}">{mapped.effect_name}</span>
				{/if}
				{#if mapped?.is_shattered}
					<span class="shrink-0 text-[10px] text-amber-500">Shattered</span>
				{/if}
			</div>
			{#if mapped?.occupying_group}
				<div class="mt-1 text-[11px] break-words text-muted-foreground">
					Occupied by <span class="font-medium text-foreground">{mapped.occupying_group}</span>
				</div>
			{/if}
			{#if !mapped}
				<div class="mt-1 text-[11px] text-muted-foreground">
					Nobody has been through this hole yet. Assign a system from the node's menu once someone
					has.
				</div>
			{:else}
				<div class="mt-1 flex items-center gap-1 text-[11px] text-muted-foreground">
					<span>{mapped.region}</span>
					<span>· {mapped.constellation}</span>
					<span class="text-border">·</span>
					<a
						class="transition-colors hover:text-foreground"
						href={zkillboardSystemUrl(mapped.solar_system_id)}
						target="_blank"
						rel="noopener">zKill</a
					>
					<a
						class="transition-colors hover:text-foreground"
						href={dotlanUrl}
						target="_blank"
						rel="noopener">Dotlan</a
					>
					{#if isWormhole}
						<a
							class="transition-colors hover:text-foreground"
							href="https://anoik.is/systems/{mapped.name}"
							target="_blank"
							rel="noopener">Anoik</a
						>
					{/if}
				</div>
			{/if}
		</div>

		{#if mapped && mapped.statics.length > 0}
			<div class="border-b border-border/50 px-3 py-2">
				<div class="flex items-center gap-2">
					<span class="text-[10px] tracking-wider text-muted-foreground uppercase">Statics</span>
					<div class="flex gap-1.5">
						{#each mapped.statics as st (st.code)}
							{@const dest = destClassMeta(st.dest_class)}
							<Popover.Root>
								<Popover.Trigger
									class="font-mono text-xs font-medium transition-opacity hover:opacity-70"
									style="color: var(--color-{dest.token})"
								>
									{st.code} <span class="uppercase opacity-60">{dest.short}</span>
								</Popover.Trigger>
								<Popover.Content class="w-48 p-0" align="start">
									<StaticDetails static={st} />
								</Popover.Content>
							</Popover.Root>
						{/each}
					</div>
				</div>
			</div>
		{/if}

		{#if mods.length > 0}
			<div class="border-b border-border/50 px-3 py-2">
				<div class="flex flex-col gap-1">
					<span class="text-[10px] tracking-wider text-muted-foreground uppercase"> Effect </span>
					<EffectModifiers modifiers={mods} columns={2} />
				</div>
			</div>
		{/if}

		{#if mapped?.sovereignty}
			<div class="px-3 py-2">
				<div class="flex flex-col gap-1.5">
					<span class="text-[10px] tracking-wider text-muted-foreground uppercase">Sovereignty</span
					>
					<div class="flex items-center gap-2">
						<EveImage
							kind={mapped.sovereignty.kind}
							id={mapped.sovereignty.id}
							size={32}
							class="size-5 rounded"
						/>
						<span class="text-xs">
							{#if 'ticker' in mapped.sovereignty}<span class="font-medium"
									>[{mapped.sovereignty.ticker}]</span
								>
							{/if}{mapped.sovereignty.name}
						</span>
					</div>
				</div>
			</div>
		{/if}
	</MapPanelContent>
</MapPanel>
