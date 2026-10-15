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
			<div class="grid items-start gap-10 lg:grid-cols-12 lg:gap-14">
				<div class="lg:col-span-5">
					<SectionLabel label="Open source · Self-hosted" accent />
					<h2 class="mt-6 font-heading text-4xl font-semibold tracking-tight sm:text-5xl">
						Run it yourself.<br />It is one command.
					</h2>
					<p class="mt-5 text-muted-foreground">
						Not a trial of a hosted thing. The whole application, on your own machine, with your
						corp's chain in a database only you can reach.
					</p>
				</div>

				<div class="min-w-0 lg:col-span-7">
					<div
						class="flex items-center gap-3 border border-amber-500/40 bg-background p-4 font-mono text-sm shadow-lg shadow-black/5 dark:shadow-black/30"
					>
						<span class="shrink-0 text-muted-foreground select-none">$</span>
						<span class="min-w-0 flex-1 break-all">{command}</span>
						<button
							class="shrink-0 text-muted-foreground transition-colors hover:text-foreground"
							aria-label="Copy the install command"
							data-testid="copy-command"
							onclick={copy}
						>
							{#if copied}
								<CheckIcon class="size-4 text-emerald-500" />
							{:else}
								<CopyIcon class="size-4" />
							{/if}
						</button>
					</div>
					<p class="mt-4 text-sm text-muted-foreground">
						Postgres, the API, the web server and TLS come up together under Docker. It asks for
						your domain and your EVE application, checks the machine, and works the rest out.
						<a
							href="#running-it"
							class="text-foreground underline decoration-border underline-offset-4 hover:decoration-foreground"
							>What it checks, and wsctl afterwards</a
						>
					</p>

					<div class="mt-8 grid gap-px border border-border bg-border sm:grid-cols-3">
						{#each owns as item (item.title)}
							{@const Icon = item.icon}
							<div class="flex flex-col gap-2 bg-background p-4">
								<Icon class="size-4 text-amber-500" />
								<h3 class="text-sm font-medium">{item.title}</h3>
								<p class="text-xs text-muted-foreground">{item.body}</p>
							</div>
						{/each}
					</div>
				</div>
			</div>

			<div class="mt-16 border-t border-border pt-6">
				<p class="font-mono text-[10px] tracking-[0.2em] text-muted-foreground uppercase">
					What this copy already knows
				</p>
				<ul class="mt-4 grid grid-cols-2 gap-x-6 gap-y-4 sm:grid-cols-4">
					{#each knows as stat (stat.label)}
						<li data-testid="landing-stat">
							<span class="block font-heading text-2xl font-semibold tracking-tight tabular-nums">
								{stat.value.toLocaleString('en-US')}
							</span>
							<span class="mt-0.5 block text-xs text-muted-foreground">{stat.label}</span>
						</li>
					{/each}
				</ul>
				<p class="mt-4 text-xs text-muted-foreground/80">
					Seeded from CCP's static data export on the first boot, and re-seeded when they publish a
					new one.
				</p>
			</div>
		</Reveal>
	</div>
</section>

<style>
	.glow {
		background: radial-gradient(
			52rem 30rem at 85% 0%,
			color-mix(in oklab, var(--color-amber-500) 10%, transparent),
			transparent 70%
		);
	}
</style>
