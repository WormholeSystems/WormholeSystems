<script lang="ts">
	// Account settings: the things that are true of you wherever you are, not of one map.
	import IdCardIcon from '@lucide/svelte/icons/id-card';
	import KeyRoundIcon from '@lucide/svelte/icons/key-round';
	import MonitorIcon from '@lucide/svelte/icons/monitor';
	import MessageCircleIcon from '@lucide/svelte/icons/message-circle';

	import SettingsShell from '$lib/components/settings/SettingsShell.svelte';
	import type { Section } from '$lib/components/settings/SettingsShell.svelte';

	let { children }: { children: import('svelte').Snippet } = $props();

	const sections: Section[] = [
		{
			href: '/settings/characters',
			label: 'Characters',
			description: 'Who you fly, and what they let WormholeSystems see',
			icon: IdCardIcon,
		},
		{
			href: '/settings/display',
			label: 'Display',
			description: 'How the site looks to you, on every map',
			icon: MonitorIcon,
		},
		{
			href: '/settings/discord',
			label: 'Discord',
			description: 'Linking, and what the bot answers',
			icon: MessageCircleIcon,
		},
		{
			href: '/settings/tokens',
			label: 'API tokens',
			description: 'Keys for scripts that act as you',
			icon: KeyRoundIcon,
		},
	];
</script>

<SettingsShell
	title="Settings"
	subtitle="Your account, across every map."
	back={{ href: '/maps', label: 'Back to your maps' }}
	{sections}
>
	{@render children()}
</SettingsShell>
