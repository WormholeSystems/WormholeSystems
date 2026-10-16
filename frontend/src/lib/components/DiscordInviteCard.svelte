<script lang="ts">
	// The call to add the bot to a server. Everything but a webhook needs it there: it can only
	// post to channels of servers it is in, and Discord only lets it DM people it shares one with.
	// Dressed as Discord on purpose, so it reads as "this goes to Discord" at a glance; its
	// colours are Discord's and the same in both themes.
	let { inviteUrl }: { inviteUrl: string } = $props();

	const LOGO =
		'M20.317 4.37a19.79 19.79 0 0 0-4.885-1.515.074.074 0 0 0-.079.037c-.21.375-.444.865-.608 1.25a18.27 18.27 0 0 0-5.487 0 12.64 12.64 0 0 0-.617-1.25.077.077 0 0 0-.079-.037A19.74 19.74 0 0 0 3.677 4.37a.07.07 0 0 0-.032.027C.533 9.046-.32 13.58.099 18.057a.082.082 0 0 0 .031.057 19.9 19.9 0 0 0 5.993 3.03.078.078 0 0 0 .084-.028c.462-.63.874-1.295 1.226-1.994a.076.076 0 0 0-.041-.106 13.1 13.1 0 0 1-1.872-.892.077.077 0 0 1-.008-.128c.126-.094.252-.192.372-.291a.074.074 0 0 1 .078-.01c3.928 1.793 8.18 1.793 12.062 0a.074.074 0 0 1 .078.01c.12.099.246.198.373.292a.077.077 0 0 1-.006.127 12.3 12.3 0 0 1-1.873.892.077.077 0 0 0-.041.107c.36.698.772 1.362 1.225 1.993a.076.076 0 0 0 .084.028 19.84 19.84 0 0 0 6.002-3.03.077.077 0 0 0 .032-.054c.5-5.177-.838-9.674-3.549-13.66a.061.061 0 0 0-.031-.03ZM8.02 15.33c-1.183 0-2.157-1.085-2.157-2.419 0-1.333.956-2.419 2.157-2.419 1.21 0 2.176 1.096 2.157 2.42 0 1.333-.956 2.418-2.157 2.418Zm7.975 0c-1.183 0-2.157-1.085-2.157-2.419 0-1.333.955-2.419 2.157-2.419 1.21 0 2.176 1.096 2.157 2.42 0 1.333-.946 2.418-2.157 2.418Z';
</script>

<section
	class="relative isolate overflow-hidden rounded-lg bg-[#5865F2] p-6 text-white shadow-sm"
	data-testid="discord-invite"
>
	<!-- The mascot, large and faded, bleeding off the corner. -->
	<svg
		viewBox="0 0 24 24"
		aria-hidden="true"
		class="pointer-events-none absolute -right-6 -bottom-8 -z-10 size-44 fill-white/10"
	>
		<path d={LOGO} />
	</svg>

	<div class="flex flex-col gap-4 sm:max-w-[34rem]">
		<div class="flex items-center gap-3">
			<span class="grid size-10 shrink-0 place-items-center rounded-full bg-white/15">
				<svg viewBox="0 0 24 24" aria-hidden="true" class="size-5 fill-white">
					<path d={LOGO} />
				</svg>
			</span>
			<div class="flex flex-col">
				<h3 class="text-base leading-tight font-bold">Add the bot to your Discord server</h3>
				<span class="text-xs text-white/75">WormholeSystems · Discord app</span>
			</div>
		</div>

		<p class="text-sm leading-relaxed text-white/90">
			Alerts that post as the bot or send you a direct message only work once the bot is in your
			server. It also brings
			<code class="rounded bg-black/20 px-1 py-0.5 font-mono text-[0.8em]">/wh</code>
			to everyone there.
		</p>

		<div class="flex flex-wrap items-center gap-x-4 gap-y-2">
			<a
				href={inviteUrl}
				target="_blank"
				rel="noopener"
				class="inline-flex h-10 items-center gap-2 rounded-[4px] bg-white px-4 text-sm font-semibold text-[#5865F2] transition-colors hover:bg-[#f2f3ff] focus-visible:ring-2 focus-visible:ring-white focus-visible:ring-offset-2 focus-visible:ring-offset-[#5865F2] focus-visible:outline-none active:bg-[#e3e5ff]"
			>
				<svg viewBox="0 0 24 24" aria-hidden="true" class="size-4 fill-current">
					<path d={LOGO} />
				</svg>
				Add to Server
			</a>
			<span class="text-xs text-white/70">
				Needs Manage Server. Channel webhooks work without the bot.
			</span>
		</div>
	</div>
</section>
