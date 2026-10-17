<script lang="ts">
	import ArrowRightIcon from '@lucide/svelte/icons/arrow-right';
	import FlagIcon from '@lucide/svelte/icons/flag';
	import GitForkIcon from '@lucide/svelte/icons/git-fork';
	import LayoutGridIcon from '@lucide/svelte/icons/layout-grid';
	import MapIcon from '@lucide/svelte/icons/map';
	import NetworkIcon from '@lucide/svelte/icons/network';
	import SatelliteDishIcon from '@lucide/svelte/icons/satellite-dish';
	import TelescopeIcon from '@lucide/svelte/icons/telescope';
	import UsersIcon from '@lucide/svelte/icons/users';

	import { Button } from '$lib/components/ui/button';
	import { ROLE_HELP, ROLE_LABEL, ROLES_ASCENDING } from '$lib/map/roles';
	import Band from './_landing/Band.svelte';
	import Feature from './_landing/Feature.svelte';
	import Frame from './_landing/Frame.svelte';
	import Reveal from './_landing/Reveal.svelte';
	import RunningIt from './_landing/RunningIt.svelte';
	import SectionLabel from './_landing/SectionLabel.svelte';
	import SelfHost from './_landing/SelfHost.svelte';
	import Shot from './_landing/Shot.svelte';
	import SignatureDemo from './_landing/SignatureDemo.svelte';

	let { data } = $props();

	const roles = ROLES_ASCENDING.map((role) => ({ term: ROLE_LABEL[role], text: ROLE_HELP[role] }));

	const rest = [
		{
			icon: UsersIcon,
			title: 'Everyone, where they are',
			body: 'Pilots who share their location show up on the map in the ship they are flying, with how many jumps they are from wherever you are looking.',
		},
		{
			icon: FlagIcon,
			title: 'Rally points',
			body: 'Mark where the fleet forms up. It sits at the top of everybody’s map with the jumps to get there.',
		},
		{
			icon: TelescopeIcon,
			title: 'EVE Scout',
			body: 'Thera and Turnur connections pulled in and kept current, so the route planner can use them too.',
		},
		{
			icon: NetworkIcon,
			title: 'Lays itself out',
			body: 'Drag systems where you like them, or let the map draw the chain as a tree from home outwards.',
		},
		{
			icon: LayoutGridIcon,
			title: 'Your own layout',
			body: 'Drag the panels where you want them. Phone, tablet, laptop and desktop each remember their own arrangement.',
		},
		{
			icon: SatelliteDishIcon,
			title: 'Raidable skyhooks',
			body: 'Theft windows across New Eden, nearest first, for the nights the chain has nothing better.',
		},
	];
</script>

