<script lang="ts">
	import { page } from '$app/state';

	import { Button } from '$lib/components/ui/button';
	import * as Card from '$lib/components/ui/card';

	// The SSO callback sends a character the instance does not let in back here, before
	// anything about it is stored.
	const refused = $derived(page.url.searchParams.get('refused') === '1');
</script>

<div class="grid min-h-[60vh] place-items-center">
	<Card.Root class="w-full max-w-sm text-center">
		<Card.Header>
			<Card.Title>Sign in</Card.Title>
			<Card.Description>Authenticate with your EVE Online character.</Card.Description>
		</Card.Header>
		<Card.Content class="flex flex-col gap-4">
			{#if refused}
				<p
					class="border border-destructive/50 bg-destructive/10 px-3 py-2.5 text-sm text-destructive"
					role="alert"
					data-testid="login-refused"
				>
					This instance is limited to certain characters, corporations or alliances, and the
					character you signed in with is not one of them.
				</p>
			{/if}
			<Button href="/auth/login" data-sveltekit-reload class="w-full">
				Log in with EVE Online
			</Button>
		</Card.Content>
	</Card.Root>
</div>
