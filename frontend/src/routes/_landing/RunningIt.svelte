<script lang="ts">
	// The long version of self-hosting: what setup checks before it asks for anything, and
	// the rest of wsctl, so the story is the whole life of an install rather than day one.
	import CheckIcon from '@lucide/svelte/icons/check';

	import Band from './Band.svelte';
	import Reveal from './Reveal.svelte';
	import SectionLabel from './SectionLabel.svelte';

	let { index, tone = 'plain' }: { index: string; tone?: 'plain' | 'tint' } = $props();

	const checks = [
		'Docker is installed and running, on Compose v2',
		'there is disk for the static data',
		'the domain actually resolves to this machine',
		'ports 80 and 443 are free',
		'the checkout is not behind origin',
	];

	const commands = [
		{ cmd: 'setup', body: 'Checks the machine, asks for what it needs, brings the stack up.' },
		{
			cmd: 'update',
			body: 'Pulls, rebuilds, restarts, and takes newer static data when CCP has one.',
		},
		{ cmd: 'status', body: 'What is running, which SDE build is loaded, whether the URL answers.' },
		{ cmd: 'doctor', body: 'Checks Docker, disk, ports and DNS without changing anything.' },
		{ cmd: 'discord-register', body: 'Uploads the slash command to your Discord application.' },
	];
</script>

<Band id="running-it" {tone} labelledby="running-it-heading">
	<Reveal>
		<div class="grid gap-10 lg:grid-cols-12 lg:gap-14">
			<div class="lg:col-span-5 xl:col-span-4">
				<SectionLabel {index} label="Running it" />
				<h2
					id="running-it-heading"
					class="mt-6 font-heading text-3xl font-semibold tracking-tight text-balance sm:text-4xl"
				>
					Setup checks first, wsctl after
				</h2>
				<p class="mt-4 text-muted-foreground">
					The installer puts wsctl on the machine and offers to run the setup right away.
					Certificates are issued on the first boot and the static data seeds itself.
				</p>
			</div>

			<div
				class="grid gap-px border border-border bg-border lg:col-span-7 xl:col-span-8 xl:grid-cols-2"
			>
				<div class="bg-background p-6">
					<p class="font-mono text-[10px] tracking-[0.2em] text-muted-foreground uppercase">
						Checked before it asks for anything
					</p>
					<ul class="mt-4 flex flex-col gap-2.5 text-sm text-muted-foreground">
						{#each checks as check (check)}
							<li class="flex items-baseline gap-3">
								<CheckIcon class="size-3.5 shrink-0 translate-y-0.5 text-emerald-500" />
								{check}
							</li>
						{/each}
					</ul>
				</div>
				<div class="bg-background p-6">
					<p class="font-mono text-[10px] tracking-[0.2em] text-muted-foreground uppercase">
						And afterwards
					</p>
					<dl class="mt-4 flex flex-col gap-3 text-sm">
						{#each commands as item (item.cmd)}
							<div class="flex flex-col gap-0.5">
								<dt class="font-mono text-xs text-amber-500/90">wsctl {item.cmd}</dt>
								<dd class="text-xs text-muted-foreground">{item.body}</dd>
							</div>
						{/each}
					</dl>
				</div>
			</div>
		</div>
	</Reveal>
</Band>
