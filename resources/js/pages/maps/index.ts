import { TMapConfig } from '@/types/map';
import { TCharacter, TKillmail, TMap, TMapRouteSolarsystem, TMapSolarSystem, TShipHistory, TSolarsystem } from '@/types/models';

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
};
