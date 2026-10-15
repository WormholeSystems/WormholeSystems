<script lang="ts">
	import ArrowRightIcon from '@lucide/svelte/icons/arrow-right';
	import BellIcon from '@lucide/svelte/icons/bell';
	import CrosshairIcon from '@lucide/svelte/icons/crosshair';
	import GitForkIcon from '@lucide/svelte/icons/git-fork';
	import LayoutGridIcon from '@lucide/svelte/icons/layout-grid';
	import MapIcon from '@lucide/svelte/icons/map';
	import RouteIcon from '@lucide/svelte/icons/route';
	import TelescopeIcon from '@lucide/svelte/icons/telescope';
	import UndoIcon from '@lucide/svelte/icons/undo-2';

	import { Button } from '$lib/components/ui/button';
	import * as Tooltip from '$lib/components/ui/tooltip';
	import MapPanelHeader from '$lib/components/map-panel/MapPanelHeader.svelte';
	import MassBar from '$lib/components/MassBar.svelte';
	import AccessDemo from './_landing/AccessDemo.svelte';
	import CountUp from './_landing/CountUp.svelte';
	import LandingChain from './_landing/LandingChain.svelte';
	import Reveal from './_landing/Reveal.svelte';
	import Section from './_landing/Section.svelte';
	import SelfHost from './_landing/SelfHost.svelte';
	import SignatureDemo from './_landing/SignatureDemo.svelte';

	let { data } = $props();

	// What this install actually holds, straight out of its own database. Deliberately not
	// usage numbers: every copy is somebody's own server, so a global "capsuleers aboard"
	// would be a figure nobody running this could stand behind.
	const knows = $derived([
		{ value: data.reference.solar_systems, label: 'Solar systems' },
		{ value: data.reference.wormhole_systems, label: 'Wormhole systems' },
		{ value: data.reference.stargates, label: 'Stargates' },
		{ value: data.reference.wormhole_types, label: 'Wormhole types' },
	]);

	// The same numbers the connection panel shows, in the states it colours differently.
	const holes = [
		{ name: 'K162 → J123746', remaining: 82, jumps: 4, status: 'fresh' },
		{ name: 'C247 → J104351', remaining: 38, jumps: 14, status: 'reduced' },
		{ name: 'B274 → Korasen', remaining: 7, jumps: 22, status: 'critical' },
	];

	const rest = [
		{
			icon: RouteIcon,
			title: 'Smart routing',
			body: 'The shortest way home through the chain and the gates together, weighted by security and by how much lifetime and mass you will trust.',
		},
		{
			icon: CrosshairIcon,
			title: 'Threat analysis',
			body: 'Recent kills surface against the systems they happened in, so you know what you are jumping into.',
		},
		{
			icon: BellIcon,
			title: 'Discord alerts',
			body: 'Watch a system, a jump range, or the whole chain. Slash commands answer from the same map.',
		},
		{
			icon: TelescopeIcon,
			title: 'EVE Scout',
			body: 'Thera and Turnur connections pulled in and kept current, so routing can use them too.',
		},
		{
			icon: LayoutGridIcon,
			title: 'Your own layout',
			body: 'Drag the panels where you want them. Phone, tablet, laptop and desktop each remember their own arrangement.',
		},
		{
			icon: UndoIcon,
			title: 'Undo anything',
			body: 'Every change is a command with an inverse, so a mis-pasted scan or a wrong signature is one keystroke back.',
		},
	];
</script>

