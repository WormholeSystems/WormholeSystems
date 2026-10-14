<script lang="ts">
	// Sections fade up as they come into view, once each.
	//
	// Visible is the resting state and hiding is what JS opts into, never the other way
	// round: a section that starts at opacity 0 and waits for an observer is a section that
	// is missing entirely for anyone whose observer never fires, and for anything reading
	// the server's HTML.
	import type { Snippet } from 'svelte';

	let { children }: { children: Snippet } = $props();

	let el = $state<HTMLElement | null>(null);
	let armed = $state(false);
	let shown = $state(false);

	$effect(() => {
		if (!el) return;
		if (
			typeof IntersectionObserver === 'undefined' ||
			window.matchMedia('(prefers-reduced-motion: reduce)').matches
		) {
			return;
		}
		// Anything already on screen when we get here has nothing to animate into.
		const rect = el.getBoundingClientRect();
		if (rect.top < window.innerHeight) return;

		armed = true;
		const io = new IntersectionObserver(
			(entries) => {
				if (entries.some((e) => e.isIntersecting)) {
					shown = true;
					io.disconnect();
				}
			},
			{ threshold: 0.1, rootMargin: '0px 0px -6% 0px' },
		);
		io.observe(el);
		return () => io.disconnect();
	});
</script>

<div bind:this={el} class:reveal={armed && !shown}>
	{@render children()}
</div>

<style>
	.reveal {
		opacity: 0;
		transform: translateY(12px);
	}

	/* The transition lives on the element itself so removing .reveal animates it back. */
	div {
		transition:
			opacity 500ms ease-out,
			transform 500ms ease-out;
	}

	@media (prefers-reduced-motion: reduce) {
		div {
			transition: none;
		}

		.reveal {
			opacity: 1;
			transform: none;
		}
	}
</style>