<div class="overflow-x-clip">
	<!-- Hero: what it is in a sentence, then the whole thing at once. -->
	<!-- Above the next band, which the app window hangs over. -->
	<section class="relative isolate z-10">
		<div class="hero-grid pointer-events-none absolute inset-0 -z-10" aria-hidden="true"></div>
		<div class="mx-auto w-full max-w-4xl px-4 pt-16 text-center sm:px-6 sm:pt-24">
			<p
				class="inline-flex items-center gap-2 border border-border bg-background/60 px-2.5 py-1 font-mono text-[10px] tracking-[0.2em] text-muted-foreground uppercase backdrop-blur"
			>
				<span class="size-1.5 animate-pulse rounded-full bg-green-500"></span>
				Live, shared wormhole maps
			</p>
			<h1
				class="mt-7 font-heading text-5xl leading-[1.02] font-semibold tracking-tight text-balance sm:text-7xl"
			>
				Map the chain together.
			</h1>
			<p class="mx-auto mt-6 max-w-2xl text-base text-muted-foreground sm:text-lg">
				One live map for the whole corp. Signatures, mass, lifetime and everyone's position, in step
				for everybody looking at it. Open source, and yours to run.
			</p>
			<div class="mt-9 flex flex-wrap items-center justify-center gap-3">
				<Button href="/maps" size="lg">
					<MapIcon data-icon="inline-start" />
					Open your maps
				</Button>
				<Button href="#self-host" size="lg" variant="outline">
					Self-host it
					<ArrowRightIcon data-icon="inline-end" />
				</Button>
			</div>
			<p class="mt-7 font-mono text-[10px] tracking-[0.15em] text-muted-foreground/70 uppercase">
				ESI-secure · No client install · Your database
			</p>
		</div>

		<div
			class="mx-auto mt-14 -mb-24 w-full max-w-[1480px] px-4 sm:mt-20 sm:-mb-40 sm:px-8 lg:-mb-60 xl:-mb-64"
		>
			<!-- The app as a window floating over the page, the way it sits on a desktop. -->
			<div class="relative">
				<div
					class="hero-glow pointer-events-none absolute -inset-x-24 -top-20 bottom-1/4 -z-10"
					aria-hidden="true"
				></div>
				<div class="app-window overflow-hidden rounded-[11px] bg-background">
					<div
						class="relative flex h-8 items-center border-b border-border bg-muted/60 px-3.5 sm:h-9"
						aria-hidden="true"
					>
						<span class="flex gap-2">
							<span class="size-3 rounded-full bg-[#ff5f57]"></span>
							<span class="size-3 rounded-full bg-[#febc2e]"></span>
							<span class="size-3 rounded-full bg-[#28c840]"></span>
						</span>
						<span
							class="absolute inset-x-24 truncate text-center text-xs font-medium text-muted-foreground"
						>
							Home Chain<span class="max-sm:hidden">&nbsp;· WormholeSystems</span>
						</span>
					</div>
					<Shot
						name="app"
						phoneWidth={760}
						priority
						alt="The map page: a chain of seven systems out of a C5 home, one of them hostile, with the home system's signatures, its class and statics, and the pilots on the map in the ships they are flying."
					/>
				</div>
			</div>
		</div>
	</section>

	<SelfHost reference={data.reference} />

	<!-- The tour: a part of the app, then what it is for. -->
	<Feature
		id="signatures"
		index="01"
		label="Signatures"
		title="Scanning is copy and paste"
		body="Drop probe-scanner output in and the panel diffs it: new signatures appear, renamed ones keep their link, and a wormhole you have flown ties itself to the connection it opened. Nobody re-scans what the group already worked out."
		points={[
			'Green is new, amber was seen again, red is gone and one click from deleted',
			'Reads the scanner in any client language',
		]}
	>
		<SignatureDemo />
	</Feature>

	<Feature
		index="02"
		label="Connections"
		title="Mass and lifetime that add up"
		body="Every jump is recorded against the hole it went through, with the ship that made it, so a connection knows what it has actually taken rather than what somebody remembered. Critical ones surface before anyone rolls into them."
		points={[
			'Both ends of the hole, its type, limits and time left',
			'A jump log per connection, from tracking or typed in by hand',
		]}
		reverse
	>
		<Frame class="mx-auto max-w-[666px]">
			<Shot
				name="connection"
				alt="A connection's details open over the map: an H296 out of home and the K162 on the far side, healthy lifetime, mass at critical with a bar showing 13 percent left, and beside it the jump log of nine jumps including a Moros out and back."
			/>
		</Frame>
	</Feature>

	<Feature
		index="03"
		label="Routing"
		title="The way out, through the chain"
		body="Plan from any system to any other across your wormholes and the gates together, weighted by security and by how much lifetime and mass you are willing to trust. Every pilot's distance comes from the same graph."
		points={[
			'Shortest, safer, or the long way round through low-sec',
			'Avoid systems, and leave EOL or critical holes out',
		]}
	>
		<Frame>
			<Shot
				name="route"
				phoneWidth={520}
				alt="The route planner: from home J115405 to Jita in three jumps, two of them through the chain's wormholes and out of the high-sec exit at Ikuchi."
			/>
		</Frame>
	</Feature>

	<Feature
		index="04"
		label="Intel"
		title="Know what is next door"
		body="Recent kills land on the systems they happened in, and a look over the killboard says who lives where and how busy they have been. Notes keep the rest: structures, timezones, what not to bring."
		points={[
			'Threat rings on the map for the busy systems',
			'Kills filtered to the chain, or to one side of it',
		]}
		reverse
	>
		<Frame>
			<Shot
				name="intel"
				phoneWidth={720}
				alt="The intel for a hostile C5 next door: a critical threat rating with the alliance and corporation behind it, the notes the scout left about their structures and timezone, and the week's kills in the chain."
			/>
		</Frame>
	</Feature>

	<Feature
		index="05"
		label="Access"
		title="Decide exactly who sees what"
		body="Grant a character, a corporation or a whole alliance, at four levels, with an expiry if you want one. Or hand out a share link for somebody outside the map entirely, and withdraw it without touching anyone else."
		points={roles}
	>
		<Frame>
			<Shot
				name="access"
				phoneWidth={680}
				alt="The access settings: the owner, a manager, members, the whole corporation, a friendly alliance as viewers, and one guest whose access ends in three days."
			/>
		</Frame>
	</Feature>

	<!-- The rest, shorter. -->
	<Band tone="tint" labelledby="more-heading">
		<Reveal>
			<div class="max-w-2xl">
				<SectionLabel index="06" label="And the rest" />
				<h2
					id="more-heading"
					class="mt-6 font-heading text-3xl font-semibold tracking-tight sm:text-4xl"
				>
					The rest of living in a wormhole
				</h2>
			</div>

			<div class="mt-12 grid gap-6 lg:grid-cols-5">
				<div
					class="flex min-w-0 flex-col overflow-hidden border border-border bg-background lg:col-span-3"
				>
					<div class="p-6">
						<h3 class="font-heading text-lg font-medium tracking-tight">Discord alerts</h3>
						<p class="mt-2 max-w-prose text-sm text-muted-foreground">
							Standing questions about the chain, answered into a channel: kills near home, hostiles
							a few jumps out, an exit in range of capitals. Slash commands answer from the same
							map.
						</p>
					</div>
					<div class="bento-stage flex flex-1 items-center border-t border-border p-4 sm:p-8">
						<div
							class="w-full overflow-hidden border border-border shadow-xl shadow-black/10 dark:shadow-black/40"
						>
							<Shot
								name="alerts"
								phoneWidth={560}
								alt="Three Discord alerts: kills within two jumps of home, hostiles in the chain pinging everyone, and dreadnought range of Amamake, each posting to a channel."
							/>
						</div>
					</div>
				</div>
				<div
					class="flex min-w-0 flex-col overflow-hidden border border-border bg-background lg:col-span-2"
				>
					<div class="p-6">
						<h3 class="font-heading text-lg font-medium tracking-tight">Undo anything</h3>
						<p class="mt-2 max-w-prose text-sm text-muted-foreground">
							Every change is a command with an inverse, kept with who made it. A mis-pasted scan is
							one keystroke back, or rewind the map to any point in the evening.
						</p>
					</div>
					<div
						class="bento-stage flex flex-1 items-center justify-center border-t border-border p-4 sm:p-8"
					>
						<div class="w-full max-w-[384px] shadow-xl shadow-black/10 dark:shadow-black/40">
							<Shot
								name="history"
								alt="The map's history, newest first: who edited the notes, set the occupier, marked a system hostile, set a rally point, and so on back through the evening."
							/>
						</div>
					</div>
				</div>
			</div>

			<div class="mt-6 grid gap-px border border-border bg-border sm:grid-cols-2 lg:grid-cols-3">
				{#each rest as item (item.title)}
					{@const Icon = item.icon}
					<div class="flex flex-col gap-2 bg-background p-6">
						<Icon class="size-4 text-amber-500" />
						<h3 class="mt-1 font-heading text-base font-medium tracking-tight">{item.title}</h3>
						<p class="text-sm text-muted-foreground">{item.body}</p>
					</div>
				{/each}
			</div>
		</Reveal>
	</Band>

	<RunningIt index="07" />

	<section class="relative isolate border-t border-border bg-muted/40 dark:bg-card/40">
		<div class="hero-grid cta pointer-events-none absolute inset-0 -z-10" aria-hidden="true"></div>
		<div class="mx-auto w-full max-w-6xl px-4 py-24 text-center sm:px-6 sm:py-32">
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
</div>

<style>
	/* The map canvas's own ruling, fading out from behind the headline. */
	.hero-grid {
		background-image:
			linear-gradient(to right, var(--color-grid) 1px, transparent 1px),
			linear-gradient(to bottom, var(--color-grid) 1px, transparent 1px);
		background-size: 40px 40px;
		background-position: center top;
		mask-image: radial-gradient(ellipse 70% 55% at 50% 0%, black 20%, transparent 75%);
	}

	.hero-grid.cta {
		background-position: center center;
		mask-image: radial-gradient(ellipse 45% 60% at 50% 50%, black 10%, transparent 75%);
	}

	/* The same ruling, faint, as a stage for the smaller shots. */
	.bento-stage {
		background-color: var(--canvas);
		background-image:
			linear-gradient(
				to right,
				color-mix(in oklab, var(--color-grid) 60%, transparent) 1px,
				transparent 1px
			),
			linear-gradient(
				to bottom,
				color-mix(in oklab, var(--color-grid) 60%, transparent) 1px,
				transparent 1px
			);
		background-size: 24px 24px;
	}

	.hero-glow {
		background: radial-gradient(
			ellipse 50% 60% at 50% 40%,
			color-mix(in oklab, var(--color-amber-500) 9%, transparent),
			transparent 70%
		);
		filter: blur(20px);
	}

	/* A window on a desktop: a hairline edge with a lit top, and shadow in layers, tight
	   under the edge and wide and soft beneath, so it reads as floating in either theme. */
	.app-window {
		box-shadow:
			0 0 0 1px color-mix(in oklab, var(--foreground) 12%, transparent),
			inset 0 1px 0 color-mix(in oklab, white 10%, transparent),
			0 1px 2px rgb(0 0 0 / 0.06),
			0 6px 12px -2px rgb(0 0 0 / 0.08),
			0 24px 48px -12px rgb(0 0 0 / 0.18),
			0 60px 120px -24px rgb(0 0 0 / 0.25);
	}

	:global(.dark) .app-window {
		box-shadow:
			0 0 0 1px color-mix(in oklab, white 12%, transparent),
			inset 0 1px 0 color-mix(in oklab, white 8%, transparent),
			0 1px 2px rgb(0 0 0 / 0.4),
			0 8px 16px -4px rgb(0 0 0 / 0.45),
			0 28px 56px -12px rgb(0 0 0 / 0.6),
			0 70px 140px -24px rgb(0 0 0 / 0.7);
	}
</style>
