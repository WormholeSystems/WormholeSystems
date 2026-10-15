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
	import * as Tooltip from '$lib/components/ui/tooltip';
	import { ROLE_HELP, ROLE_LABEL, ROLES_ASCENDING } from '$lib/map/roles';
	import Feature from './_landing/Feature.svelte';
	import Frame from './_landing/Frame.svelte';
	import Reveal from './_landing/Reveal.svelte';
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

<Tooltip.Provider delayDuration={300}>
	<div class="overflow-x-clip">
		<!-- Hero: what it is in a sentence, then the whole thing at once. -->
		<section class="relative isolate border-b border-border">
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
					One live map for the whole corp. Signatures, mass, lifetime and everyone's position, in
					step for everybody looking at it. Open source, and yours to run.
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

			<div class="mx-auto mt-14 w-full max-w-7xl px-4 pb-16 sm:mt-20 sm:px-6 sm:pb-24">
				<div class="hero-tilt relative">
					<div
						class="hero-glow pointer-events-none absolute -inset-x-16 -top-24 bottom-1/3 -z-10"
						aria-hidden="true"
					></div>
					<div
						class="overflow-hidden border border-border bg-background shadow-2xl ring-1 shadow-black/15 ring-black/5 dark:shadow-black/60 dark:ring-white/5"
					>
						<div
							class="flex h-7 items-center gap-3 border-b border-border bg-muted/40 px-3"
							aria-hidden="true"
						>
							<span class="flex gap-1.5">
								<span class="size-2 bg-foreground/15"></span>
								<span class="size-2 bg-foreground/15"></span>
								<span class="size-2 bg-foreground/15"></span>
							</span>
							<span
								class="flex-1 truncate text-center font-mono text-[10px] tracking-wider text-muted-foreground"
							>
								maps / nightfall home chain
							</span>
							<span class="w-10"></span>
						</div>
						<Shot
							name="app"
							phoneWidth={760}
							priority
							alt="The map page: a chain of nine systems out of a C5 home, coloured by class and status, with a route to Jita highlighted. Beside it the signatures of the home system, the pilots and their ships, recent kills, and the route planner."
						/>
					</div>
				</div>
			</div>
		</section>

		<!-- The tour: a part of the app, then what it is for. -->
		<div class="mx-auto flex w-full max-w-7xl flex-col gap-28 px-4 py-24 sm:gap-36 sm:px-6">
			<Feature
				id="signatures"
				label="01 · Signatures"
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
				label="02 · Connections"
				title="Mass and lifetime that add up"
				body="Every jump is recorded against the hole it went through, with the ship that made it, so a connection knows what it has actually taken rather than what somebody remembered. Critical ones surface before anyone rolls into them."
				points={[
					'Both ends of the hole, its type, limits and time left',
					'A jump log per connection, from tracking or typed in by hand',
				]}
				reverse
			>
				<Frame bare class="mx-auto max-w-[618px]">
					<div>
						<Shot
							name="connection"
							phoneWidth={520}
							alt="A connection's details: an H296 out of home and the K162 on the far side, healthy lifetime, mass at critical with a bar showing 13 percent left, and beside it the jump log of nine jumps including a Moros out and back."
						/>
					</div>
				</Frame>
			</Feature>

			<Feature
				label="03 · Routing"
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
				label="04 · Intel"
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
				label="05 · Access"
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
			<section aria-labelledby="more-heading">
				<Reveal>
					<div class="max-w-2xl">
						<p class="font-mono text-[10px] tracking-[0.2em] text-muted-foreground uppercase">
							06 · And the rest
						</p>
						<h2
							id="more-heading"
							class="mt-4 font-heading text-3xl font-semibold tracking-tight sm:text-4xl"
						>
							The rest of living in a wormhole
						</h2>
					</div>

					<div class="mt-12 grid gap-6 lg:grid-cols-5">
						<div class="flex min-w-0 flex-col overflow-hidden border border-border lg:col-span-3">
							<div class="p-6">
								<h3 class="font-heading text-lg font-medium tracking-tight">Discord alerts</h3>
								<p class="mt-2 max-w-prose text-sm text-muted-foreground">
									Standing questions about the chain, answered into a channel: kills near home,
									hostiles a few jumps out, an exit in range of capitals. Slash commands answer from
									the same map.
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
						<div class="flex min-w-0 flex-col overflow-hidden border border-border lg:col-span-2">
							<div class="p-6">
								<h3 class="font-heading text-lg font-medium tracking-tight">Undo anything</h3>
								<p class="mt-2 max-w-prose text-sm text-muted-foreground">
									Every change is a command with an inverse, kept with who made it. A mis-pasted
									scan is one keystroke back, or rewind the map to any point in the evening.
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

					<div
						class="mt-6 grid gap-px border border-border bg-border sm:grid-cols-2 lg:grid-cols-3"
					>
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
			</section>
		</div>

		<SelfHost reference={data.reference} />

		<section class="relative isolate border-t border-border">
			<div
				class="hero-grid cta pointer-events-none absolute inset-0 -z-10"
				aria-hidden="true"
			></div>
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
</Tooltip.Provider>

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
			color-mix(in oklab, var(--color-amber-500) 16%, transparent),
			transparent 70%
		);
		filter: blur(20px);
	}

	/* The screenshot leans back a little and straightens as it scrolls into place. Only where
	   the browser can tie the animation to scrolling; elsewhere it simply sits flat. */
	@media (prefers-reduced-motion: no-preference) {
		@supports (animation-timeline: view()) {
			.hero-tilt {
				transform-origin: 50% 0;
				animation: straighten linear both;
				animation-timeline: view();
				animation-range: entry 10% cover 45%;
			}
		}
	}

	@keyframes straighten {
		from {
			transform: perspective(1800px) rotateX(14deg) scale(0.96);
		}
		to {
			transform: perspective(1800px) rotateX(0deg) scale(1);
		}
	}
</style>
