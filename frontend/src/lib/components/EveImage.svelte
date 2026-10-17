<script lang="ts">
	import Hint from '$lib/components/Hint.svelte';

	// An image from CCP's image server, for every entity kind. `size` is the server resolution:
	// a power of two, at or above 2x the displayed size to stay crisp. Factions are served from
	// the corporations endpoint.
	const paths = {
		character: 'characters',
		corporation: 'corporations',
		alliance: 'alliances',
		faction: 'corporations',
		type: 'types',
	} as const;
	const variants = {
		character: 'portrait',
		corporation: 'logo',
		alliance: 'logo',
		faction: 'logo',
		type: 'icon',
	} as const;

	let {
		kind,
		id,
		class: cls = '',
		title,
		size = 64,
	}: {
		kind: keyof typeof paths;
		id: number;
		class?: string;
		title?: string;
		size?: 32 | 64 | 128 | 256 | 512;
	} = $props();
</script>

{#snippet image(props: Record<string, unknown>)}
	<img
		src="https://images.evetech.net/{paths[kind]}/{id}/{variants[kind]}?size={size}"
		class={cls}
		{...props}
		alt=""
		loading="lazy"
	/>
{/snippet}

<!-- Most images carry no title, and those stay a bare image rather than a tooltip each. -->
{#if title}
	<Hint text={title} focusable={false}>
		{#snippet child({ props })}
			{@render image(props)}
		{/snippet}
	</Hint>
{:else}
	{@render image({})}
{/if}
