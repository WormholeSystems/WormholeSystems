<script lang="ts">
	// A screenshot or a live panel, set off from the page: a hairline border, a soft shadow,
	// and a faint wash of light behind it so the dark theme's black-on-black still reads as
	// an object rather than a hole.
	import type { Snippet } from 'svelte';

	let { class: cls = '', children }: { class?: string; children: Snippet } = $props();
</script>

<figure class="relative isolate {cls}">
	<div class="glow pointer-events-none absolute -inset-8 -z-10" aria-hidden="true"></div>
	<div
		class="overflow-hidden border border-border bg-background shadow-2xl shadow-black/10 dark:shadow-black/50"
	>
		{@render children()}
	</div>
</figure>

<style>
	.glow {
		background: radial-gradient(
			closest-side,
			color-mix(in oklab, var(--color-amber-500) 9%, transparent),
			transparent
		);
	}
</style>
