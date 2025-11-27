import { TSolarsystemType, TWormholeClass } from '@/types/models';

export type TUniverseSolarsystem = {
    id: number;
    name: string;
    region_id: number;
    constellation_id: number;
    security: number;
    type: TSolarsystemType;
    class: TWormholeClass | null;
    position: {
        x: number;
        y: number;
    };
    region: {
        id: number;
        name: string;
    };
    constellation: {
        id: number;
        name: string;
    };
    sovereignty: TUniverseSovereignty | null;
    has_stations: boolean;
    has_belts: boolean;
};

export type TUniverseSovereignty = {
    alliance: {
        id: number;
        name: string;
        ticker: string;
    } | null;
    corporation: {
        id: number;
        name: string;
        ticker: string;
    } | null;
    faction: {
        id: number;
        name: string;
    } | null;
};

export type TUniverseRegion = {
    id: number;
    name: string;
    slug: string;
};

export type TUniverseBounds = {
    minX: number;
    maxX: number;
    minY: number;
    maxY: number;
};

export type TUniverseConnection = {
    from: number;
    to: number;
    regional: boolean;
};

export type TUniverseMapProps = {
    solarsystems: TUniverseSolarsystem[];
    connections: TUniverseConnection[];
    regions: TUniverseRegion[];
    bounds: TUniverseBounds;
};
