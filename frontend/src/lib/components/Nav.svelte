<script lang="ts">
	import LogOutIcon from '@lucide/svelte/icons/log-out';
	import SettingsIcon from '@lucide/svelte/icons/settings';
	import MapIcon from '@lucide/svelte/icons/map';
	import PlusIcon from '@lucide/svelte/icons/plus';
	import Trash2Icon from '@lucide/svelte/icons/trash-2';
	import TriangleAlertIcon from '@lucide/svelte/icons/triangle-alert';

	import { createQuery } from '@tanstack/svelte-query';

	import { browser } from '$app/environment';
	import { page } from '$app/state';
	import { q } from '$lib/api/queries';
	import { createIdentityActions } from '$lib/characters/identity';
	import { scopeMeta } from '$lib/esi/scopes';
	import type { CharacterSummary } from '$lib/api/types/CharacterSummary';
	import type { MapEntry } from '$lib/api/types/MapEntry';
	import type { ServerStatus as ServerState } from '$lib/api/types/ServerStatus';
	import { Button } from '$lib/components/ui/button';
	import * as DropdownMenu from '$lib/components/ui/dropdown-menu';
	import EveImage from '$lib/components/EveImage.svelte';
	import ServerStatus from '$lib/components/ServerStatus.svelte';
	import Logo from '$lib/components/Logo.svelte';
	import ThemeToggle from '$lib/components/ThemeToggle.svelte';
	import Hint from '$lib/components/Hint.svelte';

	let {
		me,
		maps = [],
		status = null,
	}: { me: CharacterSummary | null; maps?: MapEntry[]; status?: ServerState | null } = $props();

	// The shortcuts start from the list the layout already loads; after that the cache owns
	// it, so a pin toggled on /maps shows up here without a navigation.
	const mapsQuery = createQuery(() => ({
		...q.myMaps(),
		enabled: browser && me !== null,
		initialData: maps.length > 0 ? maps : undefined,
	}));
	const pinned = $derived((mapsQuery.data ?? maps).filter((m) => m.is_pinned && !m.is_archived));

	const charactersQuery = createQuery(() => ({
		...q.myCharacters(),
		enabled: browser && me !== null,
	}));
	const characters = $derived(charactersQuery.data ?? []);
	// A character short of an ESI permission quietly does less (no location, no waypoints),
	// so the portrait says so rather than leaving it to be noticed.
	const lacking = $derived(characters.filter((c) => c.missing_scopes.length > 0));
	const activeLacks = $derived(lacking.some((c) => c.is_active));
	function missingNames(scopes: string[]): string {
		return scopes.map((s) => scopeMeta(s)?.name ?? s).join(', ');
	}

	// Three fit without the names being squeezed to nothing; the rest go behind a count.
	const INLINE = 3;
	const inline = $derived(pinned.slice(0, INLINE));
	const overflow = $derived(pinned.slice(INLINE));
	const here = $derived(page.url.pathname);

	const identity = createIdentityActions();
</script>

