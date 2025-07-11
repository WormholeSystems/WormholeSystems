export type TMap = {
    id: number;
    name: string;
    slug: string;
    map_solarsystems?: TMapSolarSystem[];
    map_connections?: TMapConnection[];
    map_solarsystems_count?: number;
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
    status: TMapSolarsystemStatus | null;
    solarsystem: TSolarsystem | null;
    statics: TWormhole[] | null;
    pinned: boolean;
    signatures: TSignature[] | null;
    signatures_count?: number;
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

export type TKillmail = {
    id: number;
    hash: string;
    solarsystem: TSolarsystem;
    ship_type: TType;
    data: TRawKillmail;
    zkb: TzKillboard;
    time: string;
};

export type TRawKillmail = {
    victim: {
        character_id: number | null;
        corporation_id: number | null;
        alliance_id: number | null;
        faction_id: number | null;
        ship_type_id: number;
        items: TRawKillmailItem[];
    };
    attackers: TRawKillmailAttacker[];
};

export type TRawKillmailItem = {
    type_id: number;
    flag: number;
    quantity_destroyed: number;
    quantity_dropped: number;
};

export type TRawKillmailAttacker = {
    character_id: number | null;
    corporation_id: number | null;
    alliance_id: number | null;
    faction_id: number | null;
    ship_type_id: number;
    damage_done: number;
    final_blow: boolean;
    weapons_type_id: number | null;
};

export type TType = {
    id: number;
    name: string;
    group_id: number;
    market_group_id: number | null;
    description: string | null;
    icon_id: number | null;
    graphic_id: number | null;
};

export type TzKillboard = {
    awox: boolean;
    destroyedValue: number;
    droppedValue: number;
    fittedValue: number;
    hash: string;
    labels: string[];
    locationID: number;
    npc: boolean;
    points: number;
    solo: boolean;
    totalValue: number;
};

export type TSignature = {
    id: string;
    signature_id: string;
    type: string;
    category: string | null;
    name: string | null;
    created_at: string;
    updated_at: string;
};

export type TMapSolarsystemStatus = 'active' | 'unscanned' | 'unknown' | 'friendly' | 'hostile' | 'empty';

export type TCharacter = {
    id: number;
    name: string;
    corporation: TCorporation | null;
    alliance: TAlliance | null;
    faction: TFaction | null;
    status: TCharacterStatus | null;
};

export type TCharacterStatus = {
    solarsystem_id: number;
    ship_type: TType | null;
    ship_name: string | null;
    is_online: boolean;
    station_id: number | null;
    structure_id: number | null;
    last_online_at: string | null;
    checked_last_online_at: string | null;
    checked_location_at: string | null;
    solarsystem: TSolarsystem | null;
};
