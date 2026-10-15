<script lang="ts">
	// The self-hosting band, first thing after the app itself: this is the reason to pick
	// this over somebody else's copy. The longer story of what setup checks and what wsctl
	// does afterwards is further down, in RunningIt.
	import CheckIcon from '@lucide/svelte/icons/check';
	import CopyIcon from '@lucide/svelte/icons/copy';
	import { copyText } from '$lib/clipboard';
	import DatabaseIcon from '@lucide/svelte/icons/database';
	import LockIcon from '@lucide/svelte/icons/lock';
	import ServerIcon from '@lucide/svelte/icons/server';

	import type { ReferenceCounts } from '$lib/api/types/ReferenceCounts';
	import Reveal from './Reveal.svelte';
	import SectionLabel from './SectionLabel.svelte';

	let { reference }: { reference: ReferenceCounts } = $props();

	// What this install actually holds, straight out of its own database. Deliberately not
	// usage numbers: every copy is somebody's own server, so a global "capsuleers aboard"
	// would be a figure nobody running this could stand behind.
	const knows = $derived([
		{ value: reference.solar_systems, label: 'Solar systems' },
		{ value: reference.wormhole_systems, label: 'Wormhole systems' },
		{ value: reference.stargates, label: 'Stargates' },
		{ value: reference.wormhole_types, label: 'Wormhole types' },
	]);

	const command = "curl --proto '=https' --tlsv1.2 -sSf https://install.wormhole.systems | sh";
	let copied = $state(false);
	let resetTimer: ReturnType<typeof setTimeout> | undefined;

	function copy() {
		void copyText(command, { silent: true });
		copied = true;
		clearTimeout(resetTimer);
		resetTimer = setTimeout(() => (copied = false), 2000);
	}

	// What setup looks at before it asks for a single credential, in a word each.
	const checks = ['Docker and Compose v2', 'Disk', 'DNS', 'Ports 80 and 443', 'Up to date'];

	const owns = [
		{
			icon: ServerIcon,
			title: 'Your machine',
			body: 'One box, four containers, no account with anybody.',
		},
		{
			icon: DatabaseIcon,
			title: 'Your database',
			body: 'The chain lives in your Postgres. Nothing phones home.',
		},
		{
			icon: LockIcon,
			title: 'Your keys',
			body: 'Your own EVE application, so ESI tokens never leave the host.',
		},
	];
</script>

<!-- Straight under the hero, which hangs its app window over the top of this band: the
     top padding is the overlap plus the band's own breathing room. -->
<section
	id="self-host"
	class="relative overflow-hidden border-t border-border bg-muted/40 dark:bg-card/40"
>
	<div class="glow pointer-events-none absolute inset-0" aria-hidden="true"></div>
	<div
		class="relative mx-auto w-full max-w-7xl px-4 pt-36 pb-24 sm:px-6 sm:pt-52 sm:pb-28 lg:pt-72 xl:pt-80"
	>
		<Reveal>
			<!-- Read top to bottom: what it is, then the command, then what backs it up. -->
			<div class="mx-auto max-w-3xl text-center">
				<SectionLabel label="Open source · Self-hosted" accent center />
				<h2
					class="mt-6 font-heading text-4xl font-semibold tracking-tight text-balance sm:text-5xl"
				>
					Run it yourself. It is one command.
				</h2>
				<p class="mx-auto mt-5 max-w-2xl text-muted-foreground">
					Not a trial of a hosted thing: the whole application on your own machine, with your corp's
					chain in a database only you can reach.
				</p>
			</div>

			<!-- The command is the point of the band, so it gets the room. -->
			<div class="mx-auto mt-12 max-w-5xl">
				<div
					class="overflow-hidden border border-amber-500/40 bg-background shadow-2xl shadow-amber-950/10 dark:shadow-black/50"
				>
					<div
						class="flex h-10 items-center justify-between gap-3 border-b border-border bg-muted/40 pr-1.5 pl-4"
					>
						<span class="font-mono text-[10px] tracking-[0.2em] text-muted-foreground uppercase">
							Install<span class="max-sm:hidden">&nbsp;· any Linux box with Docker</span>
						</span>
						<button
							class="flex h-7 items-center gap-1.5 px-2.5 font-mono text-[11px] tracking-wider uppercase transition-colors {copied
								? 'text-emerald-500'
								: 'text-muted-foreground hover:bg-muted hover:text-foreground'}"
							aria-label="Copy the install command"
							data-testid="copy-command"
							onclick={copy}
						>
							{#if copied}
								<CheckIcon class="size-3.5" />
								Copied
							{:else}
								<CopyIcon class="size-3.5" />
								Copy
							{/if}
						</button>
					</div>
					<div class="flex gap-3 px-5 py-6 font-mono text-sm sm:px-7 sm:py-8 sm:text-lg">
						<span class="shrink-0 text-amber-500 select-none" aria-hidden="true">$</span>
						<code class="min-w-0 flex-1 break-all text-foreground">{command}</code>
					</div>
				</div>
				<ul
					class="mt-5 flex flex-wrap items-center justify-center gap-x-5 gap-y-2 text-xs text-muted-foreground"
					aria-label="What setup checks before it asks for anything"
				>
					{#each checks as check (check)}
						<li class="flex items-center gap-1.5">
							<CheckIcon class="size-3 text-emerald-500" />
							{check}
						</li>
					{/each}
				</ul>
			</div>

			<!-- And the quieter row underneath. -->
			<div
				class="mx-auto mt-16 grid max-w-5xl gap-px border border-border bg-border sm:grid-cols-3"
			>
				{#each owns as item (item.title)}
					{@const Icon = item.icon}
					<div class="flex flex-col gap-2 bg-background p-5">
						<Icon class="size-4 text-amber-500" />
						<h3 class="text-sm font-medium">{item.title}</h3>
						<p class="text-xs text-muted-foreground">{item.body}</p>
					</div>
				{/each}
			</div>

			<div class="mx-auto mt-10 max-w-5xl">
				<ul class="grid grid-cols-2 gap-x-6 gap-y-4 sm:grid-cols-4">
					{#each knows as stat (stat.label)}
						<li data-testid="landing-stat" class="text-center">
							<span class="block font-heading text-2xl font-semibold tracking-tight tabular-nums">
								{stat.value.toLocaleString('en-US')}
							</span>
							<span class="mt-0.5 block text-xs text-muted-foreground">{stat.label}</span>
						</li>
					{/each}
				</ul>
				<p class="mt-5 text-center text-xs text-muted-foreground/80">
					What this copy already knows, seeded from CCP's static data export on the first boot.
					<a
						href="#running-it"
						class="text-foreground underline decoration-border underline-offset-4 hover:decoration-foreground"
						>What setup does, and wsctl afterwards</a
					>
				</p>
			</div>
		</Reveal>
	</div>
</section>

<style>
	.glow {
		background: radial-gradient(
			60rem 34rem at 50% 45%,
			color-mix(in oklab, var(--color-amber-500) 10%, transparent),
			transparent 70%
		);
	}
</style>
