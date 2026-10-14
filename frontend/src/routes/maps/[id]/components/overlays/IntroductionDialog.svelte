<script lang="ts">
	// The one-time walkthrough a map opens with: welcome, the ESI permissions, the preferences
	// that depend on them, and a summary.
	//
	// Granting one permission asks for everything already consented to as well, because SSO
	// reissues the token wholesale and a per-scope link would drop the rest. The settings step
	// disables what the missing scopes cannot support rather than offering dead switches.
	import Logo from '$lib/components/Logo.svelte';
	import ArrowLeftIcon from '@lucide/svelte/icons/arrow-left';
	import ArrowRightIcon from '@lucide/svelte/icons/arrow-right';
	import CheckCircleIcon from '@lucide/svelte/icons/check-circle-2';
	import ExternalLinkIcon from '@lucide/svelte/icons/external-link';

	import { createQuery } from '@tanstack/svelte-query';
	import { page } from '$app/state';

	import { q } from '$lib/api/queries';
	import type { UpdateMapUserSettings } from '$lib/api/types/UpdateMapUserSettings';
	import { Button } from '$lib/components/ui/button';
	import * as Dialog from '$lib/components/ui/dialog';
	import { Switch } from '$lib/components/ui/switch';
	import { cn } from '$lib/utils';
	import type { MapState } from '../../state/map-state.svelte';
	import { atLeast } from '$lib/map/roles';
	import { ESI_SCOPES } from '$lib/esi/scopes';
	import { INTRO_STEPS, INTRO_TIPS, introSummary, introToggles } from './introduction-content';
	import { PLACEMENTS } from '$lib/map/placement';
	import { automationPatch } from '$lib/map/user-settings';

	let { map, onfinished }: { map: MapState; onfinished?: () => void } = $props();

	let step = $state(1);
	const scopesQuery = createQuery(() => ({ ...q.myScopes(), enabled: map.signedIn }));
	const scopes = $derived(scopesQuery.data ?? []);

	const granted = $derived(new Set(scopes.filter((s) => s.granted).map((s) => s.scope)));
	const missing = $derived(ESI_SCOPES.filter((s) => !granted.has(s.scope)));
	const hasLocation = $derived(granted.has('esi-location.read_location.v1'));

	const settings = $derived(map.userSettings);
	// Closed locally the moment it is dismissed, rather than waiting for the settings round
	// trip: without that the dialog is still open when the close handler fires again.
	let closed = $state(false);
	const open = $derived(!closed && !(settings?.introduction_confirmed ?? true));

	/** Consent for these on top of everything already granted, then come back here. */
	function grantUrl(wanted: string[]) {
		const params = new URLSearchParams({
			scopes: wanted.join(','),
			return_to: page.url.pathname + page.url.search,
		});
		return `/auth/login?${params}`;
	}

	// Placement is the map's, not this viewer's. Only the owner is offered it here:
	// everyone else would meet a choice they cannot make.
	const isOwner = $derived(map.data?.role === 'owner');
	const placement = $derived(map.data?.map.layout === 'tree' ? 'tree' : 'manual');

	function update(patch: UpdateMapUserSettings) {
		map.patchUserSettings(patch);
	}

	function finish() {
		closed = true;
		update({ introduction_confirmed: true });
		onfinished?.();
	}

	const summary = $derived(
		introSummary(settings, ESI_SCOPES.length - missing.length, ESI_SCOPES.length),
	);

	const toggles = $derived(introToggles(settings, hasLocation));
</script>

