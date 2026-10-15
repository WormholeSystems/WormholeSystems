<script lang="ts">
	// Every error page: what happened, in a sentence, and what to do about it. The status is
	// there, small, for whoever needs to quote it; the message is the large part. Nothing
	// that was thrown is ever printed: the loads hand over a message written for a person,
	// and anything unplanned arrives as a short id that is in the server log beside it.
	import { browser } from '$app/environment';
	import { page } from '$app/state';
	import ArrowRightIcon from '@lucide/svelte/icons/arrow-right';
	import BookOpenIcon from '@lucide/svelte/icons/book-open';
	import HouseIcon from '@lucide/svelte/icons/house';
	import LogInIcon from '@lucide/svelte/icons/log-in';
	import MapIcon from '@lucide/svelte/icons/map';
	import RotateCwIcon from '@lucide/svelte/icons/rotate-cw';

	import { Button } from '$lib/components/ui/button';

	interface Explanation {
		/** The short name after the code, in the mono label. */
		name: string;
		title: string;
		body: string;
		/** What the far end of the little severed chain is called. */
		far: string;
		actions: ('signin' | 'retry' | 'maps' | 'home' | 'docs')[];
	}

	const status = $derived(page.status);
	const kind = $derived(page.error?.kind);
	const signedIn = $derived(Boolean(page.data.me));
	// A navigation that failed because this browser lost its connection never reached the
	// server, so the status says nothing useful; the browser does.
	const disconnected = $derived(browser && status >= 500 && !navigator.onLine);

	const explanation = $derived.by((): Explanation => {
		if (disconnected || kind === 'offline') {
			return {
				name: disconnected ? 'Offline' : 'Unreachable',
				title: disconnected ? 'You are offline.' : 'The server is not answering.',
				body: disconnected
					? 'This browser has lost its connection. Nothing on the map is lost: once you are back online, try again.'
					: 'This is usually temporary, the server restarting after an update or a moment of load. Nothing on the map is lost. Try again in a moment.',
				far: 'No answer',
				actions: ['retry', 'home'],
			};
		}
		if (status === 401) {
			return {
				name: 'Signed out',
				title: kind === 'map' ? 'Sign in to open this map.' : 'Sign in to see this.',
				body:
					kind === 'map'
						? 'This map is only open to the people it is shared with. Sign in with your EVE character and you come straight back here.'
						: 'This page belongs to an account. Sign in with your EVE character and you come straight back here.',
				far: 'Locked',
				actions: ['signin', 'home'],
			};
		}
		if (status === 403) {
			return {
				name: 'No access',
				title: 'Your role here does not allow that.',
				body: "The map's owner or one of its managers can give you a higher role, under Access in the map's settings.",
				far: 'No access',
				actions: ['maps', 'home'],
			};
		}
		if (status === 404 && kind === 'map') {
			return {
				name: 'Map not found',
				title: 'This map is not open to you.',
				body: "It may have been deleted, or it is not shared with your character, corporation or alliance. Ask the map's owner to add you under Access.",
				far: 'Unknown',
				actions: ['maps', 'home'],
			};
		}
		if ((status === 404 && kind === 'share') || status === 410) {
			return {
				name: status === 410 ? 'Gone' : 'Link withdrawn',
				title: 'This share link no longer works.',
				body: 'Whoever made it has withdrawn it, or made a new one, which shuts the old one. Ask them for the current link.',
				far: 'Withdrawn',
				actions: signedIn ? ['maps', 'home'] : ['home'],
			};
		}
		if (status === 404) {
			return {
				name: 'Not found',
				title: 'There is nothing at this address.',
				body: 'It may be mistyped, or the page has moved. Everything else is where you left it.',
				far: 'Unknown',
				actions: signedIn ? ['maps', 'home', 'docs'] : ['home', 'docs'],
			};
		}
		if (status >= 500) {
			return {
				name: 'Server error',
				title: 'Something went wrong on our side.',
				body: 'It has been logged. Trying again usually works. If it keeps happening, the reference below helps whoever runs this server find it.',
				far: 'No answer',
				actions: ['retry', 'home'],
			};
		}
		return {
			name: 'Error',
			// From an `error()` call in a load, so it was written to be read.
			title: page.error?.message ?? 'That did not work.',
			body: 'Go back and try again, or start from somewhere else.',
			far: 'Unknown',
			actions: ['home'],
		};
	});

	const returnTo = $derived(page.url.pathname + page.url.search);
	const signInHref = $derived(`/auth/login?return_to=${encodeURIComponent(returnTo)}`);
	const errorId = $derived(page.error?.errorId);
