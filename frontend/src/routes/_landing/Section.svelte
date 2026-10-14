<script lang="ts">
	// One full-width band explaining one thing, with the product itself alongside the words.
	// Bands alternate surface so the page reads as sections rather than one long column.
	import type { Snippet } from 'svelte';

	import Reveal from './Reveal.svelte';

	let {
		id,
		label,
		title,
		body,
		tone = 'plain',
		reverse = false,
		wide = false,
		children,
	}: {
		id?: string;
		label: string;
		/** Omitted for a band that is only its content, like the stats row. */
		title?: string;
		body?: string;
		tone?: 'plain' | 'muted';
		/** Puts the product on the left, so consecutive bands do not march the same way. */
		reverse?: boolean;
		/** Heading above, content across the whole band, for content that needs the width. */
		wide?: boolean;
		children: Snippet;
	} = $props();
</script>

<section {id} class="border-t border-border {tone === 'muted' ? 'bg-card/40' : ''}">
	<div class="mx-auto w-full max-w-6xl px-6 py-20 sm:py-24">
		<Reveal>
			{#if title && wide}
				<div class="max-w-2xl">
					<p class="font-mono text-[10px] tracking-[0.2em] text-muted-foreground uppercase">
						{label}
					</p>
					<h2 class="mt-4 font-heading text-3xl font-semibold tracking-tight sm:text-4xl">
						{title}
					</h2>
					{#if body}
						<p class="mt-4 text-muted-foreground">{body}</p>
					{/if}
				</div>
				<div class="mt-10">
					{@render children()}
				</div>
			{:else if title}
				<div class="grid items-center gap-10 lg:grid-cols-2 lg:gap-16">
					<div class={reverse ? 'lg:order-last' : ''}>
						<p class="font-mono text-[10px] tracking-[0.2em] text-muted-foreground uppercase">
							{label}
						</p>
						<h2 class="mt-4 font-heading text-3xl font-semibold tracking-tight sm:text-4xl">
							{title}
						</h2>
						{#if body}
							<p class="mt-4 max-w-prose text-muted-foreground">{body}</p>
						{/if}
					</div>
					<div class="min-w-0">
						{@render children()}
					</div>
				</div>
			{:else}
				<p
					class="text-center font-mono text-[10px] tracking-[0.2em] text-muted-foreground uppercase"
				>
					{label}
				</p>
				<div class="mt-8">
					{@render children()}
				</div>
			{/if}
		</Reveal>
	</div>
</section>
