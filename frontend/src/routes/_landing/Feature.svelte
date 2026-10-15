<script lang="ts">
	// One stop on the tour: a few lines about a part of the app beside the part itself.
	// Consecutive features swap sides on wide screens so the page does not march one way;
	// on a phone the words always come first.
	import type { Snippet } from 'svelte';

	import Reveal from './Reveal.svelte';

	let {
		id,
		label,
		title,
		body,
		points = [],
		reverse = false,
		children,
	}: {
		id?: string;
		label: string;
		title: string;
		body: string;
		/** Short specifics under the body, each one a thing you can go and find in the app. */
		points?: (string | { term: string; text: string })[];
		reverse?: boolean;
		children: Snippet;
	} = $props();
</script>

<section {id} class="scroll-mt-16">
	<Reveal>
		<div class="grid items-center gap-10 lg:grid-cols-12 lg:gap-14">
			<div class="lg:col-span-5 xl:col-span-4 {reverse ? 'lg:order-last' : ''}">
				<p class="font-mono text-[10px] tracking-[0.2em] text-muted-foreground uppercase">
					{label}
				</p>
				<h2
					class="mt-4 font-heading text-3xl font-semibold tracking-tight text-balance sm:text-4xl"
				>
					{title}
				</h2>
				<p class="mt-4 text-muted-foreground">{body}</p>
				{#if points.length}
					<ul class="mt-6 flex flex-col border-t border-border">
						{#each points as point, i (i)}
							<li class="flex gap-3 border-b border-border py-2.5 text-sm text-muted-foreground">
								<span class="mt-2 size-1 shrink-0 bg-amber-500" aria-hidden="true"></span>
								{#if typeof point === 'string'}
									{point}
								{:else}
									<span>
										<span class="font-medium text-foreground">{point.term}</span>
										<span class="sr-only">:</span>
										<span class="ml-1">{point.text}</span>
									</span>
								{/if}
							</li>
						{/each}
					</ul>
				{/if}
			</div>
			<div class="min-w-0 lg:col-span-7 xl:col-span-8">
				{@render children()}
			</div>
		</div>
	</Reveal>
</section>
