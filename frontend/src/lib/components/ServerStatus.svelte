<script lang="ts">
	// Tranquility, in the header: whether the server is up and what time it is in EVE. The
	// clock is there because the game runs on UTC (downtime, timers, other timezones) and the
	// browser's clock does not. Everything past that lives in the tooltip.
	import { createQuery, useQueryClient } from '@tanstack/svelte-query';
	import { untrack } from 'svelte';

	import { key, q } from '$lib/api/queries';
	import { ticking } from '$lib/now.svelte';
	import type { ServerState } from '$lib/api/types/ServerState';
	import type { ServerStatus } from '$lib/api/types/ServerStatus';
	import * as Tooltip from '$lib/components/ui/tooltip';
	import { openUserSocket } from '$lib/ws';

	let { signedIn = false, initial = null }: { signedIn?: boolean; initial?: ServerStatus | null } =
		$props();

	// Layout seed, 60s poll and socket push all converge on the one cache entry.
	const client = useQueryClient();
	const statusQuery = createQuery(() => ({
		...q.serverStatus(),
		initialData: untrack(() => initial) ?? undefined,
	}));
	const status = $derived(statusQuery.data ?? null);
	// Minutes are all that shows, but ticking faster stops the clock sitting a minute
	// behind after the tab has been asleep.
	const clock = ticking(10_000);
	const now = $derived(clock.current);

	const STATES = {
		unknown: { dot: 'bg-muted-foreground/40', label: 'Checking Tranquility…', short: null },
		online: { dot: 'bg-emerald-500', label: 'Tranquility is up', short: null },
		// Up, but the door is shut: nothing is broken, and nothing works.
		vip: { dot: 'bg-amber-500', label: 'VIP mode: only CCP can log in', short: 'VIP' },
		offline: { dot: 'bg-red-500', label: 'Tranquility is down', short: 'Down' },
		unreachable: {
			dot: 'bg-amber-500',
			label: 'ESI is unreachable, so the server state is unknown',
			short: 'ESI down',
		},
	} satisfies Record<ServerState, { dot: string; label: string; short: string | null }>;

	const meta = $derived(STATES[status?.state ?? 'unknown']);
	/** Worth a word in the header instead of a player count. */
	const degraded = $derived(meta.short !== null);
	// Live tracking runs on ESI, so without it the map quietly stops updating. VIP does not
	// count: the server is up and the pilots on it still move.
	const trackingPaused = $derived(status?.state === 'offline' || status?.state === 'unreachable');
	/** A headcount only means something when there is a server behind it. */
	const hasPopulation = $derived(status?.state === 'online' || status?.state === 'vip');

	const eveTime = $derived(
		`${String(now.getUTCHours()).padStart(2, '0')}:${String(now.getUTCMinutes()).padStart(2, '0')}`,
	);
	const eveDate = $derived(
		now.toLocaleDateString('en-GB', {
			timeZone: 'UTC',
			day: 'numeric',
			month: 'short',
			year: 'numeric',
		}),
	);

	// One decimal: "24.5K" costs the same room as "25K" and says more.
	const compact = new Intl.NumberFormat('en-US', {
		notation: 'compact',
		compactDisplay: 'short',
		maximumFractionDigits: 1,
	});
	const full = new Intl.NumberFormat('en-GB');

	const uptime = $derived.by(() => {
		if (!status?.start_time) return null;
		const minutes = Math.floor((now.getTime() - new Date(status.start_time).getTime()) / 60_000);
		if (minutes < 0) return null;
		const hours = Math.floor(minutes / 60);
		return hours > 0 ? `${hours}h ${minutes % 60}m` : `${minutes}m`;
	});

	// The query's own interval is the fallback for anyone this push cannot reach, since
	// signed-out visitors have no socket.
	$effect(() => {
		if (!signedIn) return;
		return openUserSocket((event) => {
			if (event.type === 'server_status_changed')
				client.invalidateQueries({ queryKey: key.serverStatus });
		});
	});
</script>

<Tooltip.Root>
	<Tooltip.Trigger>
		{#snippet child({ props })}
			<span
				{...props}
				class="flex items-center gap-2 font-mono text-xs text-muted-foreground"
				data-testid="server-status"
				data-state={status?.state ?? 'unknown'}
			>
				<span class="size-1.5 shrink-0 rounded-full {meta.dot}"></span>
				<span class="tabular-nums">{eveTime}</span>
				{#if degraded}
					<span class="hidden text-foreground/70 sm:inline">{meta.short}</span>
				{:else if status && status.players > 0}
					<span class="hidden tabular-nums text-muted-foreground/60 sm:inline">
						{compact.format(status.players)}
					</span>
				{/if}
			</span>
		{/snippet}
	</Tooltip.Trigger>
	<Tooltip.Content class="max-w-64">
		<div class="flex flex-col gap-1 text-xs">
			<span class="font-medium">{meta.label}</span>
			<span class="text-muted-foreground">{eveDate} {eveTime} EVE time</span>
			{#if status && hasPopulation}
				<span class="text-muted-foreground">
					{full.format(status.players)} pilots online
					{#if uptime}· up {uptime}{/if}
				</span>
			{/if}
			{#if status?.server_version && hasPopulation}
				<span class="text-muted-foreground/70">Build {status.server_version}</span>
			{/if}
			{#if trackingPaused}
				<span class="text-muted-foreground">
					Live tracking is paused until the server is back.
				</span>
			{/if}
		</div>
	</Tooltip.Content>
</Tooltip.Root>
