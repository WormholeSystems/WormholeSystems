import { E2E_CHARACTER_ID, E2E_SESSION, ensureIdentity } from './db';

// Create (or refresh) the main e2e identity every spec signs in as by default.
export default async function globalSetup() {
	await ensureIdentity({
		characterId: E2E_CHARACTER_ID,
		session: E2E_SESSION,
		name: 'E2E Pilot',
		ownerHash: 'e2e-owner-hash',
	});
}
