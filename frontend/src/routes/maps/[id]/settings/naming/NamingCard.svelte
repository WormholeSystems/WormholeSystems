<script lang="ts">
	// The alias sequence and the three bookmark formats. The formats are opaque token strings,
	// so each previews against a worked example as you type.
	import { oneOf } from '$lib/lookup';

	import { guessNextAlias } from '$lib/naming/alias';
	import { draft as draftOf } from '$lib/draft.svelte';
	import type { AliasScheme } from '$lib/naming/alias';
	import {
		BOOKMARK_TOKENS,
		DEFAULT_FORMAT_KSPACE,
		DEFAULT_FORMAT_RETURN,
		DEFAULT_FORMAT_WORMHOLE,
		DEFAULT_IGNORED_ALIAS,
		renderBookmark,
	} from '$lib/naming/bookmark';
	import type { BookmarkToken } from '$lib/naming/bookmark';
	import type { MapNaming } from '$lib/api/types/MapNaming';
	import { Button } from '$lib/components/ui/button';
	import * as Card from '$lib/components/ui/card';
	import * as Field from '$lib/components/ui/field';
	import { Input } from '$lib/components/ui/input';
	import * as ToggleGroup from '$lib/components/ui/toggle-group';

	let {
		naming,
		disabled = false,
		onsave,
	}: {
		naming: MapNaming;
		disabled?: boolean;
		onsave: (naming: MapNaming) => void;
	} = $props();

	const buffer = draftOf(() => naming);
	const dirty = $derived(buffer.dirty);

	const ALIAS_SCHEMES = ['numeric', 'alphabetical'] as const satisfies readonly AliasScheme[];
	const scheme = $derived(buffer.value.alias_scheme);

	/** The first three children of a chain, so switching scheme shows the difference. */
	const aliasPreview = $derived.by(() => {
		const taken: string[] = [];
		for (let i = 0; i < 3; i++) {
			taken.push(guessNextAlias('', taken, { scheme, ignoredAlias: buffer.value.ignored_alias }));
		}
		const child = guessNextAlias(taken[0], taken, {
			scheme,
			ignoredAlias: buffer.value.ignored_alias,
		});
		return [...taken, child].join(', ');
	});

	const EXAMPLE = {
		alias: '1a',
		current: '1',
		sig: 'ABC',
		class: 'C5',
		name: 'J155207',
		region: 'E-R00028',
		occupier: 'Hard Knocks',
		size: 'MD',
		wh: 'H296',
		mass: 'crit',
		life: 'EOL',
	} satisfies Record<BookmarkToken, string>;

	const KSPACE_EXAMPLE = {
		...EXAMPLE,
		alias: '1b',
		class: 'HS',
		name: 'Jita',
		region: 'The Forge',
		occupier: '',
		wh: 'B041',
	} satisfies Record<BookmarkToken, string>;

	/** Back up the chain, so the two aliases swap. */
	const RETURN_EXAMPLE = {
		...EXAMPLE,
		alias: '1',
		current: '1a',
	} satisfies Record<BookmarkToken, string>;

	const FORMATS = [
		{
			key: 'bookmark_wormhole' as const,
			label: 'Wormhole',
			help: 'A hole leading further into wormhole space.',
			fallback: DEFAULT_FORMAT_WORMHOLE,
			example: EXAMPLE,
		},
		{
			key: 'bookmark_kspace' as const,
			label: 'K-space',
			help: 'A hole leading out to known space.',
			fallback: DEFAULT_FORMAT_KSPACE,
			example: KSPACE_EXAMPLE,
		},
		{
			key: 'bookmark_return' as const,
			label: 'Return',
			help: 'The hole back the way you came.',
			fallback: DEFAULT_FORMAT_RETURN,
			example: RETURN_EXAMPLE,
		},
	];
</script>

<Card.Root>
	<Card.Header>
		<Card.Title>Chain naming</Card.Title>
		<Card.Description>
			How aliases and bookmarks are generated for everyone on this map.
		</Card.Description>
	</Card.Header>
	<Card.Content>
		<Field.FieldGroup>
			<Field.Field>
				<Field.FieldLabel>Alias scheme</Field.FieldLabel>
				<ToggleGroup.Root
					type="single"
					variant="outline"
					value={buffer.value.alias_scheme}
					{disabled}
					onValueChange={(value) => {
						const picked = oneOf(ALIAS_SCHEMES, value);
						if (picked) buffer.value.alias_scheme = picked;
					}}
					data-testid="alias-scheme"
				>
					<ToggleGroup.Item value="numeric">Numeric</ToggleGroup.Item>
					<ToggleGroup.Item value="alphabetical">Alphabetical</ToggleGroup.Item>
				</ToggleGroup.Root>
				<Field.FieldDescription>
					Chains start <span class="font-mono" data-testid="alias-preview">{aliasPreview}</span>
					{#if scheme === 'alphabetical'}
						. H, L, N and P are reserved for exits to known space.
					{/if}
				</Field.FieldDescription>
			</Field.Field>

			<Field.Field>
				<Field.FieldLabel for="ignored-alias">Home alias</Field.FieldLabel>
				<Input
					id="ignored-alias"
					bind:value={buffer.value.ignored_alias}
					placeholder={DEFAULT_IGNORED_ALIAS}
					{disabled}
					data-testid="ignored-alias"
				/>
				<Field.FieldDescription>
					Sits outside the chain: its holes start a fresh sequence, and anything pointing back at it
					is bookmarked as a way home.
				</Field.FieldDescription>
			</Field.Field>

			<Field.FieldSet>
				<Field.FieldLegend>Bookmark formats</Field.FieldLegend>
				<Field.FieldDescription>
					Available: {BOOKMARK_TOKENS.map((t) => `{${t}}`).join(' ')}. Empty ones drop out.
				</Field.FieldDescription>
				<Field.FieldGroup>
					{#each FORMATS as format (format.key)}
						<Field.Field>
							<Field.FieldLabel for={format.key}>{format.label}</Field.FieldLabel>
							<Input
								id={format.key}
								bind:value={buffer.value[format.key]}
								placeholder={format.fallback}
								{disabled}
								class="font-mono"
								data-testid={format.key}
							/>
							<Field.FieldDescription>
								{format.help}
								<span class="font-mono text-foreground" data-testid="{format.key}-preview">
									{renderBookmark(buffer.value[format.key] || format.fallback, format.example)}
								</span>
							</Field.FieldDescription>
						</Field.Field>
					{/each}
				</Field.FieldGroup>
			</Field.FieldSet>
		</Field.FieldGroup>
	</Card.Content>
	<Card.Footer class="justify-end gap-2">
		<Button variant="ghost" disabled={disabled || !dirty} onclick={() => buffer.reset()}>
			Reset
		</Button>
		<Button
			variant="outline"
			disabled={disabled || !dirty}
			onclick={() => onsave({ ...buffer.value })}
			data-testid="save-naming">Save</Button
		>
	</Card.Footer>
</Card.Root>
