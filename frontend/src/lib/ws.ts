import type { MapEvent } from '$lib/api/types/MapEvent';
import type { UserEvent } from '$lib/api/types/UserEvent';

// Realtime sockets. Frames carry no payload we act on: each one only means "something
// changed, refetch". The map socket reconnects with backoff and reports its state, because
// a silently dead socket looks exactly like a quiet map.

function socketUrl(path: string): string {
	const scheme = location.protocol === 'https:' ? 'wss' : 'ws';
	return `${scheme}://${location.host}${path}`;
}

export type SocketState = 'connecting' | 'open' | 'reconnecting' | 'revoked';

/** The server closes a map socket with this once the reader has lost access to the map. */
export const ACCESS_REVOKED = 4403;

/**
 * One place for the wire cast. The frames are produced by this repo's own backend and
 * typed by ts-rs, so a schema would only re-state the generated types and drift on every
 * change; the guard just refuses frames that are not event-shaped at all.
 */
export function parseFrame(raw: string): unknown | null {
	try {
		const parsed: unknown = JSON.parse(raw);
		if (typeof parsed === 'object' && parsed !== null && 'type' in parsed) return parsed;
		return null;
	} catch {
		return null;
	}
}

const FIRST_RETRY_MS = 500;
const MAX_RETRY_MS = 15_000;

/**
 * Open the per-map event stream. `onEvent` fires per frame and once per successful reconnect,
 * with `null` for the reconnect catch-up, meaning "you missed something, reload everything".
 * Returns a close function that also stops the retry loop.
 */
export function openMapSocket(
	mapId: number,
	onEvent: (event: MapEvent | null) => void,
	onState?: (state: SocketState) => void,
): () => void {
	let ws: WebSocket | null = null;
	let retryMs = FIRST_RETRY_MS;
	let timer: ReturnType<typeof setTimeout> | null = null;
	let closed = false;
	// Only a reconnect needs the catch-up refetch; the first connect has the initial load.
	let reconnecting = false;

	function connect() {
		if (closed) return;
		onState?.(reconnecting ? 'reconnecting' : 'connecting');
		ws = new WebSocket(socketUrl(`/ws/map/${mapId}`));

		ws.onopen = () => {
			if (closed) return;
			retryMs = FIRST_RETRY_MS;
			onState?.('open');
			if (reconnecting) {
				reconnecting = false;
				onEvent(null);
			}
		};
		ws.onmessage = (frame) => {
			const parsed = parseFrame(frame.data as string);
			// Unreadable frame: fall back to a full refetch rather than ignoring it.
			onEvent(parsed === null ? null : (parsed as MapEvent));
		};
		ws.onclose = (event) => {
			if (closed) return;
			if (event.code === ACCESS_REVOKED) {
				// Retrying would only be refused again. One refetch lets the map's own 403
				// handling take over.
				closed = true;
				onState?.('revoked');
				onEvent(null);
				return;
			}
			reconnecting = true;
			onState?.('reconnecting');
			timer = setTimeout(connect, retryMs);
			retryMs = Math.min(retryMs * 2, MAX_RETRY_MS);
		};
		// An error is always followed by a close, which owns the retry.
		ws.onerror = () => ws?.close();
	}

	connect();

	return () => {
		closed = true;
		if (timer) clearTimeout(timer);
		ws?.close();
	};
}

/**
 * Open the per-user channel (activity heartbeat + status pings). Returns a close function.
 * Unlike the map socket the payload matters, since the channel mixes events addressed to this
 * user with news that concerns everyone.
 */
export function openUserSocket(onEvent: (event: UserEvent) => void): () => void {
	const ws = new WebSocket(socketUrl('/ws/user'));
	ws.onmessage = (frame) => {
		// A frame we cannot read is not worth acting on.
		const parsed = parseFrame(frame.data as string);
		if (parsed !== null) onEvent(parsed as UserEvent);
	};
	return () => ws.close();
}
