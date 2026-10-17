<script lang="ts">
	import GlobeIcon from '@lucide/svelte/icons/globe';
	import * as Tooltip from '$lib/components/ui/tooltip';
	import MapPinIcon from '@lucide/svelte/icons/map-pin';

	import { createQuery } from '@tanstack/svelte-query';

	import { q } from '$lib/api/queries';
	import type { MapSystemView } from '$lib/api/types/MapSystemView';
	import { Badge } from '$lib/components/ui/badge';
	import MapPanel from '$lib/components/map-panel/MapPanel.svelte';
	import MapPanelContent from '$lib/components/map-panel/MapPanelContent.svelte';
	import MapPanelHeader from '$lib/components/map-panel/MapPanelHeader.svelte';
	import EveImage from '$lib/components/EveImage.svelte';
	import { isWormholeClass } from '$lib/map/classes';
	import { timeAgo } from '$lib/format';

	let { system }: { system: MapSystemView } = $props();

	// Threat comes from killmails in a system, so a hole nobody has been through has none.
	const mapped = $derived(system.kind === 'system' ? system : null);

	// Threat is only tracked in wormhole space, so k-space never even asks.
	const analysisQuery = createQuery(() => ({
		...q.threatAnalysis(mapped?.solar_system_id ?? 0),
		enabled: mapped !== null && isWormholeClass(mapped.wormhole_class_id),
	}));
	const analysis = $derived(analysisQuery.data ?? null);

	const badgeClass = $derived.by(() => {
		switch (analysis?.threat_level) {
			case 'critical':
				return 'text-red-600 bg-red-600/10';
			case 'high':
				return 'text-orange-500 bg-orange-500/10';
			default:
				return 'text-muted-foreground bg-muted/50';
		}
	});
</script>

<MapPanel testid="threat-card">
	<MapPanelHeader>
		Threat Analysis
		{#snippet actions()}
			{#if analysis}
				<Badge variant="secondary" class={badgeClass} data-testid="threat-badge">
					{analysis.threat_level[0].toUpperCase() + analysis.threat_level.slice(1)}
				</Badge>
			{/if}
		{/snippet}
	</MapPanelHeader>
	<MapPanelContent>
		<div class="flex flex-col gap-2 p-3 text-xs">
			{#if !mapped || !isWormholeClass(mapped.wormhole_class_id)}
				<!-- Threat is derived from killmails in wormhole space; k-space has none. -->
				<p class="text-muted-foreground">
					No analysis for this system. Threat is only tracked in wormhole space.
				</p>
			{:else if !analysis}
				<p class="text-muted-foreground">No threat data available for this system.</p>
			{:else if analysis.entities.length === 0}
				<p class="text-muted-foreground">No significant activity detected.</p>
			{:else}
				<div class="flex flex-col gap-1.5">
					{#each analysis.entities as e (e.entity_type + e.id)}
						<div class="flex items-center gap-2">
							<EveImage
								kind={e.entity_type === 'alliance' ? 'alliance' : 'corporation'}
								id={e.id}
								class="size-6 rounded-sm"
							/>
							<span class="truncate">{e.name}</span>
							<span class="ml-auto shrink-0 text-muted-foreground">{e.kills} kills</span>
							<!-- Two zKillboard links that look alike, so each says which it is. -->
							<Tooltip.Root>
								<Tooltip.Trigger>
									{#snippet child({ props })}
										<a
											{...props}
											href="https://zkillboard.com/{e.entity_type}/{e.id}/"
											target="_blank"
											rel="noopener"
											aria-label="Everything they have killed, on zKillboard"
											class="flex text-muted-foreground hover:text-foreground"
										>
											<GlobeIcon class="size-3.5" />
										</a>
									{/snippet}
								</Tooltip.Trigger>
								<Tooltip.Content>Everything they have killed, anywhere</Tooltip.Content>
							</Tooltip.Root>
							<Tooltip.Root>
								<Tooltip.Trigger>
									{#snippet child({ props })}
										<a
											{...props}
											href="https://zkillboard.com/{e.entity_type}/{e.id}/system/{mapped.solar_system_id}/"
											target="_blank"
											rel="noopener"
											aria-label="What they have killed in this system, on zKillboard"
											class="flex text-muted-foreground hover:text-foreground"
										>
											<MapPinIcon class="size-3.5" />
										</a>
									{/snippet}
								</Tooltip.Trigger>
								<Tooltip.Content>What they have killed in {mapped.name}</Tooltip.Content>
							</Tooltip.Root>
						</div>
					{/each}
				</div>
			{/if}
			{#if analysis?.threat_analyzed_at}
				<p class="text-[10px] text-muted-foreground">
					Analyzed {timeAgo(analysis.threat_analyzed_at)}
				</p>
			{/if}
			{#if mapped}
				<a
					class="text-[10px] text-muted-foreground underline-offset-2 hover:underline"
					href="https://zkillboard.com/system/{mapped.solar_system_id}/"
					target="_blank"
					rel="noopener"
				>
					zKillboard
				</a>
			{/if}
		</div>
	</MapPanelContent>
</MapPanel>
