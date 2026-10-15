import adapter from '@sveltejs/adapter-node';
import { sveltekit } from '@sveltejs/kit/vite';
import tailwindcss from '@tailwindcss/vite';
import { defineConfig } from 'vitest/config';

// In dev the Axum API runs separately; proxy the backend paths so the whole app
// (including the EVE SSO flow, Discord linking and both WebSockets) lives on the vite
// origin.
const backend = 'http://127.0.0.1:3000';

export default defineConfig({
	plugins: [
		tailwindcss(),
		sveltekit({
			compilerOptions: {
				// Force runes mode for the project, except for libraries. Can be removed in svelte 6.
				runes: ({ filename }) =>
					filename.split(/[/\\]/).includes('node_modules') ? undefined : true,
			},
			adapter: adapter(),
			// Sent as a header on every rendered page. Nothing is prerendered, so `auto`
			// always picks a fresh nonce, which SvelteKit puts on its own scripts and
			// app.html puts on the theme script.
			csp: {
				mode: 'auto',
				directives: {
					'default-src': ['self'],
					'script-src': ['self'],
					// The markup carries `style="..."` attributes (Svelte's `style:`, server
					// rendered, and positions from bits-ui), which a nonce cannot cover.
					'style-src': ['self', 'unsafe-inline'],
					// Portraits, logos and type icons come from EVE's image server; `data:`
					// and `blob:` are images made in the page.
					'img-src': ['self', 'data:', 'blob:', 'https://images.evetech.net'],
					// `self` covers ws: and wss: on the same host, which is where both
					// sockets connect.
					'connect-src': ['self'],
					'object-src': ['none'],
					'base-uri': ['self'],
					'frame-ancestors': ['none'],
					// Every form here is handled in script. Signing in goes to EVE and
					// Discord by redirect, which form-action does not restrict.
					'form-action': ['self'],
				},
			},
		}),
	],
	server: {
		proxy: {
			'/api': backend,
			'/auth': backend,
			'/discord': backend,
			'/ws': { target: backend, ws: true },
		},
	},
	test: {
		// Unit tests cover pure logic only. `e2e/` is Playwright's and must stay out of
		// vitest's reach, or it collects those specs and fails on the wrong runner.
		include: ['src/**/*.test.ts'],
	},
});
