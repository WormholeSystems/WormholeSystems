// TSignatureCategory now defined in this file

import type { TMapConnection, TResolvedSolarsystem, TSolarsystem, TWormhole } from '@/pages/maps';

export type TSolarsystemType = 'eve' | 'wormhole' | 'abyssal';

export type TRegion = {
    id: number;
    name: string;
};

export type TWormholeEffectName = 'Pulsar' | 'Cataclysmic Variable' | 'Magnetar' | 'Red Giant' | 'Wolf-Rayet Star' | 'Black Hole';

export type TWormholeEffect = {
    id: number;
    name: string;
    type: 'Buff' | 'Debuff';
    strength: string;
};

export type TMassStatus = 'fresh' | 'reduced' | 'critical';

export type TRoutePreference = 'shorter' | 'safer' | 'less_secure';

export type TLifetimeStatus = 'healthy' | 'eol' | 'critical';

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
    solarsystem_id: number;
    ship_type: TType | null;
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

export type TSignatureCategory = {
    id: number;
    name: string;
    code: string;
    created_at: string;
    updated_at: string;
};

export type TWormholeClass = 1 | 2 | 3 | 4 | 5 | 6 | 7 | 8 | 9 | 10 | 11 | 12 | 13 | 14 | 15 | 16 | 17 | 18;
export type TKSpaceClass = 'n' | 'l' | 'h' | 'p';
export type TSolarsystemClass = TWormholeClass | TKSpaceClass | 'unknown';
export type TStringedSolarsystemClass = `${TSolarsystemClass}`;

export type TSignatureType = {
    id: number;
    name: string;
    signature: string;
    signature_category_id: number;
    category_name: string;
    target_class: TSolarsystemClass | null;
    extra: string | null;
    spawn_areas: TStringedSolarsystemClass[] | null;
    created_at?: string;
    updated_at?: string;
};

export type TSignature = {
    id: number;
    map_solarsystem_id: number;
    map_connection_id: number | null;
    signature_id: string | null;
    signature_type_id: number | null;
    signature_category_id: number | null;
    raw_type_name: string | null;
    signature_type: TSignatureType | null;
    signature_category: TSignatureCategory | null;
    mass_status: TMassStatus | null;
    ship_size: TShipSize | null;
    lifetime: TLifetimeStatus;
    lifetime_updated_at: string | null;
    created_at: string;
    updated_at: string;
    wormhole: TWormhole | null;
    map_connection: TMapConnection | null;
};

export type TMapSolarsystemStatus = 'active' | 'unscanned' | 'unknown' | 'friendly' | 'hostile' | 'empty';

export type TCharacter = {
    id: number;
    name: string;
    corporation: TCorporation | null;
    alliance: TAlliance | null;
    faction: TFaction | null;
    status: TCharacterStatus | null;
    route?: TSolarsystem[]; // Fastest route
    esi_scopes?: string[];
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
};

export type TMapRouteSolarsystem = {
    id: number;
    map_id: number;
    solarsystem_id: number;
    solarsystem?: TResolvedSolarsystem | null;
    is_pinned: boolean;
    route?: TResolvedSolarsystem[];
};

export type TMapUserSetting = {
    id: number;
    user_id: number;
    map_id: number;
    tracking_allowed: boolean;
    is_tracking: boolean;
    has_write_access: boolean;
    route_allow_eol: boolean;
    route_allow_mass_status: TMassStatus;
    route_use_evescout: boolean;
    route_preference: TRoutePreference;
    security_penalty: number;
    killmail_filter: 'all' | 'jspace' | 'kspace';
    introduction_confirmed_at: string | null;
    prompt_for_signature_enabled: boolean;
    layout_breakpoints?: Record<string, any> | null;
};

export type TShipHistory = {
    id: number;
    character_id: number;
    ship_type_id: number;
    name: string;
    ship_id: number;
    ship_type: TType | null;
    character: TCharacter | null;
    created_at: string;
    updated_at: string;
};

export type TServerStatus = {
    id: number;
    server_version: string;
    start_time: string;
    players: number;
    vip: boolean;
    created_at: string;
    updated_at: string;
};

export type TToken = {
    id: number;
    name: string;
    created_at: string;
    last_used_at: string;
};

export type TAudit = {
    id: number;
    character_id: number | null;
    character: TCharacter | null;
    event: 'created' | 'updated' | 'deleted';
    new_values: Record<string, any>;
    old_values: Record<string, any>;
    tags: string[];
    created_at: string;
};
