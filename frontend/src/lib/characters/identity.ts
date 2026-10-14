// Changing which characters the account has or acts as, from the nav or the settings page.

import { createMutation, useQueryClient } from '@tanstack/svelte-query';
import { toast } from 'svelte-sonner';

import { api, errorMessage } from '$lib/api/client';
import { key } from '$lib/api/queries';

interface IdentityChange {
	work: () => Promise<unknown>;
	/** Whether the session now acts as somebody else. */
	newIdentity: boolean;
}

/**
 * Component init only (it creates a mutation).
 *
 * A change of who the session acts as reloads the page, deliberately: every cached read is
 * scoped to the signed-in identity, and the open page (sockets included) was built for the
 * old one. Anything else only refetches the account.
 */
export function createIdentityActions() {
	const client = useQueryClient();
	const mutation = createMutation(() => ({
		mutationFn: ({ work }: IdentityChange) => work(),
		onSuccess: async (_data: unknown, { newIdentity }: IdentityChange) => {
			if (newIdentity) location.reload();
			else await client.invalidateQueries({ queryKey: key.me });
		},
		onError: (err: unknown) => toast.error(errorMessage(err)),
	}));

	return {
		switchTo(characterId: number) {
			mutation.mutate({ work: () => api.switchCharacter(characterId), newIdentity: true });
		},
		/** The server switches the session away first when the active one goes. */
		remove(characterId: number, active: boolean) {
			mutation.mutate({ work: () => api.removeCharacter(characterId), newIdentity: active });
		},
	};
}
