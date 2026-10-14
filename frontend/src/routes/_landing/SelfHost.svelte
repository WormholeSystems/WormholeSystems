<script lang="ts">
	// The self-hosting band. This is the reason to pick this over somebody else's copy, so
	// it gets its own lit band and the most room on the page.
	import CheckIcon from '@lucide/svelte/icons/check';
	import CopyIcon from '@lucide/svelte/icons/copy';
	import { copyText } from '$lib/clipboard';
	import DatabaseIcon from '@lucide/svelte/icons/database';
	import GitForkIcon from '@lucide/svelte/icons/git-fork';
	import LockIcon from '@lucide/svelte/icons/lock';
	import ServerIcon from '@lucide/svelte/icons/server';

	import Reveal from './Reveal.svelte';

	const command = "curl --proto '=https' --tlsv1.2 -sSf https://install-next.wormhole.systems | sh";
	let copied = $state(false);
	let resetTimer: ReturnType<typeof setTimeout> | undefined;

	function copy() {
		void copyText(command, { silent: true });
		copied = true;
		clearTimeout(resetTimer);
		resetTimer = setTimeout(() => (copied = false), 2000);
	}

	// What setup checks before it asks for a single credential.
	const checks = [
		'Docker is installed and running, on Compose v2',
		'there is disk for the static data',
		'the domain actually resolves to this machine',
		'ports 80 and 443 are free',
		'the checkout is not behind origin',
	];

	// The rest of wsctl, so the story is the whole life of the install, not day one.
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

	const owns = [
		{
			icon: ServerIcon,
			title: 'Your machine',
			body: 'One box, four containers, no account with anybody.',
		},
		{
			icon: DatabaseIcon,
			title: 'Your database',
			body: 'The chain lives in your Postgres. Nothing phones home.',
		},
		{
			icon: LockIcon,
			title: 'Your keys',
			body: 'Your own EVE application, so ESI tokens never leave the host.',
		},
	];
</script>

<section id="self-host" class="relative overflow-hidden border-t border-border bg-card/40">
	<div class="glow pointer-events-none absolute inset-0" aria-hidden="true"></div>
	<div class="relative mx-auto w-full max-w-6xl px-6 py-24 sm:py-28">
		<Reveal>
			<div class="max-w-2xl">
				<p
					class="flex items-center gap-2 font-mono text-[10px] tracking-[0.2em] text-amber-500 uppercase"
				>
					<GitForkIcon class="size-3.5" />
					Open source · Self-hosted
				</p>
				<h2 class="mt-4 font-heading text-4xl font-semibold tracking-tight sm:text-5xl">
					Run it yourself.<br />It is one command.
				</h2>
				<p class="mt-5 text-muted-foreground">
					Not a trial of a hosted thing. The whole application, on your own machine, with your
					corp's chain in a database only you can reach.
				</p>
			</div>

			<div class="mt-12 grid gap-10 lg:grid-cols-[1.05fr_1fr] lg:gap-16">
				<div>
					<div
						class="flex items-center gap-3 rounded border border-amber-500/40 bg-background p-4 font-mono text-sm"
					>
						<span class="shrink-0 text-muted-foreground select-none">$</span>
						<span class="min-w-0 flex-1 break-all">{command}</span>
						<button
							class="shrink-0 text-muted-foreground transition-colors hover:text-foreground"
							aria-label="Copy the install command"
							data-testid="copy-command"
							onclick={copy}
						>
							{#if copied}
								<CheckIcon class="size-4 text-emerald-500" />
							{:else}
								<CopyIcon class="size-4" />
							{/if}
						</button>
					</div>
					<p class="mt-4 text-sm text-muted-foreground">
						The installer puts wsctl on the machine and offers to run the setup right away.
						Postgres, the API, the web server and TLS come up together under Docker. Certificates
						are issued on the first boot and the static data seeds itself. It asks for your domain
						and your EVE application, and works the rest out.
					</p>

					<div class="mt-8 grid gap-px border border-border bg-border sm:grid-cols-3">
						{#each owns as item (item.title)}
							{@const Icon = item.icon}
							<div class="flex flex-col gap-2 bg-background p-4">
								<Icon class="size-4 text-amber-500" />
								<h3 class="text-sm font-medium">{item.title}</h3>
								<p class="text-xs text-muted-foreground">{item.body}</p>
							</div>
						{/each}
					</div>
				</div>

				<div class="flex flex-col gap-8">
					<div>
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

					<div>
						<p class="font-mono text-[10px] tracking-[0.2em] text-muted-foreground uppercase">
							And afterwards
						</p>
						<dl class="mt-4 flex flex-col gap-2.5 text-sm">
							{#each commands as item (item.cmd)}
								<div class="flex flex-col gap-0.5 sm:flex-row sm:gap-3">
									<dt class="w-40 shrink-0 font-mono text-xs text-amber-500/90">
										wsctl {item.cmd}
									</dt>
									<dd class="text-xs text-muted-foreground">{item.body}</dd>
								</div>
							{/each}
						</dl>
					</div>
				</div>
			</div>
		</Reveal>
	</div>
</section>

<style>
	.glow {
		background: radial-gradient(
			52rem 26rem at 12% 0%,
			color-mix(in oklab, var(--color-amber-500) 10%, transparent),
			transparent 70%
		);
	}
</style>
