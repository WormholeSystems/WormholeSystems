<script lang="ts">
	import '../app.css';

	import { browser } from '$app/environment';
	import { page } from '$app/state';
	import { QueryCache, QueryClient, QueryClientProvider } from '@tanstack/svelte-query';
	import { toast } from 'svelte-sonner';

	import { errorMessage } from '$lib/api/client';
	import AnnouncementBanner from '$lib/components/AnnouncementBanner.svelte';
	import ConfirmDialog from '$lib/components/ConfirmDialog.svelte';
	import Nav from '$lib/components/Nav.svelte';
	import Seo from '$lib/components/Seo.svelte';
	import { Toaster } from '$lib/components/ui/sonner';
	import * as Tooltip from '$lib/components/ui/tooltip';

	let { children, data } = $props();

	// One client for the whole app, created here and never in a load (a re-run load would
	// build a fresh client and silently wipe the cache). Defaults match what the hand-rolled
	// fetching did, not TanStack's: nothing retried, nothing refetched on focus, and queries
	// only ever run in the browser because the api client uses relative URLs (server paint
	// comes from the load functions). An `enabled:` override replaces that default, so any
	// hand-written gate on a page that can SSR must include `browser` itself.
	const queryClient = new QueryClient({
		defaultOptions: {
			queries: {
				enabled: browser,
				staleTime: 30_000,
				retry: false,
				refetchOnWindowFocus: false,
			},
		},
		queryCache: new QueryCache({
			onError: (err, query) => {
				if (query.meta?.toastOnError) toast.error(`load: ${errorMessage(err)}`);
			},
		}),
	});

	// The map page and the landing page run edge to edge and set their own padding; every
	// other page wants the usual frame.
	const flush = $derived(page.url.pathname === '/' || /^\/maps\/\d+$/.test(page.url.pathname));

	// Marks the page as interactive; e2e tests wait for this before clicking, since
	// SSR-rendered controls are dead until hydration attaches their handlers.
	$effect(() => {
		document.documentElement.dataset.hydrated = 'true';
	});
</script>

<Seo />

<QueryClientProvider client={queryClient}>
	<!-- The app's one tooltip provider: every tooltip waits a second, then the next one along
	     shows at once, and a mouse click is never taken as asking for one. -->
	<Tooltip.Provider delayDuration={1000} ignoreNonKeyboardFocus>
		<Toaster position="top-center" closeButton />
		<ConfirmDialog />
		<Nav me={data.me} maps={data.maps} status={data.status} />
		<AnnouncementBanner announcement={data.announcement} />
		<main class={flush ? '' : 'p-6'}>
			{@render children()}
		</main>
	</Tooltip.Provider>
</QueryClientProvider>
