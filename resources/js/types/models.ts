export type TMap = {
    id: number;
    name: string;
    slug: string;
    map_solarsystems: TMapSolarSystem[];
    map_connections: TMapConnection[];
};

export type TMapSolarSystem = {
    id: number;
    solarsystem_id: number;
    name: string;
    alias: string;
    occupier_alias: string;
    class: number;
    effect: TWormholeEffectName | null;
    effects: TWormholeEffect[];
    map_id: number;
    position: {
        x: number;
        y: number;
    } | null;
    status: string | null;
    solarsystem: TSolarsystem | null;
    statics: TWormhole[] | null;
    pinned: boolean;
};

export type TMapConnection = {
    id: number;
    map_id: number;
    from_map_solarsystem_id: number;
    to_map_solarsystem_id: number;
    wormhole: TWormhole | null;
    ship_size: TShipSize;
    mass_status: TMassStatus;
    is_eol: boolean;
};

export type TWormhole = {
    id: number;
    name: string;
    total_mass: number;
    maximum_jump_mass: number;
    ship_size: string;
    maximum_lifetime: number;
    leads_to: string;
    type_id: number;
    created_at: string;
    updated_at: string;
};

export type TSolarsystem = {
    id: number;
    name: string;
    type: 'eve' | 'wormhole' | 'abyssal';
    region: TRegion | null;
    constellation: TConstellation | null;
    security: number;
    class: number;
    effect: TWormholeEffectName | null;
    sovereignty: TSovereignty | null;
};

export type TRegion = {
    id: number;
    name: string;
};

export type TConstellation = {
    id: number;
    name: string;
    region_id: number;
};

export type TWormholeEffectName = 'Pulsar' | 'Cataclysmic Variable' | 'Magnetar' | 'Red Giant' | 'Wolf-Rayet Star' | 'Black Hole';

export type TWormholeEffect = {
    id: number;
    name: TWormholeEffectName;
    type: 'Buff' | 'Debuff';
    strength: string;
};

export type TMassStatus = 'fresh' | 'reduced' | 'critical';

export type TShipSize = 'frigate' | 'medium' | 'large';

export type TSovereignty = {
    id: number;
    alliance: TAlliance | null;
    corporation: TCorporation | null;
    faction: TFaction | null;
};

export type TAlliance = {
    id: number;
    name: string;
    ticker: string;
};

export type TCorporation = {
    id: number;
    name: string;
    ticker: string;
};

export type TFaction = {
    id: number;
    name: string;
};
