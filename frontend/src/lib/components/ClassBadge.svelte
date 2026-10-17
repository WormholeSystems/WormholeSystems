<script lang="ts">
	// A solar system's class letter (C5, H, L, NS, P…) in its class colour. The colour has to go
	// through a CSS variable: the class tokens live in a plain `:root` block, not in `@theme`,
	// so Tailwind never generates `text-c5` and a utility class would silently inherit.
	import { classMeta } from '$lib/map/classes';
	import Hint from '$lib/components/Hint.svelte';
	import { cn } from '$lib/utils';

	let {
		classId,
		security,
		class: className,
		title,
	}: {
		classId: number | null;
		/** Security status, used for the k-space classes that have no class id. */
		security: number | null;
		class?: string;
		title?: string;
	} = $props();

	const meta = $derived(classMeta(classId, security));
</script>

<Hint text={title} focusable={false}>
	{#snippet child({ props })}
		<span
			class={cn('font-mono', className)}
			style="color: var(--color-{meta.token})"
			{...props}
			data-testid="class-badge"
			data-class={meta.short}
		>
			{meta.short}
		</span>
	{/snippet}
</Hint>
