// TSignatureCategory now defined in this file

import type { TMapConnection, TResolvedSolarsystem, TWormhole } from '@/pages/maps';

export type TSolarsystemType = 'eve' | 'wh' | 'hidden' | 'abyssal' | 'void';

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

export type TShipSize = 'frigate' | 'medium' | 'large' | 'xlarge';

export type TConnectionType = 'wormhole' | 'stargate';

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

export type TKillmailAffiliation = {
    id: number;
    name: string;
    ticker: string;
};

export type TKillmail = {
    id: number;
    hash: string;
    solarsystem_id: number;
    ship_type: TType | null;
    data: TRawKillmail;
    zkb: TzKillboard;
    victim_corporation: TKillmailAffiliation | null;
    victim_alliance: TKillmailAffiliation | null;
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

export type TWormholeClass = 1 | 2 | 3 | 4 | 5 | 6 | 12 | 13 | 14 | 15 | 16 | 17 | 18 | 19 | 20 | 21 | 22 | 23;
export type TKSpaceClass = 'n' | 'l' | 'h' | 'p';
export type TSolarsystemClass = TWormholeClass | TKSpaceClass | 'unknown';

/**
 * The canonical wire/storage form of a solarsystem class: the string value of
 * the PHP `SolarsystemClass` enum (e.g. "1".."18", "h"/"l"/"n", "unknown").
 * Every system resolves to exactly one of these server-side.
 */
export type TStringedSolarsystemClass = `${TSolarsystemClass}`;

/**
 * Metadata for a single solarsystem class. Generated from the PHP enum via
 * `php artisan generate:solarsystem-classes`; see const/solarsystemClasses.ts.
 */
export type TSolarsystemClassMeta = {
    value: TStringedSolarsystemClass;
    label: string;
    short_label: string;
    color_token: string;
    sort_weight: number;
    is_standard: boolean;
    is_special: boolean;
    is_drifter: boolean;
    is_known_space: boolean;
    is_wormhole_space: boolean;
};

export type TSignatureType = {
    id: number;
    name: string;
    signature: string;
    signature_category_id: number;
    category_name: string;
    target_class: TStringedSolarsystemClass | null;
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

export type TThreatLevel = 'critical' | 'high' | 'unknown';

export type TThreatEntity = {
    id: number;
    name: string;
    type: 'alliance' | 'corporation' | 'unknown';
    kills: number;
};

export type TThreatAnalysis = {
    solarsystem_id: number;
    threat_level: TThreatLevel;
    threat_data: TThreatEntity[];
    threat_analyzed_at: string | null;
};

export type TThreatSearchSystem = {
    solarsystem_id: number;
    kills: number;
    occupier_alias: string | null;
};

export type TThreatSearchResult = {
    id: number;
    name: string;
    type: TThreatEntity['type'];
    total_kills: number;
    systems_count: number;
    systems: TThreatSearchSystem[];
};

export type TNoteSearchResult = {
    map_solarsystem_id: number;
    solarsystem_id: number;
    alias: string | null;
    occupier_alias: string | null;
    note_excerpt: string;
};

export type TOccupierSearchResult = {
    solarsystem_id: number;
    occupier_alias: string | null;
};

export type TCharacter = {
    id: number;
    name: string;
    corporation: TCorporation | null;
    alliance: TAlliance | null;
    faction: TFaction | null;
    status: TCharacterStatus | null;
    route?: TResolvedSolarsystem[]; // Fastest route
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
    id: number | null;
    user_id: number | null;
    map_id: number;
    tracking_allowed: boolean;
    is_tracking: boolean;
    route_allow_lifetime_status: TLifetimeStatus;
    route_allow_mass_status: TMassStatus;
    route_use_evescout: boolean;
    route_preference: TRoutePreference;
    security_penalty: number;
    killmail_filter: 'all' | 'jspace' | 'kspace';
    introduction_confirmed_at: string | null;
    prompt_for_signature_enabled: boolean;
    preselect_signature_enabled: boolean;
    suggest_alias_enabled: boolean;
    copy_bookmark_enabled: boolean;
    layout_breakpoints?: Record<string, any> | null;
    hidden_cards: string[] | null;
    show_threat_level: boolean;
    show_statics_first: boolean;
    compact_signature_list: boolean;
    is_archived: boolean;
    is_pinned: boolean;
    background_image_url: string | null;
    background_image_mode: 'grid' | 'viewport';
    layout_override: 'manual' | 'tree' | null;
};

export type TMapWebhookType = 'proximity' | 'killmail' | 'jump_range';

export type TJumpShipType = 'dreadnought' | 'carrier' | 'force_auxiliary' | 'supercarrier' | 'titan' | 'jump_freighter' | 'rorqual' | 'black_ops';

export type TKillmailFilterSubject = 'ship_type' | 'ship_group' | 'character' | 'corporation' | 'alliance';

export type TKillmailFilterSide = 'victim' | 'attacker' | 'either';

export type TKillmailFilterMode = 'include' | 'exclude';

export type TKillmailFilterMatch = 'any' | 'all';

export type TKillmailFilterRule = {
    subject: TKillmailFilterSubject;
    side: TKillmailFilterSide;
    mode: TKillmailFilterMode;
    ids: number[];
};

export type TEveSearchResult = {
    id: number;
    name: string;
    group_name?: string | null;
    category_name?: string | null;
    mass?: number;
};

export type TMapWebhook = {
    id: number;
    name: string;
    alerts_count?: number;
};

export type TMapWebhookRole = {
    id: number;
    name: string;
    discord_role_id: string;
    alerts_count?: number;
};

export type TMapAlert = {
    id: number;
    map_webhook_id: number;
    map_webhook_role_id: number | null;
    type: TMapWebhookType;
    target_solarsystem_id: number | null;
    ship_type: TJumpShipType | null;
    jdc_level: number | null;
    include_highsec: boolean;
    max_jumps: number | null;
    filter_match: TKillmailFilterMatch;
    filters: TKillmailFilterRule[];
    is_active: boolean;
    last_fired_at: string | null;
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
