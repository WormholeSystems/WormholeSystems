<script lang="ts">
	// How long ago a scan was last pasted into the system, so a stale list is obvious at a
	// glance. The exact time, in EVE time, is in the tooltip.
	import ClockIcon from '@lucide/svelte/icons/clock';
	import * as Tooltip from '$lib/components/ui/tooltip';
	import { timeAgo, utcShort } from '$lib/format';
	import { ticking } from '$lib/now.svelte';

	let { pastedAt }: { pastedAt: string | null } = $props();

	const clock = ticking(30_000);
</script>

{#if pastedAt}
	<Tooltip.Provider delayDuration={300}>
		<Tooltip.Root>
			<Tooltip.Trigger
				class="ml-1 inline-flex items-center gap-1 whitespace-nowrap text-muted-foreground/70 normal-case tabular-nums"
				data-testid="signatures-pasted-at"
			>
				<ClockIcon class="size-3" />
				{timeAgo(pastedAt, clock.current)}
			</Tooltip.Trigger>
			<Tooltip.Content>Signatures last pasted {utcShort(pastedAt)}</Tooltip.Content>
		</Tooltip.Root>
	</Tooltip.Provider>
{/if}
