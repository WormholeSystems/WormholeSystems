import { afterEach, describe, expect, it, vi } from 'vitest';

import { ACCESS_REVOKED, openMapSocket, parseFrame, type SocketState } from './ws';

describe('parseFrame', () => {
	it('accepts an event-shaped frame', () => {
		expect(parseFrame('{"type":"map_updated","map_id":1}')).toEqual({
			type: 'map_updated',
			map_id: 1,
		});
	});

	it('refuses frames that are not events at all', () => {
		expect(parseFrame('not json')).toBeNull();
		expect(parseFrame('42')).toBeNull();
		expect(parseFrame('null')).toBeNull();
		expect(parseFrame('{"kind":"other"}')).toBeNull();
	});
});

class FakeSocket {
	static all: FakeSocket[] = [];
	onopen: (() => void) | null = null;
	onmessage: ((frame: { data: string }) => void) | null = null;
	onclose: ((event: { code: number }) => void) | null = null;
	onerror: (() => void) | null = null;
	constructor(public url: string) {
		FakeSocket.all.push(this);
	}
	close() {}
}

describe('openMapSocket', () => {
	afterEach(() => {
		vi.useRealTimers();
		vi.unstubAllGlobals();
		FakeSocket.all = [];
	});

	function open() {
		vi.useFakeTimers();
		vi.stubGlobal('WebSocket', FakeSocket);
		vi.stubGlobal('location', { protocol: 'http:', host: 'localhost' });
		const events: unknown[] = [];
		const states: SocketState[] = [];
		const close = openMapSocket(
			7,
			(event) => events.push(event),
			(state) => states.push(state),
		);
		return { events, states, close };
	}

	it('reconnects after an ordinary close and asks for a catch-up', () => {
		const { events, states } = open();
		FakeSocket.all[0].onclose?.({ code: 1006 });
		vi.advanceTimersByTime(1_000);
		FakeSocket.all[1].onopen?.();
		expect(FakeSocket.all).toHaveLength(2);
		expect(states).toEqual(['connecting', 'reconnecting', 'reconnecting', 'open']);
		expect(events).toEqual([null]);
	});

	it('passes events through, and asks for the full catch-up when the server lagged', () => {
		const { events } = open();
		FakeSocket.all[0].onopen?.();
		FakeSocket.all[0].onmessage?.({ data: '{"type":"map_updated","map_id":7}' });
		FakeSocket.all[0].onmessage?.({ data: '{"type":"lagged"}' });
		FakeSocket.all[0].onmessage?.({ data: 'garbage' });
		expect(events).toEqual([{ type: 'map_updated', map_id: 7 }, null, null]);
	});

	it('stops for good once access is revoked', () => {
		const { events, states } = open();
		FakeSocket.all[0].onclose?.({ code: ACCESS_REVOKED });
		vi.advanceTimersByTime(60_000);
		expect(FakeSocket.all).toHaveLength(1);
		expect(states).toEqual(['connecting', 'revoked']);
		expect(events).toEqual([null]);
	});
});
