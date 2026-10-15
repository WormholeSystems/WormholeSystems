<script lang="ts">
	// A screenshot or a live panel, set off from the page: a hairline border, a soft shadow,
	// and a faint wash of light behind it so the dark theme's black-on-black still reads as
	// an object rather than a hole.
	import type { Snippet } from 'svelte';

	let {
		caption,
		bare = false,
		class: cls = '',
		children,
	}: {
		/** A mono label under the frame, saying what is in it. */
		caption?: string;
		/** No border of its own: for shots of popovers, which bring theirs. */
		bare?: boolean;
		class?: string;
		children: Snippet;
	} = $props();
</script>

<figure class="relative isolate {cls}">
	<div class="glow pointer-events-none absolute -inset-8 -z-10" aria-hidden="true"></div>
	<div
		class="overflow-hidden {bare
			? ''
			: 'border border-border bg-background shadow-2xl shadow-black/10 dark:shadow-black/50'}"
	>
		{@render children()}
	</div>
	{#if caption}
		<figcaption
			class="mt-3 font-mono text-[10px] tracking-[0.15em] text-muted-foreground/80 uppercase"
		>
			{caption}
		</figcaption>
	{/if}
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