<nav class="sticky top-0 z-40 border-b border-border bg-background">
	<!-- Three columns so the middle holds its place however long the shortcut list grows.
	     Below md there is no middle, and the left side gets all the room the right does
	     not need, rather than an even half that a phone cannot fit the wordmark into. -->
	<div
		class="grid h-12 grid-cols-[minmax(0,1fr)_auto] items-center gap-3 px-4 sm:gap-4 sm:px-5 md:grid-cols-[1fr_auto_1fr]"
	>
		<div class="flex min-w-0 items-center gap-3 sm:gap-4">
			<a href="/" class="flex items-center gap-2 text-foreground">
				<Logo class="size-5" />
				<span class="font-heading text-sm font-semibold tracking-tight">WormholeSystems</span>
				<!-- Says what it is before anyone finds out the hard way. -->
				<Hint
					text={'Beta: in daily use, still changing. Updates migrate your data.'}
					focusable={false}
				>
					{#snippet child({ props })}
						<span
							class="border border-amber-500/40 px-1.5 py-0.5 text-[10px] tracking-wider whitespace-nowrap text-amber-500 uppercase"
							data-testid="beta"
							{...props}
						>
							Beta
						</span>
					{/snippet}
				</Hint>
			</a>
			<!-- Without an account this only leads to the sign-in page, so it is not shown. -->
			{#if me}
				<a
					href="/maps"
					class="flex items-center gap-1.5 text-sm transition-colors {here === '/maps'
						? 'text-foreground'
						: 'text-muted-foreground hover:text-foreground'}"
				>
					<MapIcon class="size-4" />
					<!-- On a phone the icon alone, so the wordmark and the badge keep their line. -->
					<span class="sr-only sm:not-sr-only">Maps</span>
				</a>
			{/if}

			<span class="hidden min-w-0 items-center gap-1 lg:flex" data-testid="pinned-maps">
				{#each inline as map (map.id)}
					<Hint text={map.name}>
						{#snippet child({ props })}
							<a
								href="/maps/{map.id}"
								class="flex max-w-44 min-w-0 items-center gap-1.5 px-2 py-1 text-xs transition-colors {here ===
								`/maps/${map.id}`
									? 'bg-muted/50 text-foreground'
									: 'text-muted-foreground hover:bg-muted/30 hover:text-foreground'}"
								data-testid="pinned-map"
								{...props}
							>
								<MapIcon class="size-3.5 shrink-0" />
								<span class="truncate">{map.name}</span>
							</a>
						{/snippet}
					</Hint>
				{/each}
				{#if overflow.length > 0}
					<DropdownMenu.Root>
						<DropdownMenu.Trigger
							class="px-2 py-1 text-xs text-muted-foreground transition-colors hover:text-foreground"
							data-testid="pinned-overflow"
						>
							+{overflow.length}
						</DropdownMenu.Trigger>
						<DropdownMenu.Content align="start" class="w-56">
							{#each overflow as map (map.id)}
								<DropdownMenu.Item>
									{#snippet child({ props })}
										<a href="/maps/{map.id}" {...props}>
											<MapIcon />
											<span class="truncate">{map.name}</span>
										</a>
									{/snippet}
								</DropdownMenu.Item>
							{/each}
						</DropdownMenu.Content>
					</DropdownMenu.Root>
				{/if}
			</span>
		</div>

		<span class="hidden justify-self-center md:block">
			<ServerStatus signedIn={me !== null} initial={status} />
		</span>

		<span class="flex items-center justify-end gap-3">
			<a
				href="/documentation"
				class="hidden text-sm transition-colors sm:block {here.startsWith('/documentation')
					? 'text-foreground'
					: 'text-muted-foreground hover:text-foreground'}"
				data-testid="nav-docs"
			>
				Docs
			</a>
			<ThemeToggle />
			{#if me}
				<DropdownMenu.Root>
					<DropdownMenu.Trigger
						aria-label={lacking.length > 0
							? 'Account: characters are missing permissions'
							: 'Account'}
						class="relative block size-7 border border-border transition-colors hover:border-foreground/50"
					>
						<!-- Clipped on its own, so the warning can sit over the corner. -->
						<span class="block size-full overflow-hidden">
							<EveImage kind="character" id={me.character_id} class="size-full object-cover" />
						</span>
						{#if lacking.length > 0}
							<Hint
								text={`${lacking.map((c) => c.name).join(', ')} ${
									lacking.length === 1 ? 'is' : 'are'
								} missing permissions`}
								focusable={false}
								label
							>
								{#snippet child({ props })}
									<span
										class="absolute -right-1.5 -bottom-1.5 flex size-4 items-center justify-center rounded-full bg-background"
										{...props}
										data-testid="missing-scopes-warning"
									>
										<TriangleAlertIcon class="size-3.5 text-red-500" />
									</span>
								{/snippet}
							</Hint>
						{/if}
					</DropdownMenu.Trigger>
					<DropdownMenu.Content align="end">
						<DropdownMenu.Group>
							{#each characters as c (c.character_id)}
								<DropdownMenu.Item
									class={c.is_active ? 'text-foreground' : ''}
									onSelect={() => identity.switchTo(c.character_id)}
								>
									<EveImage
										kind="character"
										id={c.character_id}
										class="size-5 border border-border"
									/>
									<span class="truncate">{c.name}</span>
									{#if c.needs_login}
										<Hint text={'EVE signed this character out'} focusable={false}>
											{#snippet child({ props })}
												<span class="ml-auto shrink-0 text-xs text-amber-500" {...props}>
													Log in again
												</span>
											{/snippet}
										</Hint>
									{/if}
									{#if c.missing_scopes.length > 0}
										<Hint
											text={`Missing: ${missingNames(c.missing_scopes)}${
												c.is_active ? '' : '. Switch to it to grant them.'
											}`}
											focusable={false}
											label
										>
											{#snippet child({ props })}
												<span
													class="shrink-0 {c.needs_login ? '' : 'ml-auto'}"
													{...props}
													data-testid="character-missing-scopes"
												>
													<!-- Important: a highlighted row repaints everything in it. -->
													<TriangleAlertIcon class="size-3.5 text-red-500!" />
												</span>
											{/snippet}
										</Hint>
									{/if}
								</DropdownMenu.Item>
							{/each}
						</DropdownMenu.Group>

						{#if activeLacks}
							<DropdownMenu.Separator />
							<DropdownMenu.Item>
								{#snippet child({ props })}
									<a href="/settings/characters" data-testid="grant-missing-scopes" {...props}>
										<TriangleAlertIcon class="text-red-500!" />
										Grant missing permissions
									</a>
								{/snippet}
							</DropdownMenu.Item>
						{/if}

						<DropdownMenu.Separator />

						<DropdownMenu.Group>
							<DropdownMenu.Item>
								{#snippet child({ props })}
									<a href="/auth/login?link=true" data-sveltekit-reload {...props}>
										<PlusIcon />
										Add character
									</a>
								{/snippet}
							</DropdownMenu.Item>

							{#if characters.length > 1}
								<DropdownMenu.Item
									variant="destructive"
									onSelect={() => identity.remove(me.character_id, true)}
								>
									<Trash2Icon />
									Remove {me.name}
								</DropdownMenu.Item>
							{/if}
						</DropdownMenu.Group>

						<DropdownMenu.Separator />

						<DropdownMenu.Group>
							<DropdownMenu.Item>
								{#snippet child({ props })}
									<a href="/settings/characters" {...props} data-testid="nav-settings">
										<SettingsIcon />
										Settings
									</a>
								{/snippet}
							</DropdownMenu.Item>
							<DropdownMenu.Item>
								{#snippet child({ props })}
									<a href="/auth/logout" data-sveltekit-reload {...props}>
										<LogOutIcon />
										Log out
									</a>
								{/snippet}
							</DropdownMenu.Item>
						</DropdownMenu.Group>
					</DropdownMenu.Content>
				</DropdownMenu.Root>
			{:else}
				<Button href="/auth/login" data-sveltekit-reload variant="outline">Log in</Button>
			{/if}
		</span>
	</div>
</nav>
