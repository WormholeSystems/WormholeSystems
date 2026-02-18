import { TLayout } from '@/composables/useLayout';
import { TEveScoutConnection } from '@/types/eve-scout';
import { TMapConfig } from '@/types/map';
import {
    TAudit,
    TCharacter,
    TKillmail,
    TLifetimeStatus,
    TMapRouteSolarsystem,
    TMapSolarsystemStatus,
    TMapUserSetting,
    TMassStatus,
    TShipHistory,
    TShipSize,
    TSignature,
    TSolarsystemClass,
    TSolarsystemType,
    TWormholeClass,
    TWormholeEffectName,
} from '@/types/models';
import type { TStaticSolarsystem } from '@/types/static-data';

export type TShortestPath = {
    from_solarsystem_id: number;
    to_solarsystem_id: number;
    from_solarsystem: TSolarsystem;
    to_solarsystem: TSolarsystem;
    route: TSolarsystem[];
    jumps: number;
};

export type TClosestSystem = {
    solarsystem: TSolarsystem;
    jumps: number;
    cost: number;
};

export type TClosestSystems = {
    results: TClosestSystem[] | null;
    from_system: TSolarsystem;
    condition: string;
    limit: number;
};

export type TMapNavigation = {
    destinations: TMapRouteSolarsystem[];
};

export type TShowMapProps = {
    map: TServerMap;
    config: TMapConfig;
    selected_map_solarsystem: TSelectedMapSolarsystem | null;
    map_killmails?: TKillmail[];
    map_characters: TCharacter[] | null;
    map_navigation?: TMapNavigation;
    ship_history?: TShipHistory[] | null;
    has_write_access: boolean;
    has_guest_access: boolean;
    active_character_has_access: boolean;
    layout: TLayout;
    map_user_settings: TMapUserSetting;
    ignored_systems: number[];
    tracking_origin?: TSelectedMapSolarsystem | null;
    tracking_target?: TTrackingTarget | null;
    eve_scout_connections?: TEveScoutConnection[];
};

export type TTrackingTarget = {
    solarsystem_id: number;
};

export type TMapSolarsystemBase = {
    id: number;
    map_id: number;
    solarsystem_id: number;
    alias: string | null;
    status: TMapSolarsystemStatus;
    occupier_alias: string | null;
    notes: string | null;
    position: {
        x: number;
        y: number;
    } | null;
    pinned: boolean;
    signatures_count: number;
    wormhole_signatures_count: number;
    map_connections_count: number;
    signatures?: TSignature[] | null;
};

export type TMapSolarsystem = TMapSolarsystemBase & {
    solarsystem: TResolvedSolarsystem;
};

export type TServerMapSolarsystem = TMapSolarsystemBase & {
    solarsystem?: TResolvedSolarsystem | null;
};

export type TMap = {
    id: number;
    name: string;
    slug: string;
    map_solarsystems: TMapSolarsystem[];
    map_connections: TMapConnection[];
};

export type TServerMap = Omit<TMap, 'map_solarsystems'> & {
    map_solarsystems: TServerMapSolarsystem[];
};

export type TSolarsystem = {
    id: number;
    name: string;
    region_id: number;
    constellation_id: number;
    class: TWormholeClass | null;
    security: number;
    type: TSolarsystemType;
    region: TRegion;
    constellation?: TTailoredConstellation;
    sovereignty: TSovereignty | null;
    statics: TStatic[] | null;
    effect: TEffect | null;
    connection_type?: 'stargate' | 'wormhole' | 'evescout' | null;
};

export type TResolvedSolarsystem = TStaticSolarsystem | TSolarsystem;

export type TRegion = {
    id: number;
    name: string;
};

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

export type TStatic = {
    id: number;
    leads_to: string;
    name: string;
    maximum_lifetime: number;
    maximum_jump_mass: number;
    total_mass: number;
};

export type TEffect = {
    name: TWormholeEffectName;
    effects: TWormholeEffect[];
};

export type TWormholeEffect = {
    id: number;
    name: string;
    type: 'Buff' | 'Debuff';
    strength: string;
};

export type TMapConnection = {
    id: number;
    from_map_solarsystem_id: number;
    to_map_solarsystem_id: number;
    mass_status: TMassStatus;
    lifetime_status: TLifetimeStatus;
    lifetime_status_updated_at: string | null;
    signatures: TTailoredSignature[] | null;
    ship_size: TShipSize;
    created_at: string;
    updated_at: string;
};

export type TWormhole = {
    id: number;
    name: string;
    total_mass: number;
    maximum_jump_mass: number;
    ship_size: string;
    maximum_lifetime: number;
    leads_to: string;
};

export type TTailoredSignature = {
    id: number;
    signature_id: string;
    map_solarsystem_id: number;
    target_class: TSolarsystemClass | null;
    extra: string | null;
    wormhole: TWormhole | null;
    signature_category: TSignatureCategory | null;
    signature_type: TSignatureType | null;
    raw_type_name: string;
    created_at: string;
    updated_at: string;
    lifetime_status: TLifetimeStatus | null;
    lifetime_status_updated_at: string | null;
    mass_status: TMassStatus | null;
    map_connection_id: number | null;
};

export type TSelectedMapSolarsystemBase = Omit<TMapSolarsystemBase, 'signatures'> & {
    notes: string | null;
    signatures: TSignature[];
    audits: TAudit[];
    map_connections: TMapConnection[];
};

export type TSelectedMapSolarsystem = TSelectedMapSolarsystemBase & {
    solarsystem?: TResolvedSolarsystem | null;
};

export type TResolvedSelectedMapSolarsystem = TSelectedMapSolarsystemBase & {
    solarsystem: TResolvedSolarsystem;
};

export type TResolvedMapRouteSolarsystem = Omit<TMapRouteSolarsystem, 'solarsystem'> & {
    solarsystem: TResolvedSolarsystem;
};

export type TResolvedMapNavigation = {
    destinations: TResolvedMapRouteSolarsystem[];
};

export type TSelectedMapSolarsystemStatic = {
    id: number;
    leads_to: string;
    name: string;
    maximum_lifetime: number;
    maximum_jump_mass: number;
    total_mass: number;
};

export type TTailoredConstellation = {
    id: number;
    name: string;
};

export type TSignatureCategory = {
    id: number;
    name: string;
};

export type TSignatureType = {
    id: number;
    name: string;
};

export type TMapSummary = {
    id: number;
    name: string;
    slug: string;
    map_solarsystems_count: number;
    map_user_setting: TMapUserSetting;
};

export type TMapInfo = {
    id: number;
    name: string;
    slug: string;
    map_user_settin: TMapUserSetting;
    owner: {
        id: number;
        name: string;
    };
};
