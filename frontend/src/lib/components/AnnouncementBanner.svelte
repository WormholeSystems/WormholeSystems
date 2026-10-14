<script lang="ts">
	// The operator's site-wide banner, above every page including the landing page.
	import ArrowRightIcon from '@lucide/svelte/icons/arrow-right';
	import InfoIcon from '@lucide/svelte/icons/info';
	import OctagonAlertIcon from '@lucide/svelte/icons/octagon-alert';
	import TriangleAlertIcon from '@lucide/svelte/icons/triangle-alert';
	import XIcon from '@lucide/svelte/icons/x';

	import { dismissalCookie } from '$lib/announcement';
	import type { Announcement } from '$lib/api/types/Announcement';
	import type { AnnouncementLevel } from '$lib/api/types/AnnouncementLevel';
	import { cn } from '$lib/utils';

	let { announcement }: { announcement: Announcement | null } = $props();

	// The server already leaves out one dismissed earlier; this covers the page it was
	// dismissed on, until the next full load sends the cookie.
	let dismissedId = $state<string | null>(null);
	const shown = $derived(announcement && announcement.id !== dismissedId ? announcement : null);

	const icons = {
		info: InfoIcon,
		warning: TriangleAlertIcon,
		critical: OctagonAlertIcon,
	} satisfies Record<AnnouncementLevel, unknown>;

	const tones: Record<AnnouncementLevel, string> = {
		info: 'border-sky-500/30 bg-sky-500/10 text-sky-950 dark:text-sky-100',
		warning: 'border-amber-500/30 bg-amber-500/10 text-amber-950 dark:text-amber-100',
		critical: 'border-red-500/40 bg-red-500/15 text-red-950 dark:text-red-100',
	};

	function dismiss(id: string) {
		document.cookie = dismissalCookie(id, location.protocol === 'https:');
		dismissedId = id;
	}
</script>

{#if shown}
	{@const Icon = icons[shown.level]}
	<div
		role="status"
		class={cn('border-b px-5 py-2.5', tones[shown.level])}
		data-testid="announcement"
		data-level={shown.level}
	>
		<div class="mx-auto flex max-w-7xl items-start gap-3 text-sm">
			<Icon class="mt-0.5 size-4 shrink-0" />
			<div class="min-w-0 flex-1">
				{#if shown.title}
					<p class="font-semibold">{shown.title}</p>
				{/if}
				<p class="text-pretty opacity-90">{shown.message}</p>
				{#if shown.link}
					<a
						href={shown.link.url}
						class="mt-1 inline-flex items-center gap-1 font-medium underline underline-offset-4 hover:no-underline"
					>
						{shown.link.label}
						<ArrowRightIcon class="size-3.5" />
					</a>
				{/if}
			</div>
			{#if shown.dismissible}
				<button
					type="button"
					aria-label="Dismiss announcement"
					class="-m-1 shrink-0 p-1 opacity-70 transition hover:opacity-100"
					onclick={() => dismiss(shown.id)}
					data-testid="announcement-dismiss"
				>
					<XIcon class="size-4" />
				</button>
			{/if}
		</div>
	</div>
{/if}