</script>

<section class="relative isolate grid min-h-[75vh] place-items-center px-4 py-16 sm:px-6">
	<div class="error-grid pointer-events-none absolute inset-0 -z-10" aria-hidden="true"></div>

	<div class="w-full max-w-xl text-center" data-testid="error-page" data-status={status}>
		<!-- A chain that stops: where you are, a hole, and nothing on the other side. -->
		<div class="mx-auto flex max-w-sm items-center justify-center" aria-hidden="true">
			<span
				class="border border-amber-500/70 bg-background px-2.5 py-1.5 font-mono text-[11px] text-foreground shadow-sm"
			>
				<span class="text-amber-500">◆</span> You
			</span>
			<span class="relative mx-1 h-px flex-1 border-t border-dashed border-muted-foreground/50">
				<span
					class="absolute top-1/2 left-1/2 -translate-x-1/2 -translate-y-1/2 border border-border bg-background px-1.5 py-0.5 font-mono text-[10px] text-muted-foreground tabular-nums"
				>
					{status}
				</span>
			</span>
			<span
				class="border border-dashed border-muted-foreground/40 bg-background px-2.5 py-1.5 font-mono text-[11px] text-muted-foreground"
			>
				{explanation.far}
			</span>
		</div>

		<p
			class="mt-10 font-mono text-[11px] tracking-[0.2em] text-muted-foreground uppercase"
			data-testid="error-status"
		>
			Error {status} · {explanation.name}
		</p>
		<h1
			class="mt-4 font-heading text-4xl font-semibold tracking-tight text-balance sm:text-5xl"
			data-testid="error-title"
		>
			{explanation.title}
		</h1>
		<p class="mx-auto mt-5 max-w-prose text-pretty text-muted-foreground">{explanation.body}</p>

		<div class="mt-8 flex flex-wrap items-center justify-center gap-3">
			{#each explanation.actions as action, i (action)}
				{@const variant = i === 0 ? 'default' : 'outline'}
				{#if action === 'signin'}
					<Button href={signInHref} size="lg" {variant} data-sveltekit-reload>
						<LogInIcon data-icon="inline-start" />
						Log in with EVE Online
					</Button>
				{:else if action === 'retry'}
					<Button size="lg" {variant} onclick={() => location.reload()} data-testid="error-retry">
						<RotateCwIcon data-icon="inline-start" />
						Try again
					</Button>
				{:else if action === 'maps'}
					<Button href="/maps" size="lg" {variant}>
						<MapIcon data-icon="inline-start" />
						Your maps
					</Button>
				{:else if action === 'home'}
					<Button href="/" size="lg" {variant}>
						<HouseIcon data-icon="inline-start" />
						Home
					</Button>
				{:else if action === 'docs'}
					<Button href="/documentation" size="lg" {variant}>
						<BookOpenIcon data-icon="inline-start" />
						Documentation
						<ArrowRightIcon data-icon="inline-end" />
					</Button>
				{/if}
			{/each}
		</div>

		{#if errorId}
			<p class="mt-10 font-mono text-[10px] tracking-[0.15em] text-muted-foreground/80 uppercase">
				Reference
				<span class="ml-1 text-foreground normal-case select-all" data-testid="error-id"
					>{errorId}</span
				>
			</p>
		{/if}
	</div>
</section>

<style>
	/* The map canvas's ruling, fading out around the message. */
	.error-grid {
		background-image:
			linear-gradient(to right, var(--color-grid) 1px, transparent 1px),
			linear-gradient(to bottom, var(--color-grid) 1px, transparent 1px);
		background-size: 40px 40px;
		background-position: center center;
		mask-image: radial-gradient(ellipse 50% 50% at 50% 45%, black 10%, transparent 75%);
	}
</style>