<Tooltip.Provider delayDuration={300}>
	<!-- Hero: the map itself, on its own canvas, drawn by the map's own components. -->
	<section class="relative overflow-hidden border-b border-border">
		<div
			class="mx-auto flex w-full max-w-7xl flex-col items-center gap-12 px-6 py-20 xl:flex-row xl:gap-16 xl:py-28"
		>
			<div class="max-w-xl shrink-0">
				<p
					class="flex items-center gap-2 font-mono text-[10px] tracking-[0.2em] text-muted-foreground uppercase"
				>
					<span class="size-1.5 animate-pulse rounded-full bg-green-500"></span>
					Live, shared wormhole maps
				</p>
				<h1
					class="mt-6 font-heading text-5xl leading-[1.02] font-semibold tracking-tight sm:text-6xl"
				>
					Map the chain<br />together.
				</h1>
				<p class="mt-6 max-w-prose text-muted-foreground">
					One live map for the whole corp. Signatures, mass, lifetime and everyone's position, in
					step for everybody looking at it. Run it on your own machine.
				</p>
				<div class="mt-8 flex flex-wrap items-center gap-3">
					<Button href="/maps" size="lg">
						<MapIcon data-icon="inline-start" />
						Open your maps
					</Button>
					<Button href="#self-host" size="lg" variant="outline">
						Self-host it
						<ArrowRightIcon data-icon="inline-end" />
					</Button>
				</div>
				<p class="mt-8 font-mono text-[10px] tracking-[0.15em] text-muted-foreground/70 uppercase">
					ESI-secure · No client install · Your database
				</p>
			</div>

			<div class="shrink-0 overflow-hidden rounded border border-border bg-card shadow-lg">
				<MapPanelHeader>
					home.map · Turnur
					{#snippet actions()}
						<span class="flex items-center gap-1.5">
							<span class="size-2 rounded-full" style="background: var(--color-status-hostile)"
							></span>
							<span class="size-2 rounded-full" style="background: var(--color-status-active)"
							></span>
							<span class="size-2 rounded-full" style="background: var(--color-status-empty)"
							></span>
						</span>
					{/snippet}
				</MapPanelHeader>
				<div class="relative overflow-hidden">
					<LandingChain />
				</div>
			</div>
		</div>
	</section>

	<Section label="What this copy knows" tone="muted">
		<div class="grid grid-cols-2 gap-px bg-border md:grid-cols-4">
			{#each knows as stat (stat.label)}
				<div class="bg-background px-6 py-10 text-center" data-testid="landing-stat">
					<div class="font-heading text-4xl font-semibold tracking-tight">
						<CountUp value={stat.value} />
					</div>
					<div class="mt-2 font-mono text-[10px] tracking-[0.15em] text-muted-foreground uppercase">
						{stat.label}
					</div>
				</div>
			{/each}
		</div>
		<p class="mt-6 text-center text-xs text-muted-foreground">
			Seeded from CCP's static data export on the first boot, and re-seeded when they publish a new
			one.
		</p>
	</Section>

	<Section
		id="signatures"
		label="01 · Signatures"
		title="Scanning is copy and paste"
		body="Drop probe-scanner output in and the panel diffs it: new signatures appear, renamed ones keep their link, and a wormhole you have flown ties itself to the connection it opened. Nobody re-scans what the group already worked out."
	>
		<!-- The signature panel's own rows, from static data. -->
		<SignatureDemo />
	</Section>

	<Section
		label="02 · Connections"
		title="Mass and lifetime that add up"
		body="Every jump is recorded against the hole it went through, with the ship that made it, so a connection knows what it has actually taken rather than what somebody remembered. Critical ones surface before anyone rolls into them."
		tone="muted"
		reverse
	>
		<div class="flex flex-col gap-5 rounded border border-border bg-card p-5">
			{#each holes as hole (hole.name)}
				<div class="flex flex-col gap-2">
					<div class="flex items-baseline justify-between text-xs">
						<span class="font-mono">{hole.name}</span>
						<span class="text-muted-foreground">{hole.jumps} jumps</span>
					</div>
					<MassBar remainingPercent={hole.remaining} />
					<div class="flex items-baseline justify-between text-[11px] text-muted-foreground">
						<span>Remaining</span>
						<span class="tabular-nums">≈ {hole.remaining}%</span>
					</div>
				</div>
			{/each}
		</div>
	</Section>

	<SelfHost />

	<Section
		label="03 · Access"
		title="Decide exactly who sees what"
		body="Grant a character, a corporation or a whole alliance, at four levels, with an expiry if you want one. Or hand out a share link for somebody outside the map entirely, and withdraw it without touching anyone else."
		tone="muted"
		wide
	>
		<AccessDemo />
	</Section>

	<Section label="04 · Everything else" title="The rest of living in a wormhole" wide>
		<div class="grid gap-px bg-border sm:grid-cols-2 lg:grid-cols-3">
			{#each rest as item (item.title)}
				{@const Icon = item.icon}
				<div class="flex flex-col gap-2 bg-background p-6">
					<Icon class="size-4 text-muted-foreground" />
					<h3 class="mt-1 font-heading text-base font-medium tracking-tight">{item.title}</h3>
					<p class="text-sm text-muted-foreground">{item.body}</p>
				</div>
			{/each}
		</div>
	</Section>

	<section class="border-t border-border bg-card/40">
		<div class="mx-auto w-full max-w-6xl px-6 py-24 text-center">
			<Reveal>
				<p class="font-mono text-[10px] tracking-[0.2em] text-muted-foreground uppercase">
					Your map, your database, your machine
				</p>
				<h2 class="mt-4 font-heading text-4xl font-semibold tracking-tight sm:text-5xl">
					Ready to map the void?
				</h2>
				<p class="mx-auto mt-5 max-w-prose text-muted-foreground">
					Nothing about your chain leaves the box you put it on. Read the code, run it, change it.
				</p>
				<div class="mt-9 flex flex-wrap items-center justify-center gap-3">
					<Button href="/maps" size="lg">
						<MapIcon data-icon="inline-start" />
						Open your maps
					</Button>
					<Button
						href="https://github.com/wormholesystems/wormholesystems"
						size="lg"
						variant="outline"
					>
						<GitForkIcon data-icon="inline-start" />
						Source on GitHub
					</Button>
				</div>
			</Reveal>
		</div>
	</section>
</Tooltip.Provider>
