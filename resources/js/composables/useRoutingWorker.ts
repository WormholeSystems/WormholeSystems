import { useStaticData } from '@/composables/useStaticData';
import type { WorkerClosestResult, WorkerComputePayload, WorkerInitPayload, WorkerResponseMessage, WorkerRouteResult } from '@/routing/types';

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
        const workerUrl = new URL('../workers/routingWorker.ts', import.meta.url);
        const response = await fetch(workerUrl.toString(), { credentials: 'include' });
        const script = await response.text();
        const blob = new Blob([script], { type: 'application/javascript' });
        const blobUrl = URL.createObjectURL(blob);
        const worker = new Worker(blobUrl, { type: 'module' });
        URL.revokeObjectURL(blobUrl);

        worker.addEventListener('message', (event: MessageEvent<WorkerResponseMessage>) => {
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
        return new Promise((resolve) => {
            this.pending.set(callId, resolve);
            worker.postMessage({
                type: 'compute',
                payload: {
                    ...payload,
                    callId,
                },
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
