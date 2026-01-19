import type { TLifetimeStatus, TMassStatus, TRoutePreference } from '@/types/models';
import type { TStaticConnections, TStaticSolarsystem } from '@/types/static-data';

export type ConnectionType = 'stargate' | 'wormhole' | 'evescout';

export type RoutingSettings = {
    routePreference: TRoutePreference;
    securityPenalty: number;
    allowEol: boolean;
    massStatus: TMassStatus;
    useEveScout: boolean;
};

export type WorkerConnection = {
    from: number;
    to: number;
    type: ConnectionType;
    massStatus?: TMassStatus;
    lifetimeStatus?: TLifetimeStatus;
};

export type WorkerRouteStep = {
    id: number;
    via: ConnectionType | null;
};

export type WorkerRouteResult = {
    id: string;
    type: 'route';
    route: WorkerRouteStep[];
    jumps: number;
    cost: number;
};

export type WorkerClosestSystem = {
    solarsystem_id: number;
    jumps: number;
    cost: number;
};

export type WorkerClosestResult = {
    id: string;
    type: 'closest';
    results: WorkerClosestSystem[];
};

export type WorkerRequest =
    | { id: string; type: 'route'; fromId: number; toId: number }
    | { id: string; type: 'closest'; fromId: number; condition: string; limit: number };

export type WorkerComputePayload = {
    callId: string;
    settings: RoutingSettings;
    dynamicConnections: WorkerConnection[];
    eveScoutConnections: WorkerConnection[];
    ignoredSystems: number[];
    requests: WorkerRequest[];
};

export type WorkerInitPayload = {
    solarsystems: TStaticSolarsystem[];
    connections: TStaticConnections;
};

export type WorkerMessage = { type: 'init'; payload: WorkerInitPayload } | { type: 'compute'; payload: WorkerComputePayload };

export type WorkerLogPayload = {
    level: 'info' | 'warn' | 'error';
    message: string;
    data?: Record<string, unknown>;
};

export type WorkerLogMessage = {
    type: 'log';
    payload: WorkerLogPayload;
};

export type WorkerResponseMessage =
    | {
          type: 'responses';
          payload: {
              callId: string;
              responses: (WorkerRouteResult | WorkerClosestResult)[];
          };
      }
    | WorkerLogMessage;
