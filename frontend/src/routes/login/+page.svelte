<script lang="ts">
	// Signing in: one card, one button that goes to EVE's own sign-in, over the map's grid.
	// What EVE's consent screen will ask for is there for whoever wants to know, folded away
	// under the button.
	import { page } from '$app/state';
	import ArrowLeftIcon from '@lucide/svelte/icons/arrow-left';
	import BookOpenIcon from '@lucide/svelte/icons/book-open';
	import ChevronDownIcon from '@lucide/svelte/icons/chevron-down';

	import Logo from '$lib/components/Logo.svelte';

	// The SSO callback sends a character the instance does not let in back here, before
	// anything about it is stored.
	const refused = $derived(page.url.searchParams.get('refused') === '1');

	// Somewhere to come back to, passed through to the SSO flow, which only ever accepts a
	// path on this site.
	const returnTo = $derived(page.url.searchParams.get('return_to'));
	const signInHref = $derived(
		returnTo ? `/auth/login?return_to=${encodeURIComponent(returnTo)}` : '/auth/login',
	);

	// What EVE's consent screen will list, in the words of what each is for.
	const scopes = [
		{ name: 'Location', why: 'which system you are in, so the map can follow you' },
		{ name: 'Ship type', why: 'what you are flying, shown beside your name' },
		{ name: 'Online status', why: 'so nobody is tracked while logged off' },
		{ name: 'Waypoints', why: 'so a route planned here can be sent to your client' },
	];

	// A soft light that follows the pointer along the card. Only a CSS variable changes, so
	// the browser repaints one gradient rather than anything being laid out again.
	let card = $state<HTMLElement | null>(null);
	function follow(event: PointerEvent) {
		if (!card) return;
		const box = card.getBoundingClientRect();
		card.style.setProperty('--px', `${event.clientX - box.left}px`);
		card.style.setProperty('--py', `${event.clientY - box.top}px`);
	}
</script>

<section
	class="relative isolate grid min-h-[calc(100dvh-6rem)] place-items-center overflow-x-clip py-12 sm:py-20"
>
	<div class="drift pointer-events-none absolute inset-0 -z-10" aria-hidden="true"></div>

	<div class="w-full max-w-md">
		<div class="relative">
			<!-- The border carries a faint, slow sweep of light; the card sits inside it, a little
			     glassy. -->
			<div class="sweep rounded-[15px] p-px">
				<div
					bind:this={card}
					onpointermove={follow}
					role="presentation"
					class="card relative overflow-hidden rounded-[14px] bg-background/75 px-6 pt-10 pb-7 text-center backdrop-blur-xl sm:px-10"
				>
					<Logo class="mx-auto size-10 text-foreground" />
					<h1 class="mt-6 font-heading text-3xl font-semibold tracking-tight sm:text-4xl">
						Sign in
					</h1>
					<p class="mx-auto mt-3 max-w-xs text-sm text-pretty text-muted-foreground">
						With your EVE character, through EVE's own sign-in. No password here, and nothing to
						install.
					</p>

					{#if refused}
						<p
							class="mt-6 border border-destructive/50 bg-destructive/10 px-3 py-2.5 text-left text-sm text-destructive"
							role="alert"
							data-testid="login-refused"
						>
							This instance is limited to certain characters, corporations or alliances, and the
							character you signed in with is not one of them.
						</p>
					{/if}

					<!-- The one thing to do here. -->
					<a
						href={signInHref}
						data-sveltekit-reload
						class="mt-8 flex h-12 w-full items-center justify-center gap-3 bg-primary px-5 text-sm font-medium text-primary-foreground shadow-lg shadow-primary/25 transition-colors hover:bg-primary/90 focus-visible:ring-2 focus-visible:ring-ring/50 focus-visible:outline-none"
						data-testid="login-sso"
					>
						<span aria-hidden="true"><Logo class="size-4" /></span>
						Log in with EVE Online
					</a>

					<details class="group mt-5 text-left" data-testid="login-scopes">
						<summary
							class="mx-auto flex w-fit cursor-pointer list-none items-center gap-1.5 text-center text-xs text-muted-foreground transition-colors select-none hover:text-foreground [&::-webkit-details-marker]:hidden"
						>
							EVE will ask for your location, ship and waypoints. Why?
							<ChevronDownIcon class="size-3 transition-transform group-open:rotate-180" />
						</summary>
						<ul class="mt-4 flex flex-col gap-2 border-t border-border pt-4">
							{#each scopes as scope (scope.name)}
								<li class="text-xs text-muted-foreground">
									<span class="text-foreground">{scope.name}</span>: {scope.why}.
								</li>
							{/each}
							<li class="pt-1 text-xs text-muted-foreground">
								Your position is only shared on maps where you switch sharing on, and you can sign
								in just to look.
							</li>
						</ul>
					</details>
				</div>
			</div>
		</div>

		<nav
			class="mt-8 flex flex-wrap items-center justify-center gap-x-6 gap-y-2 text-sm"
			aria-label="Elsewhere"
		>
			<a
				href="/"
				class="flex items-center gap-1.5 text-muted-foreground transition-colors hover:text-foreground"
			>
				<ArrowLeftIcon class="size-3.5" />
				Back to the landing page
			</a>
			<a
				href="/documentation"
				class="flex items-center gap-1.5 text-muted-foreground transition-colors hover:text-foreground"
			>
				<BookOpenIcon class="size-3.5" />
				Read the documentation
			</a>
		</nav>
	</div>
</section>

<style>
	/* The map canvas's ruling, drifting slowly and fading out towards the edges. */
	.drift {
		background-image:
			linear-gradient(to right, var(--color-grid) 1px, transparent 1px),
			linear-gradient(to bottom, var(--color-grid) 1px, transparent 1px);
		background-size: 40px 40px;
		background-position: center center;
		mask-image: radial-gradient(ellipse 60% 60% at 50% 45%, black 10%, transparent 80%);
	}

	/* The border light: a faint, wide band of amber drifting round the card. */
	@property --sweep {
		syntax: '<angle>';
		inherits: false;
		initial-value: 0deg;
	}
	.sweep {
		background: conic-gradient(
			from var(--sweep),
			var(--border) 0deg,
			var(--border) 220deg,
			color-mix(in oklab, var(--color-amber-500) 22%, var(--border)) 290deg,
			var(--border) 360deg
		);
		box-shadow:
			0 1px 2px rgb(0 0 0 / 0.06),
			0 24px 48px -12px rgb(0 0 0 / 0.18),
			0 60px 120px -24px rgb(0 0 0 / 0.22);
	}
	:global(.dark) .sweep {
		box-shadow:
			0 1px 2px rgb(0 0 0 / 0.4),
			0 28px 56px -12px rgb(0 0 0 / 0.6),
			0 70px 140px -24px rgb(0 0 0 / 0.7);
	}

	/* A soft light where the pointer is, on the glass. */
	.card::before {
		content: '';
		position: absolute;
		inset: 0;
		pointer-events: none;
		background: radial-gradient(
			22rem circle at var(--px, 50%) var(--py, -20%),
			color-mix(in oklab, var(--color-amber-500) 6%, transparent),
			transparent 60%
		);
	}

	@media (prefers-reduced-motion: no-preference) {
		.drift {
			animation: drift 60s linear infinite;
		}
		.sweep {
			animation: sweep 24s linear infinite;
		}
	}

	@keyframes drift {
		to {
			background-position:
				calc(50% + 40px) calc(50% + 40px),
				calc(50% + 40px) calc(50% + 40px);
		}
	}
	@keyframes sweep {
		to {
			--sweep: 360deg;
		}
	}
</style>
