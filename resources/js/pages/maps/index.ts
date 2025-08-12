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

export type TShowMapProps = {
    map: TMap;
    search: string;
    solarsystems: TSolarsystem[];
    config: TMapConfig;
    selected_map_solarsystem: TMapSolarSystem | null;
    map_killmails?: TKillmail[];
    map_characters: TCharacter[];
    map_route_solarsystems?: TMapRouteSolarsystem[];
    ship_history: TShipHistory[];
    has_write_access: boolean;
    layout: TLayout;
    map_user_settings: TMapUserSetting;
    shortest_path?: TShortestPath | null;
    ignored_systems: number[];
};
