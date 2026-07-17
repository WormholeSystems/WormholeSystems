export type ConnectionStateChange = {
    previous: string;
    current: string;
};

/**
 * Wraps a callback so it only fires on genuine reconnects, never on the
 * initial connection. Pusher reaches 'connected' from 'connecting' on the
 * first connect too, so the previous state cannot distinguish the two — a
 * naive previous-state check made every page load trigger a full map reload.
 */
export function createReconnectListener(onReconnect: () => void): (states: ConnectionStateChange) => void {
    let hasConnectedBefore = false;

    return ({ current }) => {
        if (current !== 'connected') {
            return;
        }
        if (hasConnectedBefore) {
            onReconnect();
        }
        hasConnectedBefore = true;
    };
}
