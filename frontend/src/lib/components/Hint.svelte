<script lang="ts">
	// A hover hint on whatever it wraps, through our tooltip. The element is the caller's:
	// `child` gets the trigger's props to spread on it, so no wrapper lands in the DOM. The
	// delay comes from the one provider in the root layout.
	import type { Snippet } from 'svelte';

	import * as Tooltip from '$lib/components/ui/tooltip';

	let {
		text,
		clippedOnly = false,
		focusable = true,
		label = false,
		side = 'top',
		child: render,
	}: {
		text: string | null | undefined;
		/** Only when the element's text is cut short by `truncate`: a name that fits says nothing more. */
		clippedOnly?: boolean;
		/** Off for plain text (a name, a region), which should not become focusable of its own. */
		focusable?: boolean;
		/** The hint is the element's only name (an icon with no text), so it is its label too. */
		label?: boolean;
		side?: 'top' | 'bottom' | 'left' | 'right';
		child: Snippet<[{ props: Record<string, unknown> }]>;
	} = $props();

	let open = $state(false);
	let trigger = $state<HTMLElement | null>(null);

	function clipped(el: HTMLElement | null): boolean {
		return el !== null && (el.scrollWidth > el.clientWidth || el.scrollHeight > el.clientHeight);
	}
</script>

<Tooltip.Root
	bind:open={() => open, (next) => (open = next && (!clippedOnly || clipped(trigger)))}
	disabled={!text}
>
	<Tooltip.Trigger bind:ref={trigger}>
		{#snippet child({ props })}
			<!-- The element's own `disabled` and `onclick` are its business, so the trigger's are
			     left out rather than fighting them; pressing down is what puts the hint away. -->
			{@const {
				disabled: _disabled,
				onclick: _onclick,
				tabindex,
				onpointerdown,
				...rest
			} = props as Record<string, unknown> & {
				onpointerdown?: (event: PointerEvent) => void;
			}}
			{@render render({
				props: {
					...rest,
					...(focusable ? { tabindex } : {}),
					...(label && text ? { 'aria-label': text } : {}),
					onpointerdown: (event: PointerEvent) => {
						onpointerdown?.(event);
						open = false;
					},
				},
			})}
		{/snippet}
	</Tooltip.Trigger>
	<Tooltip.Content {side}>{text}</Tooltip.Content>
</Tooltip.Root>
