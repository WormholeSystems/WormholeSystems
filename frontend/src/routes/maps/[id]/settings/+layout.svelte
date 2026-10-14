<script lang="ts">
	// A map's settings, split by who a setting belongs to: General, Access and Alerts change
	// the map for everyone and are Manager+, Display, Mapping and Routing are per viewer.
	// Sections you cannot use are hidden rather than shown and refused.
	//
	// Naming holds both, so it stays visible to everyone read-only: what your bookmarks will
	// say is worth reading even when you cannot change it.
	import ArrowDownUpIcon from '@lucide/svelte/icons/arrow-down-up';
	import BellIcon from '@lucide/svelte/icons/bell';
	import TagIcon from '@lucide/svelte/icons/tag';
	import CrosshairIcon from '@lucide/svelte/icons/crosshair';
	import EyeIcon from '@lucide/svelte/icons/eye';
	import RouteIcon from '@lucide/svelte/icons/route';
	import SettingsIcon from '@lucide/svelte/icons/settings';
	import UsersIcon from '@lucide/svelte/icons/users';

	import { page } from '$app/state';
	import type { MapView } from '$lib/api/types/MapView';
	import SettingsShell from '$lib/components/settings/SettingsShell.svelte';
	import type { Section } from '$lib/components/settings/SettingsShell.svelte';
	import { provideMapSettings } from './context.svelte';

	let { children, data }: { children: import('svelte').Snippet; data: { view: MapView } } =
		$props();

	const ctx = provideMapSettings(
		() => Number(page.params.id) || 0,
		() => data.view,
	);
	const mapId = $derived(ctx.mapId);
	const canManage = $derived(ctx.canManage);

	const sections = $derived.by<Section[]>(() => {
		const base = `/maps/${mapId}/settings`;
		const everyone: Section[] = [
			{
				href: `${base}/display`,
				label: 'Display',
				description: 'What the map shows you',
				icon: EyeIcon,
			},
			{
				href: `${base}/mapping`,
				label: 'Mapping',
				description: 'Tracking and scanning',
				icon: CrosshairIcon,
			},
			{
				href: `${base}/naming`,
				label: 'Naming',
				description: 'Aliases and bookmarks',
				icon: TagIcon,
			},
			{
				href: `${base}/routing`,
				label: 'Routing',
				description: 'How routes are chosen',
				icon: RouteIcon,
			},
		];
		if (!canManage) return everyone;
		return [
			{
				href: base,
				label: 'General',
				description: 'Name, description, deletion',
				icon: SettingsIcon,
			},
			...everyone,
			{
				href: `${base}/access`,
				label: 'Access',
				description: 'Who can see and edit',
				icon: UsersIcon,
			},
			{
				href: `${base}/alerts`,
				label: 'Discord alerts',
				description: 'What gets announced',
				icon: BellIcon,
			},
			{
				href: `${base}/transfer`,
				label: 'Import and export',
				description: 'Move the map as a file',
				icon: ArrowDownUpIcon,
			},
		];
	});
</script>

<SettingsShell
	title={`${ctx.view.map.name} settings`}
	subtitle="Some of these change the map for everyone on it; the rest are only yours."
	back={{ href: `/maps/${mapId}`, label: 'Back to the map' }}
	{sections}
>
	{@render children()}
</SettingsShell>
