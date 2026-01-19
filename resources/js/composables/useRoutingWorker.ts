import { useStaticData } from '@/composables/useStaticData';
import type { WorkerClosestResult, WorkerComputePayload, WorkerInitPayload, WorkerResponseMessage, WorkerRouteResult } from '@/routing/types';
import RoutingWorker from '@/workers/routingWorker?worker&inline';

let workerClient: RoutingWorkerClient | null = null;
let initPromise: Promise<void> | null = null;

class RoutingWorkerClient {
    private workerPromise: Promise<Worker>;
    private worker: Worker | null = null;
    private callCounter = 0;
    private pending = new Map<string, (value: (WorkerRouteResult | WorkerClosestResult)[]) => void>();

    constructor() {
        this.workerPromise = this.loadWorker();
    }

    private async loadWorker(): Promise<Worker> {
        const worker = new RoutingWorker();

        worker.addEventListener('message', (event: MessageEvent<WorkerResponseMessage>) => {
            if (event.data?.type === 'log') {
                const { level, message, data } = event.data.payload;
                const logPayload = data ? [message, data] : [message];

                if (level === 'warn') {
                    console.warn(...logPayload);
                    return;
                }

                if (level === 'error') {
                    console.error(...logPayload);
                    return;
                }

                console.info(...logPayload);
                return;
            }

            if (event.data?.type !== 'responses') {
                return;
            }

            const resolver = this.pending.get(event.data.payload.callId);
            if (!resolver) {
                return;
            }

            resolver(event.data.payload.responses);
            this.pending.delete(event.data.payload.callId);
        });

        worker.addEventListener('error', (event: Event) => {
            console.error('[routing-worker] error', event);
        });

        worker.addEventListener('messageerror', (event: Event) => {
            console.error('[routing-worker] message error', event);
        });

        this.worker = worker;

        return worker;
    }

    private async getWorker(): Promise<Worker> {
        if (this.worker) {
            return this.worker;
        }

        return this.workerPromise;
    }

    public async initialize(payload: WorkerInitPayload): Promise<void> {
        const worker = await this.getWorker();
        worker.postMessage({ type: 'init', payload });
    }

    public async compute(payload: Omit<WorkerComputePayload, 'callId'>): Promise<(WorkerRouteResult | WorkerClosestResult)[]> {
        const worker = await this.getWorker();
        const callId = `call-${this.callCounter++}`;
        const messagePayload = {
            ...payload,
            callId,
        };

        console.info('[routing-worker] post compute', {
            callId,
            requests: payload.requests.length,
        });

        return new Promise((resolve) => {
            this.pending.set(callId, resolve);
            worker.postMessage({
                type: 'compute',
                payload: messagePayload,
            });
        });
    }
}

export async function initializeRoutingWorker(): Promise<void> {
    if (!workerClient) {
        workerClient = new RoutingWorkerClient();
    }

    if (!initPromise) {
        initPromise = (async () => {
            const { loadStaticData } = useStaticData();
            const data = await loadStaticData();

            console.info('[routing-worker] init payload', {
                solarsystems: data.solarsystems.length,
                connections: Object.keys(data.connections).length,
            });

            await workerClient!.initialize({ solarsystems: data.solarsystems, connections: data.connections });
        })();
    }

    await initPromise;
}

export async function getRoutingWorkerClient(): Promise<RoutingWorkerClient> {
    await initializeRoutingWorker();

    if (!workerClient) {
        throw new Error('Routing worker is not initialized');
    }

    return workerClient;
}
