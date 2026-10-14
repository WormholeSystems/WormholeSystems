<script lang="ts">
	// The body of the jump prompt, for one prompt: the parent remounts it (keyed on the
	// prompt) whenever a new jump arrives, so every field here seeds itself once.
	import SearchIcon from '@lucide/svelte/icons/search';

	import { toast } from 'svelte-sonner';
	import { copyText } from '$lib/clipboard';

	import type { Signature } from '$lib/api/types/Signature';
	import type { MassStatus } from '$lib/api/types/MassStatus';
	import type { TimeStatus } from '$lib/api/types/TimeStatus';
	import type { WormholeSize } from '$lib/api/types/WormholeSize';
	import { formatBookmark, formatsFromNaming } from '$lib/naming/bookmark';
	import ClassBadge from '$lib/components/ClassBadge.svelte';
	import { Button } from '$lib/components/ui/button';
	import * as Dialog from '$lib/components/ui/dialog';
	import { Input } from '$lib/components/ui/input';
	import { Label } from '$lib/components/ui/label';
	import { RadioGroup, RadioGroupItem } from '$lib/components/ui/radio-group';
	import * as Select from '$lib/components/ui/select';
	import * as Tooltip from '$lib/components/ui/tooltip';
	import { LIFETIME_OPTIONS, MASS_OPTIONS, SIZE_OPTIONS } from '$lib/map/connection-status';
	import { typeById } from '$lib/map/signatures';
	import { groupSignatures } from '$lib/signatures/compatibility';
	import { connectionDestination } from '$lib/signatures/destination';
	import { matchesSignatureQuery } from '$lib/signatures/search';
	import { cn } from '$lib/utils';
	import type { MapState } from '../../state/map-state.svelte';
	import type { JumpPrompt, JumpTracker } from '../../state/tracking.svelte';
	import { sizeForJumpMass } from '$lib/map/mass';

	let { map, tracker, prompt }: { map: MapState; tracker: JumpTracker; prompt: JumpPrompt } =
		$props();

	let search = $state('');
	// Preselecting makes jumping holes in scanner order one keystroke each, and a wrong
	// guess confirmed by reflex one wrong link; whoever it is asks for it.
	/* svelte-ignore state_referenced_locally */
	let selected = $state<number | null>(
		map.userSettings?.preselect_signature ? (prompt.groups.likely[0]?.id ?? null) : null,
	);
	/* svelte-ignore state_referenced_locally */
	let alias = $state(prompt.suggestedAlias ?? '');
	let time = $state<TimeStatus>('stable');
	let mass = $state<MassStatus>('stable');
	let size = $state<WormholeSize | 'auto'>('auto');

	const all = $derived([
		...prompt.groups.likely,
		...prompt.groups.connected,
		...prompt.groups.unlikely,
	]);
	const chosen = $derived(all.find((s) => s.id === selected) ?? null);

	function typeOf(signature: Signature | null) {
		return signature ? typeById(prompt.catalog, signature.signature_type_id) : null;
	}

	/** An identified hole dictates its own size, so the select is locked while one is picked. */
	const lockedSize = $derived(sizeForJumpMass(typeOf(chosen)?.max_jump_mass));

	// Only where the signature says something: a scanned "stable" must not overwrite a
	// lifetime just picked by hand.
	$effect(() => {
		const signature = chosen;
		if (!signature) return;
		const named = prompt.ghostAliases.get(signature.id);
		if (named) alias = named;
		if (signature.time_status && signature.time_status !== 'stable') time = signature.time_status;
		if (signature.mass_status && signature.mass_status !== 'stable') mass = signature.mass_status;
		if (!lockedSize && signature.size) size = signature.size;
	});

	const matching = $derived.by(() => {
		const query = search.trim();
		if (!query) return prompt.groups;
		return groupSignatures(
			all.filter((s) => matchesSignatureQuery(s, typeOf(s), query)),
			new Map(prompt.catalog.types.map((t) => [t.id, t])),
			prompt.targetClassId,
		);
	});

	const sections = $derived([
		{ key: 'likely', label: null, options: matching.likely },
		{ key: 'connected', label: 'Already connected', options: matching.connected },
		{ key: 'unlikely', label: 'Unlikely · leads elsewhere', options: matching.unlikely },
	]);

	/** Every selectable row in visual order, starting with Unknown. */
	const orderedIds = $derived<(number | null)[]>([
		null,
		...sections.flatMap((s) => s.options.map((o) => o.id)),
	]);

	let listEl = $state<HTMLElement | null>(null);

	function move(delta: number) {
		if (orderedIds.length === 0) return;
		const index = orderedIds.indexOf(selected);
		selected = orderedIds[(index + delta + orderedIds.length) % orderedIds.length];
		requestAnimationFrame(() =>
			listEl
				?.querySelector('[data-state="checked"]')
				?.closest('label')
				?.scrollIntoView({ block: 'nearest' }),
		);
	}

	function confirm() {
		const chosenAlias = alias.trim() || null;
		copyBookmark(chosenAlias);
		tracker.submit({
			origin: prompt.origin,
			targetSolarSystemId: prompt.targetSolarSystemId,
			signaturePk: selected,
			alias: chosenAlias,
			at: prompt.at,
			size: lockedSize ?? (size === 'auto' ? null : size),
			massStatus: mass,
			timeStatus: time,
		});
		tracker.dismiss();
	}

	/** The bookmark for the hole just flown, named after where it came out. */
	function copyBookmark(chosenAlias: string | null) {
		if (!map.userSettings?.copy_bookmark) return;
		const naming = map.data?.map.naming;
		const type = typeOf(chosen);
		const text = formatBookmark(
			{
				alias: chosenAlias,
				name: prompt.targetName,
				region: prompt.existing?.region ?? null,
				wormholeClassId: prompt.targetClassId,
				security: prompt.targetSecurity,
				occupier: prompt.existing?.occupying_group ?? null,
			},
			{
				signatureId: chosen?.signature_id ?? null,
				size: lockedSize ?? (size === 'auto' ? null : size),
				massStatus: mass,
				timeStatus: time,
				wormholeCode: type?.signature ?? null,
			},
			formatsFromNaming(naming),
			prompt.origin.alias,
		);
		void copyText(text, { silent: true });
		toast.success('Bookmark copied', { description: text });
	}

	/** Stop asking, and map the rest of this session's jumps unlinked. */
	function disablePrompt() {
		map.patchUserSettings({ prompt_for_signature: false });
		tracker.dismiss();
	}

	const SIZES: { value: WormholeSize | 'auto'; label: string; letter: string }[] = [
		{ value: 'auto', label: 'Auto', letter: '·' },
		...SIZE_OPTIONS,
	];

	const effectiveSize = $derived(lockedSize ?? size);
	const sizeOption = $derived(SIZES.find((o) => o.value === effectiveSize) ?? SIZES[0]);
	const originLabel = $derived(
		prompt.origin.alias ? `${prompt.origin.alias} · ${prompt.origin.name}` : prompt.origin.name,
	);
