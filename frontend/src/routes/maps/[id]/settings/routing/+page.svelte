<script lang="ts">
	// How routes are chosen. The same settings the route planner's popover edits, in a form
	// with room to say what each one costs you, and below them the map's own ignore list.
	import { createQuery } from '@tanstack/svelte-query';
	import {
		ROUTE_LIFETIMES,
		ROUTE_MASSES,
		ROUTE_PREFS,
		routeChoicePatch,
		type RouteChoiceField,
		type RouteOption,
	} from '$lib/routing/options';
	import { q } from '$lib/api/queries';
	import SettingRow from '$lib/components/settings/SettingRow.svelte';
	import * as Card from '$lib/components/ui/card';
	import * as Select from '$lib/components/ui/select';
	import { Slider } from '$lib/components/ui/slider';
	import { Switch } from '$lib/components/ui/switch';
	import IgnoredSystemsCard from './IgnoredSystemsCard.svelte';
	import { mapSettings } from '../context.svelte';

	const ctx = mapSettings();
	const mapId = $derived(ctx.mapId);
	const settings = $derived(ctx.settings);
	const ignoredQuery = createQuery(() => q.listIgnored(mapId));
	const canManage = $derived(ctx.canManage);

	const saveUserSettings = ctx.saveUserSettings;

	function choose(field: RouteChoiceField, value: string) {
		const patch = routeChoicePatch(field, value);
		if (patch) saveUserSettings(patch);
	}

	const full = <T,>(o: RouteOption<T>) => ({ value: o.value, label: o.label, hint: o.hint });
	const PREFERENCES = ROUTE_PREFS.map(full);
	const LIFETIMES = ROUTE_LIFETIMES.map(full);
	const MASSES = ROUTE_MASSES.map(full);

	const preference = $derived(settings?.route_preference ?? 'shorter');
	const lifetime = $derived(settings?.route_allow_time_status ?? 'critical');
	const mass = $derived(settings?.route_allow_mass_status ?? 'reduced');
	const penalty = $derived(settings?.security_penalty ?? 50);
</script>

{#snippet picker(
	options: { value: string; label: string; hint: string }[],
	current: string,
	key: RouteChoiceField,
	testid: string,
)}
	<Select.Root type="single" value={current} onValueChange={(v) => choose(key, v)}>
		<Select.Trigger class="w-56" data-testid={testid}>
			{options.find((o) => o.value === current)?.label}
		</Select.Trigger>
		<Select.Content>
			<Select.Group>
				{#each options as option (option.value)}
					<Select.Item value={option.value} label={option.label}>
						<span class="flex flex-col">
							<span>{option.label}</span>
							<span class="text-xs text-muted-foreground">{option.hint}</span>
						</span>
					</Select.Item>
				{/each}
			</Select.Group>
		</Select.Content>
	</Select.Root>
{/snippet}

<div class="flex flex-col gap-6">
	<Card.Root>
		<Card.Header>
			<Card.Title>Routing</Card.Title>
			<Card.Description>
				How the route planner and every jump count on the map are worked out. Yours alone.
			</Card.Description>
		</Card.Header>
		<Card.Content class="flex flex-col py-0">
			<SettingRow
				id="route-preference"
				label="Route preference"
				description="Shortest counts jumps and nothing else. The other two bias the search towards or away from high security, at the cost of extra jumps."
			>
				{#snippet control()}
					{@render picker(PREFERENCES, preference, 'route_preference', 'route-preference')}
				{/snippet}
			</SettingRow>

			<SettingRow
				id="security-penalty"
				label="How hard to avoid, or seek, low security"
				description="Only applies to the safer and less-secure preferences. At zero they behave like shortest; at a hundred they will take a long way round to get what they want."
				disabled={preference === 'shorter'}
				blocked={preference === 'shorter' ? 'Shortest ignores security entirely.' : undefined}
			>
				{#snippet control()}
					<span class="flex items-center gap-3">
						<Slider
							type="single"
							min={0}
							max={100}
							step={5}
							value={penalty}
							disabled={preference === 'shorter'}
							aria-label="Security penalty"
							class="w-40"
							data-testid="security-penalty"
							onValueCommit={(v) => saveUserSettings({ security_penalty: v })}
						/>
						<span class="w-10 text-right font-mono text-xs tabular-nums">{penalty}%</span>
					</span>
				{/snippet}
			</SettingRow>

			<SettingRow
				id="route-lifetime"
				label="Wormholes to route through, by lifetime"
				description="An end-of-life hole may close while you are in it. This decides whether the router offers one anyway."
			>
				{#snippet control()}
					{@render picker(LIFETIMES, lifetime, 'route_allow_time_status', 'route-lifetime')}
				{/snippet}
			</SettingRow>

			<SettingRow
				id="route-mass"
				label="Wormholes to route through, by mass"
				description="A hole that has already passed most of its mass may not take your ship."
			>
				{#snippet control()}
					{@render picker(MASSES, mass, 'route_allow_mass_status', 'route-mass')}
				{/snippet}
			</SettingRow>

			<SettingRow
				id="route-wormholes"
				label="Use chain connections"
				description="Routes may go through this map's own wormholes. Turn it off to count stargate jumps only, so a route never sends you back through the chain."
			>
				{#snippet control()}
					<Switch
						checked={settings?.route_use_wormholes ?? true}
						aria-label="Use chain connections"
						onCheckedChange={(v) => saveUserSettings({ route_use_wormholes: v })}
					/>
				{/snippet}
			</SettingRow>

			<SettingRow
				id="route-evescout"
				label="Use EVE Scout connections"
				description="Routes may go through the public Thera and Turnur holes. They are scouted by hand and can be stale, which is why this is a choice."
			>
				{#snippet control()}
					<Switch
						checked={settings?.route_use_evescout ?? false}
						aria-label="Use EVE Scout connections"
						onCheckedChange={(v) => saveUserSettings({ route_use_evescout: v })}
					/>
				{/snippet}
			</SettingRow>
		</Card.Content>
	</Card.Root>

	<IgnoredSystemsCard {mapId} ignored={ignoredQuery.data ?? []} {canManage} />
</div>