<Dialog.Root {open} onOpenChange={(v) => !v && open && finish()}>
	<!-- A fixed height, so stepping through never resizes the dialog; each step scrolls
	     inside instead. -->
	<Dialog.Content
		class="h-[40rem] max-h-[85vh] w-full grid-rows-[auto_minmax(0,1fr)_auto] md:max-w-2xl"
		data-testid="introduction"
	>
		<div class="-mx-4 flex flex-col gap-3 border-b border-border/40 px-4 pb-3">
			<Dialog.Header>
				<span class="font-mono text-[10px] tracking-wider text-muted-foreground uppercase">
					Step {step} of {INTRO_STEPS.length}
				</span>
				<Dialog.Title class="font-heading text-xl">{INTRO_STEPS[step - 1].title}</Dialog.Title>
				<Dialog.Description class="text-sm">{INTRO_STEPS[step - 1].blurb}</Dialog.Description>
			</Dialog.Header>

			<div class="flex items-center gap-1.5" data-testid="introduction-progress">
				{#each INTRO_STEPS as _, i (i)}
					<div class={cn('h-1 flex-1', i < step ? 'bg-primary' : 'bg-muted')}></div>
				{/each}
			</div>
		</div>

		<div class="overflow-y-auto">
			{#if step === 1}
				<div class="flex flex-col gap-4">
					<p class="text-sm leading-relaxed text-muted-foreground">
						WormholeSystems builds the chain from where your characters actually are. Each
						permission unlocks a piece of that; all are optional, and revocable at any time.
					</p>
					<div class="flex flex-col gap-2">
						{#each ESI_SCOPES as item (item.scope)}
							{@const Icon = item.icon}
							{@const ok = granted.has(item.scope)}
							<div
								class="flex items-center gap-3 border border-border/60 p-2.5"
								data-scope={item.scope}
							>
								<span
									class={cn(
										'flex size-8 shrink-0 items-center justify-center rounded-md',
										ok ? 'bg-emerald-500/10 text-emerald-500' : 'bg-muted/40 text-muted-foreground',
									)}
								>
									<Icon class="size-4" />
								</span>
								<span class="min-w-0 flex-1">
									<span class="text-sm font-medium">{item.name}</span>
									<p class="mt-0.5 text-xs leading-relaxed text-muted-foreground">{item.body}</p>
								</span>
								{#if ok}
									<span
										class="flex shrink-0 items-center gap-1 text-xs text-emerald-500"
										data-testid="scope-granted"
									>
										<CheckCircleIcon class="size-3.5" />
										Granted
									</span>
								{:else}
									<Button
										variant="outline"
										size="sm"
										href={grantUrl([item.scope])}
										class="shrink-0"
									>
										<ExternalLinkIcon data-icon="inline-start" />
										Grant
									</Button>
								{/if}
							</div>
						{/each}
					</div>
				</div>
			{:else if step === 2}
				<div class="flex flex-col gap-3.5">
					{#if isOwner}
						<div class="flex flex-col gap-2" data-testid="setup-placement">
							<div class="flex items-baseline justify-between gap-3">
								<span class="font-mono text-[10px] tracking-wider text-muted-foreground uppercase">
									Chain placement
								</span>
								<span class="text-[11px] text-muted-foreground">
									For everyone on this map. Changeable later in the map's settings.
								</span>
							</div>
							<div class="grid grid-cols-2 gap-2">
								{#each PLACEMENTS as option (option.value)}
									{@const Icon = option.icon}
									{@const chosen = placement === option.value}
									<button
										type="button"
										class={cn(
											'flex flex-col gap-1.5 border p-2.5 text-left transition-colors',
											chosen
												? 'border-primary/60 bg-primary/5'
												: 'border-border/60 hover:bg-accent/20',
										)}
										aria-pressed={chosen}
										data-testid="setup-placement-{option.value}"
										onclick={() => map.placement.setForMap(option.value)}
									>
										<span class="flex w-full items-center gap-2">
											<span
												class={cn(
													'flex size-8 shrink-0 items-center justify-center rounded-md',
													chosen
														? 'bg-primary/10 text-primary'
														: 'bg-muted/40 text-muted-foreground',
												)}
											>
												<Icon class="size-4" />
											</span>
											<span class="text-sm font-medium">{option.label}</span>
											{#if chosen}
												<CheckCircleIcon class="ml-auto size-4 shrink-0 text-primary" />
											{/if}
										</span>
										<span class="text-xs leading-relaxed text-muted-foreground">{option.body}</span>
									</button>
								{/each}
							</div>
						</div>
					{/if}

					<div class="flex flex-col gap-2">
						<span class="font-mono text-[10px] tracking-wider text-muted-foreground uppercase">
							Automation
						</span>
						{#each toggles as row (row.setting.key)}
							{@const Icon = row.setting.icon}
							<div
								class={cn('border border-border/60 p-2.5', !row.enabled && 'opacity-60')}
								data-setting={row.setting.key}
							>
								<div class="flex items-center gap-3">
									<span
										class={cn(
											'flex size-8 shrink-0 items-center justify-center rounded-md',
											row.value && row.enabled
												? 'bg-primary/10 text-primary'
												: 'bg-muted/40 text-muted-foreground',
										)}
									>
										<Icon class="size-4" />
									</span>
									<span class="min-w-0 flex-1">
										<span class="text-sm font-medium">{row.setting.label}</span>
										<p class="mt-0.5 text-xs leading-relaxed text-muted-foreground">
											{row.setting.body}
										</p>
										{#if row.blocked}
											<p class="mt-1 text-xs text-amber-500">{row.blocked}</p>
										{/if}
									</span>
									<Switch
										checked={row.value && row.enabled}
										disabled={!row.enabled}
										aria-label={row.setting.label}
										onCheckedChange={(v) => update(automationPatch(row.setting.key, v))}
									/>
								</div>
							</div>
						{/each}
					</div>
				</div>
			{:else}
				<div class="flex h-full flex-col items-center justify-center gap-6 text-center">
					<div
						class="flex size-16 items-center justify-center rounded-full border border-primary/30 bg-primary/10"
					>
						<Logo class="size-8 text-primary" />
					</div>
					<div class="flex flex-col gap-1">
						<p class="font-heading text-2xl">The chain awaits.</p>
						<p class="text-sm text-muted-foreground">The map is set up: fly, and it follows.</p>
					</div>
					<div class="flex w-full max-w-sm flex-col gap-1.5 border border-border/60 p-3 text-xs">
						{#each summary as row (row.label)}
							<span class="flex items-center justify-between">
								<span class="text-muted-foreground">{row.label}</span>
								<span class={row.good ? 'text-emerald-500' : 'text-amber-500'}>{row.value}</span>
							</span>
						{/each}
					</div>
					<div class="flex w-full max-w-sm flex-col gap-2">
						{#each INTRO_TIPS as tip, i (i)}
							{@const Icon = tip.icon}
							<span class="flex items-center gap-2.5 text-left text-xs text-muted-foreground">
								<Icon class="size-3.5 shrink-0" />
								{tip.text}
							</span>
						{/each}
					</div>
				</div>
			{/if}
		</div>

		<div class="-mx-4 flex items-center justify-between border-t border-border/40 px-4 pt-3">
			<Button
				variant="outline"
				size="sm"
				disabled={step === 1}
				onclick={() => (step -= 1)}
				data-testid="introduction-back"
			>
				<ArrowLeftIcon data-icon="inline-start" />
				Back
			</Button>
			<div class="flex gap-2">
				{#if step === 1 && missing.length > 0}
					<Button size="sm" href={grantUrl(missing.map((s) => s.scope))}>Grant all</Button>
				{/if}
				{#if step < INTRO_STEPS.length}
					<Button
						size="sm"
						variant={step === 1 && missing.length > 0 ? 'outline' : 'default'}
						onclick={() => (step += 1)}
						data-testid="introduction-next"
					>
						{step === 1 && missing.length > 0 ? 'Skip for now' : 'Next'}
						<ArrowRightIcon data-icon="inline-end" />
					</Button>
				{:else}
					<Button size="sm" onclick={finish} data-testid="introduction-done">Start mapping</Button>
				{/if}
			</div>
		</div>
	</Dialog.Content>
</Dialog.Root>
