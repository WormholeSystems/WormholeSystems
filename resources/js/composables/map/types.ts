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
    config: TMapConfig;
    hovered_solarsystem_id: number | null;
    scale: number;
};

export type WithIsSelected<T> = T & {
    is_selected: boolean;
};

export type WithHovered<T> = T & {
    is_hovered: boolean;
};

export type TDataMapSolarSystem = WithIsSelected<WithHovered<TMapSolarsystem>>;

export type TProcessedConnection = TMapConnection & {
    source: TMapSolarsystem;
    target: TMapSolarsystem;
    is_on_route?: boolean;
};
