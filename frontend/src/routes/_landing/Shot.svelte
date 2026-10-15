<script lang="ts" module>
	import shots from './shots.json';

	export type ShotName = keyof typeof shots;
</script>

<script lang="ts">
	// A screenshot of the app, in whichever theme the page is in right now. Both variants are
	// in the markup and CSS shows one, keyed on the same `.dark` class the theme toggle sets,
	// so it follows the toggle rather than the OS and never flashes the wrong one. Both are
	// lazy, which is what keeps the hidden one from downloading at all: browsers skip a lazy
	// image that is not rendered. The visible one is still requested as soon as layout puts
	// it on screen, and `priority` moves the hero's to the front of the queue.
	//
	// The files, and the sizes in shots.json, come from `npm run screenshots`.
	let {
		name,
		alt,
		priority = false,
		phoneWidth,
	}: {
		name: ShotName;
		alt: string;
		priority?: boolean;
		/**
		 * On a phone, keep the shot at least this wide and let the frame crop its right-hand
		 * side, rather than shrinking a desktop panel until nothing in it can be read.
		 */
		phoneWidth?: number;
	} = $props();

	const THEMES = ['light', 'dark'] as const;
	const size = $derived(shots[name]);
</script>

{#each THEMES as theme (theme)}
	<img
		src="/landing/{name}-{theme}.webp"
		{alt}
		width={size.width}
		height={size.height}
		loading="lazy"
		decoding="async"
		fetchpriority={priority ? 'high' : 'auto'}
		style:--phone-width={phoneWidth ? `${phoneWidth}px` : undefined}
		class="h-auto w-full max-w-none max-sm:min-w-(--phone-width) {theme === 'light'
			? 'block dark:hidden'
			: 'hidden dark:block'}"
	/>
{/each}
