<script lang="ts">
	// The icon cluster on the title row: status, flags, and badges. Pushes itself
	// to the right edge with ml-auto.
	import ActivityIcon from '@lucide/svelte/icons/activity';
	import ApertureIcon from '@lucide/svelte/icons/aperture';
	import CircleDashedIcon from '@lucide/svelte/icons/circle-dashed';
	import CircleHelpIcon from '@lucide/svelte/icons/circle-help';
	import FanIcon from '@lucide/svelte/icons/fan';
	import FlagIcon from '@lucide/svelte/icons/flag';
	import HomeIcon from '@lucide/svelte/icons/home';
	import LockIcon from '@lucide/svelte/icons/lock';
	import RadarIcon from '@lucide/svelte/icons/radar';
	import SatelliteIcon from '@lucide/svelte/icons/satellite';
	import ShieldCheckIcon from '@lucide/svelte/icons/shield-check';
	import SkullIcon from '@lucide/svelte/icons/skull';

	import type { MapSystemView } from '$lib/api/types/MapSystemView';
	import type { SystemStatus } from '$lib/api/types/SystemStatus';
	import type { SigCounts } from '$lib/map/grouping';
	import * as Tooltip from '$lib/components/ui/tooltip';
	import EffectBadge from '$lib/components/EffectBadge.svelte';
	import SovereigntyBadge from '$lib/components/SovereigntyBadge.svelte';
	import { statusColor } from '$lib/map/helpers';

	let {
		node,
		sigCounts,
		unmapped,
	}: {
		node: MapSystemView;
		sigCounts: SigCounts;
		unmapped: number;
	} = $props();

	const mapped = $derived(node.kind === 'system' ? node : null);

	const STATUS_ICONS = {
		friendly: ShieldCheckIcon,
		hostile: SkullIcon,
		active: ActivityIcon,
		unscanned: RadarIcon,
		empty: CircleDashedIcon,
		unknown: CircleHelpIcon,
	} satisfies Record<SystemStatus, typeof ShieldCheckIcon>;
	const StatusIcon = $derived(STATUS_ICONS[node.status]);
</script>

<span class="ml-auto flex shrink-0 items-center gap-1">
	{#if node.status !== 'unknown'}
		<Tooltip.Root>
			<Tooltip.Trigger class="flex" aria-label={node.status}>
				<StatusIcon class="size-[14px]" style="color: {statusColor(node.status)}" />
			</Tooltip.Trigger>
			<Tooltip.Content>{node.status[0].toUpperCase() + node.status.slice(1)}</Tooltip.Content>
		</Tooltip.Root>
	{/if}
	{#if node.is_home}
		<Tooltip.Root>
			<Tooltip.Trigger class="flex"><HomeIcon class="size-[14px] text-amber-400" /></Tooltip.Trigger
			>
			<Tooltip.Content>Home system</Tooltip.Content>
		</Tooltip.Root>
	{/if}
	{#if node.is_rally}
		<Tooltip.Root>
			<Tooltip.Trigger class="flex"><FlagIcon class="size-[14px] text-pink-500" /></Tooltip.Trigger>
			<Tooltip.Content>Rally point</Tooltip.Content>
		</Tooltip.Root>
	{/if}
	{#if node.is_pinned}
		<Tooltip.Root>
			<Tooltip.Trigger class="flex">
				<LockIcon class="size-[14px] text-muted-foreground" />
			</Tooltip.Trigger>
			<Tooltip.Content>Pinned in place</Tooltip.Content>
		</Tooltip.Root>
	{/if}
	{#if sigCounts.total > 0}
		<Tooltip.Root>
			<Tooltip.Trigger class="flex" data-testid="sig-icon">
				<SatelliteIcon
					class="size-[14px] {sigCounts.uncategorized > 0 ? 'text-rose-500' : 'text-amber-500'}"
				/>
			</Tooltip.Trigger>
			<Tooltip.Content>
				{sigCounts.total} signature{sigCounts.total === 1 ? '' : 's'}{sigCounts.uncategorized > 0
					? `, ${sigCounts.uncategorized} uncategorized`
					: ''}
			</Tooltip.Content>
		</Tooltip.Root>
	{/if}
	{#if unmapped > 0}
		<Tooltip.Root>
			<Tooltip.Trigger class="flex" data-testid="unmapped-icon">
				<FanIcon class="size-[14px] text-sky-500" />
			</Tooltip.Trigger>
			<Tooltip.Content>
				Has {unmapped} unmapped wormhole{unmapped === 1 ? '' : 's'}
			</Tooltip.Content>
		</Tooltip.Root>
	{/if}
	{#if mapped?.is_shattered}
		<Tooltip.Root>
			<Tooltip.Trigger class="flex" data-testid="shattered-icon">
				<ApertureIcon class="size-3 text-amber-500/90" />
			</Tooltip.Trigger>
			<Tooltip.Content>Shattered system</Tooltip.Content>
		</Tooltip.Root>
	{/if}
	{#if mapped?.sovereignty}
		<SovereigntyBadge sovereignty={mapped.sovereignty} />
	{:else if mapped?.effect_name}
		<EffectBadge name={mapped.effect_name} wormholeClassId={mapped.wormhole_class_id ?? 0} />
	{/if}
</span>
