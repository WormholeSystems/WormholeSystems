import { defineConfig } from '@playwright/test';

import e2e from './playwright.config';

// The demo is written straight into the tables as well as through the API, so both have to
// be the e2e database, never the one a dev stack keeps its real maps in.
process.env.DATABASE_URL ??= 'postgres://wormholesystems:wormholesystems@localhost:5432/ws_e2e';

// `npm run screenshots`: the landing page's product shots, taken from a real map on the
// same stack the e2e suite runs against. The shots are not tests and never run with the
// suite; they only borrow its servers and its database.
export default defineConfig({
	testDir: 'e2e/screenshots',
	testMatch: /\.shots\.ts$/,
	workers: 1,
	retries: 0,
	timeout: 300_000,
	expect: { timeout: 15_000 },
	use: {
		baseURL: 'http://localhost:5173',
		deviceScaleFactor: 2,
		reducedMotion: 'reduce',
	},
	webServer: e2e.webServer,
});
