import { TMapConfig } from '@/types/map';
import { TMap, TMapConnection, TMapSolarSystem } from '@/types/models';

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

export type TDataMapSolarSystem = WithIsSelected<WithHovered<TMapSolarSystem>>;

export type TProcessedConnection = TMapConnection & {
    source: TMapSolarSystem;
    target: TMapSolarSystem;
    is_on_route?: boolean;
};
