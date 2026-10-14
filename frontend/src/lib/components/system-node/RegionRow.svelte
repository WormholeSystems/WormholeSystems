<script lang="ts">
	// The region name of a k-space system, on the node's bottom-left.
	let { region }: { region: string | null } = $props();
</script>

<div class="flex items-center text-[10px]">
	<span class="truncate text-muted-foreground">{region ?? ''}</span>
</div>
