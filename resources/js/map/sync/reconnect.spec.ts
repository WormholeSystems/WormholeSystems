import { describe, expect, it, vi } from 'vitest';
import { createReconnectListener } from './reconnect';

describe('createReconnectListener', () => {
    it('does not fire on the initial connection', () => {
        const onReconnect = vi.fn();
        const listener = createReconnectListener(onReconnect);

        listener({ previous: 'initialized', current: 'connecting' });
        listener({ previous: 'connecting', current: 'connected' });

        expect(onReconnect).not.toHaveBeenCalled();
    });

    it('fires on a reconnect after a dropped connection', () => {
        const onReconnect = vi.fn();
        const listener = createReconnectListener(onReconnect);

        listener({ previous: 'connecting', current: 'connected' });
        listener({ previous: 'connected', current: 'unavailable' });
        listener({ previous: 'unavailable', current: 'connecting' });
        listener({ previous: 'connecting', current: 'connected' });

        expect(onReconnect).toHaveBeenCalledTimes(1);
    });

    it('fires on every subsequent reconnect', () => {
        const onReconnect = vi.fn();
        const listener = createReconnectListener(onReconnect);

        listener({ previous: 'connecting', current: 'connected' });
        listener({ previous: 'connected', current: 'connecting' });
        listener({ previous: 'connecting', current: 'connected' });
        listener({ previous: 'connected', current: 'unavailable' });
        listener({ previous: 'unavailable', current: 'connected' });

        expect(onReconnect).toHaveBeenCalledTimes(2);
    });

    it('ignores state changes that do not reach connected', () => {
        const onReconnect = vi.fn();
        const listener = createReconnectListener(onReconnect);

        listener({ previous: 'connecting', current: 'connected' });
        listener({ previous: 'connected', current: 'unavailable' });
        listener({ previous: 'unavailable', current: 'connecting' });
        listener({ previous: 'connecting', current: 'failed' });

        expect(onReconnect).not.toHaveBeenCalled();
    });
});
