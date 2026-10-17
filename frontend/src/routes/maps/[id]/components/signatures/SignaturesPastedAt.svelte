<script lang="ts">
	// How long ago a scan was last pasted into the system, so a stale list is obvious at a
	// glance, kept to a number and a unit so it fits a narrow header. The exact time, in
	// EVE time, is in the tooltip.
	import ClockIcon from '@lucide/svelte/icons/clock';
	import * as Tooltip from '$lib/components/ui/tooltip';
	import { timeAgoShort, utcShort } from '$lib/format';
	import { ticking } from '$lib/now.svelte';

	let { pastedAt }: { pastedAt: string | null } = $props();

	// Seconds tick by the second for the first minute; after that the minutes are enough.
	const clock = ticking(1_000);

	function age(when: string, now: Date): string {
		const seconds = Math.max(0, Math.floor((now.getTime() - new Date(when).getTime()) / 1000));
		return seconds < 60 ? `${seconds}s` : timeAgoShort(when, now);
	}
</script>

{#if pastedAt}
	<Tooltip.Root>
		<Tooltip.Trigger
			class="ml-1 inline-flex items-center gap-1 leading-none whitespace-nowrap text-muted-foreground/70 normal-case tabular-nums"
			data-testid="signatures-pasted-at"
		>
			<ClockIcon class="size-3" />
			<!-- The mono digits sit high in their box; this centres them on the icon. -->
			<span class="translate-y-0.5">{age(pastedAt, clock.current)}</span>
		</Tooltip.Trigger>
		<Tooltip.Content>Signatures last pasted {utcShort(pastedAt)}</Tooltip.Content>
	</Tooltip.Root>
{/if}
