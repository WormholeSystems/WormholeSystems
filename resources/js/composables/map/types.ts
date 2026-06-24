import { TMap, TMapConnection, TMapSolarsystem } from '@/pages/maps';
import { TMapConfig } from '@/types/map';

export type Coordinates = {
    x: number;
    y: number;
};

export type TMapState = {
    map: TMap | null;
    map_container: HTMLElement | null;
    map_solarsystems: TDataMapSolarSystem[];
    map_connections: TProcessedConnection[];
    selection: {
        start: Coordinates;
        end: Coordinates | null;
    } | null;
    selected_ids: number[];
    config: TMapConfig;
    hovered_solarsystem_id: number | null;
    scale: number;
};

// Selection and hover are derived from interaction state (see isSystemSelected /
// isSystemHovered in state.ts) rather than baked onto each system, so this is just
// the resolved system with its scaled position.
export type TDataMapSolarSystem = TMapSolarsystem;

export type TProcessedConnection = TMapConnection & {
    source: TMapSolarsystem;
    target: TMapSolarsystem;
    is_on_route?: boolean;
    is_on_rally_route?: boolean;
    rally_route_reversed?: boolean;
};
