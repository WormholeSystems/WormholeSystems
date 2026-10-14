<script lang="ts">
	import ChevronLeftIcon from '@lucide/svelte/icons/chevron-left';
	import ChevronRightIcon from '@lucide/svelte/icons/chevron-right';
	import type { DocLink } from '$lib/docs';

	let { data }: { data: { html: string; prev: DocLink | null; next: DocLink | null } } = $props();
</script>

<article class="docs-prose min-w-0" data-testid="docs-page">
	<!-- eslint-disable-next-line svelte/no-at-html-tags -->
	{@html data.html}
</article>

<nav class="mt-10 flex items-stretch justify-between gap-4 border-t border-border pt-5">
	{#if data.prev}
		<a
			href={data.prev.url}
			class="group flex max-w-[45%] flex-col gap-0.5 text-sm"
			data-testid="docs-prev"
		>
			<span
				class="flex items-center gap-1 text-[10px] tracking-wider text-muted-foreground uppercase"
			>
				<ChevronLeftIcon class="size-3" /> Previous
			</span>
			<span class="truncate transition-colors group-hover:text-foreground">{data.prev.title}</span>
		</a>
	{:else}
		<span></span>
	{/if}
	{#if data.next}
		<a
			href={data.next.url}
			class="group flex max-w-[45%] flex-col items-end gap-0.5 text-right text-sm"
			data-testid="docs-next"
		>
			<span
				class="flex items-center gap-1 text-[10px] tracking-wider text-muted-foreground uppercase"
			>
				Next <ChevronRightIcon class="size-3" />
			</span>
			<span class="truncate transition-colors group-hover:text-foreground">{data.next.title}</span>
		</a>
	{/if}
</nav>
