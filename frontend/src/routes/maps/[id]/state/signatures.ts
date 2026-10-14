// The signatures domain: the scan results attached to each system, and the paste flow
// that keeps them current.

import { api } from '$lib/api/client';
import type { PastedSignature } from '$lib/api/types/PastedSignature';
import type { Signature } from '$lib/api/types/Signature';
import type { UpdateSignature } from '$lib/api/types/UpdateSignature';
import type { RunAction } from '$lib/map/actions';

/**
 * The fields a row may change. Taken from the wire type rather than restated, so a field
 * that changes shape on the server stops compiling here.
 */
export type SignaturePatch = Omit<UpdateSignature, 'map_id' | 'signature_pk'>;

export interface SignaturesHost {
	mapId: number;
	all(): Signature[];
	run: RunAction;
}

export class SignaturesApi {
	constructor(private host: SignaturesHost) {}

	get all(): Signature[] {
		return this.host.all();
	}

	/** Merge a scan-window paste into one system's signatures. */
	paste(solarSystemId: number, signatures: PastedSignature[]) {
		this.host.run('pasteSignatures', () =>
			api.pasteSignatures({ map_id: this.host.mapId, solar_system_id: solarSystemId, signatures }),
		);
	}

	/** The inline new row: a bare id in the unknown group until a paste fills it in. */
	add(solarSystemId: number, signatureId: string) {
		this.host.run('addSignature', () =>
			api.addSignature({
				map_id: this.host.mapId,
				solar_system_id: solarSystemId,
				signature_id: signatureId,
				group: 'unknown',
			}),
		);
	}

	update(signaturePk: number, patch: SignaturePatch) {
		this.host.run('updateSignature', () =>
			api.updateSignature({ map_id: this.host.mapId, signature_pk: signaturePk, ...patch }),
		);
	}

	remove(signaturePk: number) {
		this.host.run('removeSignature', () =>
			api.removeSignature({ map_id: this.host.mapId, signature_pk: signaturePk }),
		);
	}

	/** Sweep the rows a paste no longer sees. */
	removeMissing(signaturePks: number[]) {
		this.host.run('removeMissingSignatures', () =>
			api.removeSignaturesBulk({ map_id: this.host.mapId, signature_pks: signaturePks }),
		);
	}

	link(signaturePk: number, connectionId: number) {
		this.host.run('linkSignature', () =>
			api.linkSignature({
				map_id: this.host.mapId,
				signature_pk: signaturePk,
				connection_id: connectionId,
			}),
		);
	}

	unlink(signaturePk: number) {
		this.host.run('unlinkSignature', () =>
			api.unlinkSignature({ map_id: this.host.mapId, signature_pk: signaturePk }),
		);
	}
}
