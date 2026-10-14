<script lang="ts">
	// The External submenu: every site that knows about this system, grouped by site.
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
	<ContextMenu.SubContent class="w-48" data-testid="external-submenu">
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
