// Classifying rows after a paste: what the scan added, refreshed, or no longer sees.
// The pre-paste ids are snapshotted by the caller, so new (green) against updated
// (amber) stays stable after the round-trip creates the new rows.
//
// Only a row that was there when the scan was pasted can be missing from it. One that
// turned up since, a teammate's paste or a row added by hand, is news the scan never had a
// chance to see, so it is left untinted and out of "Delete missing".

import type { Signature } from '$lib/api/types/Signature';

export type PasteStatus = 'new' | 'updated' | 'deleted';

/** What one row's tint says, or null when no paste is being reviewed. */
export function pasteStatus(
	signature: Signature,
	preIds: Set<string>,
	pastedIds: Set<string> | null,
): PasteStatus | null {
	if (pastedIds === null) return null;
	if (!pastedIds.has(signature.signature_id)) {
		return preIds.has(signature.signature_id) ? 'deleted' : null;
	}
	return preIds.has(signature.signature_id) ? 'updated' : 'new';
}

/** The rows the paste no longer sees, offered for a lazy sweep. */
export function deletedByPaste(
	signatures: Signature[],
	preIds: Set<string>,
	pastedIds: Set<string> | null,
): Signature[] {
	if (pastedIds === null) return [];
	return signatures.filter((s) => pasteStatus(s, preIds, pastedIds) === 'deleted');
}
