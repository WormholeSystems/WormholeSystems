<script lang="ts" module>
	/** What a page's load returns under `seo` to describe its own card. */
	export interface SeoData {
		/** Prefixed to the site name in the tab and the card heading. */
		title?: string;
		description?: string;
		/** Absolute, or a path on this host. */
		image?: string;
		type?: 'website' | 'article';
	}
</script>

<script lang="ts">
	// The head tags a link preview is built from: Open Graph for most places, Twitter's
	// card tags for the rest.
	//
	// Rendered once, by the root layout. A page says what it wants by returning `seo` from
	// its load, rather than rendering a second copy of this: two of these on a page means
	// two <title> tags, and the browser takes the first, which is the one the page was
	// trying to replace.
	//
	// Ported from the legacy app's SeoHead, with one difference: the URLs come from the
	// request rather than being fixed at wormhole.systems. Every copy of this is somebody
	// else's server, and a card pointing at a site the reader cannot reach, showing an image
	// that host never serves, is worse than no card at all.
	import { page } from '$app/state';

	const SITE = 'WormholeSystems';
	const DESCRIPTION =
		'Wormhole mapping and tracking for EVE Online. One live chain map for your corp: signatures, connection mass and lifetime, and everyone’s position from ESI.';

	const seo = $derived((page.data.seo ?? {}) as SeoData);
	// An error page has no load of its own to name it, so the status does.
	const title = $derived(seo.title ?? (page.error ? `Error ${page.status}` : undefined));
	const heading = $derived(title ? `${title} · ${SITE}` : SITE);
	const description = $derived(seo.description ?? DESCRIPTION);
	const image = $derived(seo.image ?? '/og.png');
	const imageUrl = $derived(image.startsWith('http') ? image : `${page.url.origin}${image}`);
</script>

<svelte:head>
	<title>{heading}</title>
	<meta name="description" content={description} />

	<meta property="og:title" content={heading} />
	<meta property="og:description" content={description} />
	<meta property="og:url" content={page.url.href} />
	<meta property="og:type" content={seo.type ?? 'website'} />
	<meta property="og:site_name" content={SITE} />
	<meta property="og:locale" content="en_US" />
	<meta property="og:image" content={imageUrl} />
	<meta property="og:image:type" content="image/png" />
	<meta property="og:image:width" content="1024" />
	<meta property="og:image:height" content="768" />

	<meta name="twitter:card" content="summary_large_image" />
	<meta name="twitter:title" content={heading} />
	<meta name="twitter:description" content={description} />
	<meta name="twitter:image" content={imageUrl} />
</svelte:head>
