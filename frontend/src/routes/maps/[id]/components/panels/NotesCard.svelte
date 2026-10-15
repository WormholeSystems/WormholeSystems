<script lang="ts">
	// Per-system notes, in markdown. Member-gated: a viewer gets a 403 from the details
	// endpoint and the panel disappears.
	import PencilIcon from '@lucide/svelte/icons/pencil';

	import { createQuery } from '@tanstack/svelte-query';

	import { ApiError } from '$lib/api/client';
	import { q } from '$lib/api/queries';
	import { renderMarkdown } from '$lib/markdown';
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
	// The notes as they were when the editor opened. The save sends them along, and the
	// server refuses it when someone else has saved since.
	let base = $state<string | null>(null);
	// Someone else saved while this editor was open: theirs is in `notes`, ours in `draft`.
	let conflict = $state(false);

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
		conflict = false;
	});

	// Anyone with write access to the map writes these, so they are sanitized before
	// they reach `{@html}`. The map page does not render on the server, so DOMPurify
	// always has the browser's DOM.
	const rendered = $derived(notes ? renderMarkdown(notes, { breaks: true }) : '');

	function startEdit() {
		draft = notes ?? '';
		base = notes;
		conflict = false;
		editing = true;
	}

	async function save() {
		const value = draft.trim() || null;
		const id = system.id;
		editing = false;
		conflict = false;
		const outcome = await map.systems.saveNotes(id, value, base);
		// Their notes are coming back from the server; the draft is still here, so the
		// editor reopens on it with the choice. Unless the card has moved on meanwhile.
		if (outcome === 'conflict' && system.id === id && !editing) {
			editing = true;
			conflict = true;
		}
	}

	/** Drop the draft for what the other person saved, and keep editing from there. */
	function loadTheirs() {
		startEdit();
	}

	/** Save the draft over theirs, now that it is known what gets replaced. */
	function overwrite() {
		base = notes;
		void save();
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
			{#if editing && conflict}
				<div
					class="flex flex-wrap items-center gap-2 border-b border-amber-500/30 bg-amber-500/10 px-3 py-1.5 text-[11px]"
					role="alert"
					data-testid="notes-conflict"
				>
					<span class="grow text-amber-200">Someone else edited these notes while you were.</span>
					<!-- Both act on their notes, so both wait until those have been read back. -->
					<Button
						variant="ghost"
						class="h-5 px-1.5 text-[10px]"
						disabled={details.isFetching}
						onclick={loadTheirs}
					>
						Load theirs
					</Button>
					<Button
						variant="ghost"
						class="h-5 px-1.5 text-[10px] text-amber-400 hover:text-amber-400"
						disabled={details.isFetching}
						onclick={overwrite}
					>
						Overwrite
					</Button>
				</div>
			{/if}
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
