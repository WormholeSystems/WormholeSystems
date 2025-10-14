import { TLayout } from '@/composables/useLayout';
import { TMapConfig } from '@/types/map';
import { TCharacter, TKillmail, TMap, TMapRouteSolarsystem, TMapSolarSystem, TMapUserSetting, TShipHistory, TSolarsystem } from '@/types/models';

export type TShortestPath = {
    from_solarsystem_id: number;
    to_solarsystem_id: number;
    from_solarsystem: TSolarsystem;
    to_solarsystem: TSolarsystem;
    route: TSolarsystem[];
    jumps: number;
};

export type TShortestPathDialogProps = {
    map: TMap;
    solarsystems: TSolarsystem[];
    shortest_path?: TShortestPath | null;
    ignored_systems: number[];
};

export type TClosestSystem = {
    solarsystem: TSolarsystem;
    jumps: number;
    cost: number;
};

export type TClosestSystems = {
    results: TClosestSystem[] | null;
    from_system: TSolarsystem | null;
    condition: string;
    limit: number;
};

export type TShowMapProps = {
    map: TMap;
    search: string;
    solarsystems: TSolarsystem[];
    config: TMapConfig;
    selected_map_solarsystem: TMapSolarSystem | null;
    map_killmails?: TKillmail[];
    map_characters: TCharacter[] | null;
    map_route_solarsystems?: TMapRouteSolarsystem[];
    ship_history: TShipHistory[] | null;
    has_write_access: boolean;
    has_guest_access: boolean;
    layout: TLayout;
    map_user_settings: TMapUserSetting;
    shortest_path?: TShortestPath | null;
    ignored_systems: number[];
    closest_systems?: TClosestSystems | null;
    tracking_origin?: TMapSolarSystem | null;
};
