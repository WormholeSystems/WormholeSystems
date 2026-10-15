<script lang="ts">
	// One stop on the tour, in a band of its own: a few lines about a part of the app beside
	// the part itself. Consecutive features swap sides and surfaces on wide screens so the
	// page does not march one way; on a phone the words always come first.
	import type { Snippet } from 'svelte';

	import Band from './Band.svelte';
	import Reveal from './Reveal.svelte';
	import SectionLabel from './SectionLabel.svelte';

	let {
		id,
		index,
		label,
		title,
		body,
		points = [],
		reverse = false,
		children,
	}: {
		id?: string;
		index: string;
		label: string;
		title: string;
		body: string;
		/** Short specifics under the body, each one a thing you can go and find in the app. */
		points?: (string | { term: string; text: string })[];
		/** Media on the left and the tinted surface, for every other band. */
		reverse?: boolean;
		children: Snippet;
	} = $props();
</script>

<Band {id} tone={reverse ? 'tint' : 'plain'}>
	<Reveal>
		<div class="grid items-center gap-10 lg:grid-cols-12 lg:gap-14">
			<div class="lg:col-span-5 xl:col-span-4 {reverse ? 'lg:order-last' : ''}">
				<SectionLabel {index} {label} />
				<h2
					class="mt-6 font-heading text-3xl font-semibold tracking-tight text-balance sm:text-4xl"
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
</Band>
