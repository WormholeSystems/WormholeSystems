<script lang="ts">
	// One full-width band of the page. Bands alternate between the page surface and a tinted
	// one, with a hairline between them, so the page reads as a run of sections rather than
	// one long column.
	import type { Snippet } from 'svelte';

	let {
		id,
		tone = 'plain',
		labelledby,
		class: cls = '',
		children,
	}: {
		id?: string;
		tone?: 'plain' | 'tint';
		labelledby?: string;
		class?: string;
		children: Snippet;
	} = $props();
</script>

<section
	{id}
	aria-labelledby={labelledby}
	class="relative scroll-mt-12 border-t border-border {tone === 'tint'
		? 'bg-muted/40 dark:bg-card/40'
		: ''} {cls}"
>
	<div class="mx-auto w-full max-w-7xl px-4 py-24 sm:px-6 sm:py-32">
		{@render children()}
	</div>
</section>
