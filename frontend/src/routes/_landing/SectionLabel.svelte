<script lang="ts">
	// A section's number and name, in the mono uppercase the panel headers use, with a rule
	// running off to the right so each band announces itself before its heading does.
	let {
		index,
		label,
		accent = false,
		center = false,
	}: {
		index?: string;
		label: string;
		/** Amber, for the one band that is about running it yourself. */
		accent?: boolean;
		/** Rules on both sides, for a band that reads down the middle. */
		center?: boolean;
	} = $props();
</script>

<p
	class="flex items-center gap-3 font-mono text-[11px] tracking-[0.2em] uppercase {accent
		? 'text-amber-500'
		: 'text-muted-foreground'}"
>
	{#if center}
		<span class="h-px min-w-8 flex-1 bg-border" aria-hidden="true"></span>
	{/if}
	{#if index}
		<span
			class="border px-1.5 py-0.5 tabular-nums {accent
				? 'border-amber-500/50 text-amber-500'
				: 'border-border text-foreground'}">{index}</span
		>
	{/if}
	<span>{label}</span>
	<span class="h-px min-w-8 flex-1 bg-border" aria-hidden="true"></span>
</p>
