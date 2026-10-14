<script lang="ts">
	// The documentation shell: categories down the left on a wide screen, a select on a
	// narrow one, and the page itself in the middle.
	import { page } from '$app/state';
	import type { Snippet } from 'svelte';
	import type { DocSection } from '$lib/docs';

	let { data, children }: { data: { categories: DocSection[] }; children: Snippet } = $props();

	const here = $derived(page.url.pathname);
	const flat = $derived(
		data.categories.flatMap((c) => c.pages.map((p) => ({ ...p, group: c.title }))),
	);
</script>

<div class="mx-auto flex w-full max-w-6xl gap-8 px-5 py-8">
	<!-- The sidebar is the whole map of the section, so it does not collapse or paginate. -->
	<nav class="sticky top-20 hidden h-fit w-56 shrink-0 flex-col gap-5 lg:flex">
		{#each data.categories as category (category.slug)}
			<div class="flex flex-col gap-1">
				<span class="text-[10px] tracking-wider text-muted-foreground uppercase">
					{category.title}
				</span>
				{#each category.pages as entry (entry.url)}
					<a
						href={entry.url}
						class="text-sm transition-colors {here === entry.url
							? 'text-foreground'
							: 'text-muted-foreground hover:text-foreground'}"
						aria-current={here === entry.url ? 'page' : undefined}
					>
						{entry.title}
					</a>
				{/each}
			</div>
		{/each}
	</nav>

	<div class="min-w-0 flex-1">
		<select
			class="mb-6 h-9 w-full border border-border bg-card px-2 text-sm lg:hidden"
			data-testid="docs-mobile-nav"
			value={here}
			onchange={(ev) => (window.location.href = ev.currentTarget.value)}
		>
			{#each flat as entry (entry.url)}
				<option value={entry.url}>{entry.group} · {entry.title}</option>
			{/each}
		</select>

		{@render children()}
	</div>
</div>