</script>

<Dialog.Header class="gap-1.5 border-b border-border/50 bg-muted/30 px-6 py-4 text-left">
	<Dialog.Title>Which signature did {prompt.pilot} jump?</Dialog.Title>
	<Dialog.Description>
		{prompt.pilot} jumped from <strong>{originLabel}</strong> to
		<strong data-testid="tracking-target">{prompt.targetName}</strong>
		<ClassBadge classId={prompt.targetClassId} security={prompt.targetSecurity} />.
	</Dialog.Description>
</Dialog.Header>

<form
	class="contents"
	onsubmit={(e) => {
		e.preventDefault();
		confirm();
	}}
>
	<div class="grid gap-3 px-6 py-5">
		<div class="grid grid-cols-3 gap-3">
			<div class="col-span-2 grid gap-1.5">
				<Label for="tracking-alias" class="text-xs">Alias</Label>
				<Input
					id="tracking-alias"
					bind:value={alias}
					placeholder="Optional system alias"
					data-testid="tracking-alias"
				/>
			</div>
			<div class="grid content-start gap-1.5">
				<Label class="text-xs">Ship size</Label>
				<Select.Root
					type="single"
					value={effectiveSize}
					disabled={lockedSize !== null}
					onValueChange={(v) => (size = v as WormholeSize | 'auto')}
				>
					<Select.Trigger class="w-full" data-testid="tracking-size">
						<span class="flex items-center gap-2">
							<span
								class="inline-flex w-6 justify-center font-mono text-[10px] text-muted-foreground"
							>
								{sizeOption.letter}
							</span>
							{sizeOption.label}
						</span>
					</Select.Trigger>
					<Select.Content>
						<Select.Group>
							{#each SIZES as option (option.value)}
								<Select.Item value={option.value} label={option.label}>
									<span class="flex items-center gap-2">
										<span
											class="inline-flex w-6 justify-center font-mono text-[10px] text-muted-foreground"
										>
											{option.letter}
										</span>
										{option.label}
									</span>
								</Select.Item>
							{/each}
						</Select.Group>
					</Select.Content>
				</Select.Root>
			</div>
		</div>

		<div class="grid grid-cols-2 gap-3">
			{#snippet statuses(
				label: string,
				options: { value: string; label: string; dot: string; hint: string | null }[],
				current: string,
				onchange: (value: string) => void,
				testid: string,
			)}
				<div class="grid gap-1.5">
					<Label class="text-xs">{label}</Label>
					<Select.Root type="single" value={current} onValueChange={onchange}>
						<Select.Trigger class="w-full" data-testid={testid}>
							<span class="flex items-center gap-2">
								<span
									class={cn(
										'inline-block size-2 rounded-full',
										options.find((o) => o.value === current)?.dot,
									)}
								></span>
								{options.find((o) => o.value === current)?.label}
							</span>
						</Select.Trigger>
						<Select.Content>
							<Select.Group>
								{#each options as option (option.value)}
									<Select.Item value={option.value} label={option.label}>
										<span class="flex items-center gap-2">
											<span class={cn('inline-block size-2 rounded-full', option.dot)}></span>
											{option.label}
											{#if option.hint}
												<span class="text-muted-foreground">{option.hint}</span>
											{/if}
										</span>
									</Select.Item>
								{/each}
							</Select.Group>
						</Select.Content>
					</Select.Root>
				</div>
			{/snippet}
			{@render statuses(
				'Lifetime',
				LIFETIME_OPTIONS,
				time,
				(v) => (time = v as TimeStatus),
				'tracking-lifetime',
			)}
			{@render statuses(
				'Mass',
				MASS_OPTIONS,
				mass,
				(v) => (mass = v as MassStatus),
				'tracking-mass',
			)}
		</div>
	</div>

	<div class="flex flex-col gap-3 px-6 pb-5">
		<div class="relative">
			<SearchIcon class="absolute top-1/2 left-3 size-4 -translate-y-1/2 text-muted-foreground" />
			<Input
				bind:value={search}
				placeholder="Search signatures"
				class="pl-9"
				autofocus
				data-testid="tracking-search"
				onkeydown={(e) => {
					if (e.key === 'ArrowDown') {
						e.preventDefault();
						move(1);
					} else if (e.key === 'ArrowUp') {
						e.preventDefault();
						move(-1);
					}
				}}
			/>
		</div>

		<div bind:this={listEl} class="h-64 overflow-y-auto">
			<RadioGroup
				value={selected === null ? '__unknown' : String(selected)}
				onValueChange={(v) => (selected = v === '__unknown' ? null : Number(v))}
				class="grid grid-cols-[auto_auto_auto_1fr] content-start gap-0 gap-x-4"
			>
				<label
					class="col-span-4 grid cursor-pointer grid-cols-subgrid items-center rounded-sm p-2 text-left text-xs hover:bg-muted/40 has-data-[state=checked]:bg-muted/60"
				>
					<RadioGroupItem value="__unknown" data-testid="tracking-unknown" />
					<div class="font-medium">Unknown</div>
					<div class="text-muted-foreground">—</div>
					<div></div>
				</label>

				{#each sections as section (section.key)}
					{#if section.label && section.options.length}
						<div
							class="col-span-4 mt-2 border-t border-border/50 px-2 pt-2.5 pb-1 font-mono text-[10px] tracking-wider text-muted-foreground uppercase"
						>
							{section.label}
						</div>
					{/if}
					{#each section.options as option (option.id)}
						{@const type = typeOf(option)}
						{@const leadsTo = connectionDestination(
							option,
							prompt.origin.id,
							map.connections.all,
							map.systems.all,
						)}
						<label
							class={cn(
								'col-span-4 grid cursor-pointer grid-cols-subgrid items-center rounded-sm p-2 text-left text-xs hover:bg-muted/40 has-data-[state=checked]:bg-muted/60 has-data-[state=checked]:opacity-100',
								section.key !== 'likely' && 'opacity-60',
							)}
							data-testid="tracking-option"
							data-sig={option.signature_id}
						>
							<RadioGroupItem value={String(option.id)} />
							<div class="font-medium">{option.signature_id}</div>
							{#if type}
								<span class="flex gap-2">
									<span class="inline-block w-[4ch] font-mono">{type.signature ?? ''}</span>
									{#if type.target_class !== null}
										<!-- A class id fully decides the badge, so the security fallback is unused. -->
										<ClassBadge classId={type.target_class} security={0} />
									{/if}
									{#if type.extra}
										<span class="text-muted-foreground">({type.extra})</span>
									{/if}
								</span>
							{:else if option.name}
								<div class="text-muted-foreground">{option.name}</div>
							{:else}
								<div class="text-muted-foreground">Unknown</div>
							{/if}
							<div class="truncate text-right text-muted-foreground">
								{#if leadsTo}→ {leadsTo}{/if}
							</div>
						</label>
					{/each}
				{/each}

				{#if search && orderedIds.length === 1}
					<div class="col-span-4 px-2 py-3 text-xs text-muted-foreground">
						No signatures match "{search}"
					</div>
				{/if}
			</RadioGroup>
		</div>
	</div>

	<Dialog.Footer class="border-t border-border/50 bg-muted/30 px-6 py-4 sm:justify-between">
		<Tooltip.Provider>
			<Tooltip.Root>
				<Tooltip.Trigger>
					{#snippet child({ props })}
						<Button
							{...props}
							type="button"
							variant="outline"
							onclick={disablePrompt}
							data-testid="tracking-disable">Stop asking</Button
						>
					{/snippet}
				</Tooltip.Trigger>
				<Tooltip.Content>Re-enable it from the tracking settings.</Tooltip.Content>
			</Tooltip.Root>
		</Tooltip.Provider>
		<Button type="submit" data-testid="tracking-confirm">Confirm</Button>
	</Dialog.Footer>
</form>
