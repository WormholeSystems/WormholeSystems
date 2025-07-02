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
    effects: Record<string, string> | null;
    map_id: number;
    position: {
        x: number;
        y: number;
    } | null;
    status: string | null;
    solarsystem: TSolarsystem | null;
};

export type TMapConnection = {
    id: number;
    map_id: number;
    from_map_solarsystem_id: number;
    to_map_solarsystem_id: number;
    wormhole: TWormhole | null;
};

export type TWormhole = {
    id: number;
    name: string;
    total_mass: number;
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
