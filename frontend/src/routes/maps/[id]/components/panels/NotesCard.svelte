<script lang="ts">
	// Per-system notes, in markdown. Member-gated: a viewer gets a 403 from the details
	// endpoint and the panel disappears.
	import PencilIcon from '@lucide/svelte/icons/pencil';
	import { marked } from 'marked';

	import { createQuery } from '@tanstack/svelte-query';

	import { ApiError } from '$lib/api/client';
	import { q } from '$lib/api/queries';
	import type { MapSystemView } from '$lib/api/types/MapSystemView';
	import { Button } from '$lib/components/ui/button';
	import { Textarea } from '$lib/components/ui/textarea';
	import MapPanel from '$lib/components/map-panel/MapPanel.svelte';
	import MapPanelContent from '$lib/components/map-panel/MapPanelContent.svelte';
	import MapPanelHeader from '$lib/components/map-panel/MapPanelHeader.svelte';
	import type { MapState } from '../../state/map-state.svelte';

	let {
		map,
		system,
	}: {
		map: MapState;
		system: MapSystemView;
	} = $props();

	let editing = $state(false);
	let draft = $state('');

	// Keyed by the placement id, so the wholesale replacement of `map.systems.all` on every
	// refetch never re-asks or closes the editor; only looking at a different system does.
	const details = createQuery(() => q.systemDetails(map.mapId, system.id));
	const notes = $derived(details.data?.notes ?? null);
	// Member-gated: a viewer's 403 (or a 404) hides the panel rather than erroring.
	const hidden = $derived(
		details.error instanceof ApiError &&
			(details.error.status === 403 || details.error.status === 404),
	);

	$effect(() => {
		// The id is a value, so this fires once per distinct system, not per refetch.
		void system.id;
		editing = false;
	});

	const rendered = $derived.by(() => {
		if (!notes) return '';
		const html = marked.parse(notes, { async: false, breaks: true }) as string;
		return html.replaceAll('<a href', '<a target="_blank" rel="noopener" href');
	});

	function startEdit() {
		draft = notes ?? '';
		editing = true;
	}

	function save() {
		const value = draft.trim() || null;
		map.systems.saveNotes(system.id, value);
		editing = false;
	}
</script>

{#if !hidden}
	<MapPanel testid="notes-card">
		<MapPanelHeader>
			Notes
			{#snippet actions()}
				{#if editing}
					<Button variant="ghost" class="h-5 px-1.5 text-[10px]" onclick={() => (editing = false)}>
						Cancel
					</Button>
					<Button
						variant="ghost"
						class="h-5 px-1.5 text-[10px] text-amber-400 hover:text-amber-400"
						onclick={save}
					>
						Save
					</Button>
				{:else}
					<Button
						variant="ghost"
						size="icon"
						class="size-5 text-muted-foreground hover:text-foreground"
						aria-label="Edit notes"
						onclick={startEdit}
					>
						<PencilIcon class="size-3" />
					</Button>
				{/if}
			{/snippet}
		</MapPanelHeader>
		<MapPanelContent>
			{#if editing}
				<Textarea
					class="min-h-40 w-full resize-none rounded-none border-0 bg-transparent px-3 py-2 font-mono text-xs shadow-none focus-visible:ring-0"
					bind:value={draft}
					placeholder="Add notes..."
				/>
			{:else if rendered}
				<!-- eslint-disable-next-line svelte/no-at-html-tags -->
				<div
					class="prose prose-sm max-w-none px-3 py-2 prose-invert prose-headings:my-2 prose-headings:text-foreground prose-p:my-2 prose-a:text-amber-500 prose-a:no-underline hover:prose-a:underline prose-code:rounded prose-code:bg-muted prose-code:px-1 prose-code:py-0.5 prose-code:text-amber-400 prose-code:before:content-none prose-code:after:content-none prose-pre:border prose-pre:border-border/50 prose-pre:bg-muted prose-ol:my-2 prose-ul:my-2 prose-li:my-0.5"
				>
					{@html rendered}
				</div>
			{:else}
				<div class="flex flex-col items-center justify-center gap-2 p-4">
					<p class="font-mono text-[10px] tracking-wider text-muted-foreground/60 uppercase">
						No notes
					</p>
				</div>
			{/if}
		</MapPanelContent>
	</MapPanel>
{/if}
