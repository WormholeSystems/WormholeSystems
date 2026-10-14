<script lang="ts">
	// The External submenu in bits-ui chrome; the on-map node menu renders the same
	// groups through its own chrome.
	import ExternalLinkIcon from '@lucide/svelte/icons/external-link';

	import * as ContextMenu from '$lib/components/ui/context-menu';
	import type { SystemLinkGroup } from '$lib/map/external-links';

	let { groups }: { groups: SystemLinkGroup[] } = $props();

	const LABEL = 'text-[0.65rem] font-semibold tracking-wider text-muted-foreground uppercase';
</script>

<ContextMenu.Sub>
	<ContextMenu.SubTrigger data-testid="menu-external">
		<ExternalLinkIcon class="size-4" />
		External
	</ContextMenu.SubTrigger>
	<ContextMenu.SubContent class="w-48">
		{#each groups as group, i (group.label)}
			{#if i > 0}
				<ContextMenu.Separator />
			{/if}
			<ContextMenu.Label class="flex items-center gap-2 {LABEL}">
				<img src={group.favicon} alt="" class="size-3.5 rounded-sm" />
				{group.label}
			</ContextMenu.Label>
			{#each group.links as link (link.label)}
				<ContextMenu.Item>
					{#snippet child({ props })}
						<a {...props} target="_blank" rel="noopener" href={link.href}>
							<link.icon class="size-4" />
							{link.label}
						</a>
					{/snippet}
				</ContextMenu.Item>
			{/each}
		{/each}
	</ContextMenu.SubContent>
</ContextMenu.Sub>
