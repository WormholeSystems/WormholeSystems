import type { TSolarsystem, TSovereignty } from '@/pages/maps';

export type TStaticRegion = {
    id: number;
    name: string;
    type: string;
};

export type TStaticConstellation = {
    id: number;
    name: string;
    type: string;
    region_id: number;
    region: {
        id: number;
        name: string;
    };
};

export type TStaticService = {
    id: number;
    name: string;
};

export type TStaticSolarsystem = TSolarsystem & {
    sovereignty: TSovereignty | null;
    has_jove_observatory: boolean;
    has_stations: boolean;
    services: number[];
};

export type TStaticConnections = Record<number, number[]>;

export type TStaticData = {
    regions: TStaticRegion[];
    constellations: TStaticConstellation[];
    solarsystems: TStaticSolarsystem[];
    connections: TStaticConnections;
    services: TStaticService[];
};
