import type { TCharacterViewModel } from '@/components/characters/CharactersView.vue';
import type { TKillmailViewModel } from '@/components/map-killmails/KillmailsView.vue';
import type { TProcessedConnection } from '@/map/api';
import type { TMapSolarsystem, TResolvedSolarsystem, TSolarsystem } from '@/pages/maps';
import type { TCharacter, TSignature, TSignatureType, TType } from '@/types/models';
import type { TStaticSolarsystem } from '@/types/static-data';

/**
 * Static fake data for the public landing page. None of this touches the
 * backend — it exists purely to drive the real presentational components
 * (MapView, KillmailsView, CharactersView, SignaturesView) with believable
 * wormhole-space content.
 */

function ago(minutes: number): string {
    return new Date(Date.now() - minutes * 60_000).toISOString();
}

function ship(id: number, name: string, group_id: number): TType {
    return { id, name, group_id, market_group_id: null, description: null, icon_id: null, graphic_id: null };
}

const SHIPS = {
    loki: ship(29990, 'Loki', 963),
    astero: ship(33468, 'Astero', 1305),
    stratios: ship(33470, 'Stratios', 906),
    buzzard: ship(11188, 'Buzzard', 830),
    drake: ship(24698, 'Drake', 540),
    sabre: ship(22456, 'Sabre', 541),
    venture: ship(32880, 'Venture', 543),
    legion: ship(29986, 'Legion', 963),
    capsule: ship(670, 'Capsule', 29),
};

/**
 * Real solar systems pulled from resources/static/solarsystems.json (the same
 * dataset the live app uses) so every name, class, region, static and effect is
 * authentic. The chain is static-accurate too: every connection is either a
 * static of its source system or an inbound K162 opened by the other side's
 * static, so the topology could exist in game exactly as drawn.
 */
const SYSTEMS = {
    /** C5 home. Its H296 static leads to another C5 (alpha). */
    home: {
        id: 31001880,
        name: 'J152820',
        region_id: 11000024,
        constellation_id: 21000232,
        class: '5',
        security: -1,
        type: 'wh',
        region: { id: 11000024, name: 'E-R00024' },
        sovereignty: null,
        statics: [
            {
                id: 30702,
                leads_to: 'c5',
                name: 'H296',
                maximum_lifetime: 86400,
                maximum_jump_mass: 2_000_000_000,
                total_mass: 3_300_000_000,
                signature_strength: 10,
            },
        ],
        effect: { name: 'Pulsar', effects: [] },
        connection_type: null,
    },
    /** C5 behind home's H296 static; its own V753 static leads on to the C6 (delta). */
    alpha: {
        id: 31002063,
        name: 'J154846',
        region_id: 11000026,
        constellation_id: 21000256,
        class: '5',
        security: -1,
        type: 'wh',
        region: { id: 11000026, name: 'E-R00026' },
        sovereignty: null,
        statics: [
            {
                id: 30703,
                leads_to: 'c6',
                name: 'V753',
                maximum_lifetime: 86400,
                maximum_jump_mass: 2_000_000_000,
                total_mass: 3_300_000_000,
                signature_strength: 6.67,
            },
        ],
        effect: { name: 'Wolf-Rayet Star', effects: [] },
        connection_type: null,
    },
    /** C4 whose H900 static opened the K162 into home; its C247 static leads to the C3 (gamma). */
    beta: {
        id: 31001663,
        name: 'J231710',
        region_id: 11000021,
        constellation_id: 21000197,
        class: '4',
        security: -1,
        type: 'wh',
        region: { id: 11000021, name: 'D-R00021' },
        sovereignty: null,
        statics: [
            {
                id: 30693,
                leads_to: 'c5',
                name: 'H900',
                maximum_lifetime: 86400,
                maximum_jump_mass: 375_000_000,
                total_mass: 3_000_000_000,
                signature_strength: 2.5,
            },
            {
                id: 30691,
                leads_to: 'c3',
                name: 'C247',
                maximum_lifetime: 57600,
                maximum_jump_mass: 375_000_000,
                total_mass: 2_000_000_000,
                signature_strength: 10,
            },
        ],
        effect: { name: 'Magnetar', effects: [] },
        connection_type: null,
    },
    /** C3 behind the C4's C247 static; its D845 highsec static is the chain's exit to Jita. */
    gamma: {
        id: 31001073,
        name: 'J121856',
        region_id: 11000012,
        constellation_id: 21000106,
        class: '3',
        security: -1,
        type: 'wh',
        region: { id: 11000012, name: 'C-R00012' },
        sovereignty: null,
        statics: [
            {
                id: 30695,
                leads_to: 'hs',
                name: 'D845',
                maximum_lifetime: 86400,
                maximum_jump_mass: 375_000_000,
                total_mass: 5_000_000_000,
                signature_strength: 5,
            },
        ],
        effect: null,
        connection_type: null,
    },
    /** C6 behind alpha's V753 static — the dangerous end of the chain. */
    delta: {
        id: 31002367,
        name: 'J104859',
        region_id: 11000030,
        constellation_id: 21000297,
        class: '6',
        security: -1,
        type: 'wh',
        region: { id: 11000030, name: 'F-R00030' },
        sovereignty: null,
        statics: [
            {
                id: 30708,
                leads_to: 'c2',
                name: 'G024',
                maximum_lifetime: 57600,
                maximum_jump_mass: 375_000_000,
                total_mass: 2_000_000_000,
                signature_strength: 1.25,
            },
        ],
        effect: { name: 'Pulsar', effects: [] },
        connection_type: null,
    },
    /** Neighbouring C5 whose H296 static opened a second K162 into home. */
    epsilon: {
        id: 31001881,
        name: 'J135100',
        region_id: 11000024,
        constellation_id: 21000232,
        class: '5',
        security: -1,
        type: 'wh',
        region: { id: 11000024, name: 'E-R00024' },
        sovereignty: null,
        statics: [
            {
                id: 30702,
                leads_to: 'c5',
                name: 'H296',
                maximum_lifetime: 86400,
                maximum_jump_mass: 2_000_000_000,
                total_mass: 3_300_000_000,
                signature_strength: 10,
            },
        ],
        effect: null,
        connection_type: null,
    },
    /** Highsec exit behind the C3's D845 static. */
    jita: {
        id: 30000142,
        name: 'Jita',
        region_id: 10000002,
        constellation_id: 20000020,
        class: 'h',
        security: 0.9,
        type: 'eve',
        region: { id: 10000002, name: 'The Forge' },
        sovereignty: null,
        statics: null,
        effect: null,
        connection_type: null,
    },
} satisfies Record<string, TSolarsystem>;

function toStatic(system: TSolarsystem): TStaticSolarsystem {
    return { ...system, sovereignty: null, has_jove_observatory: false, has_stations: system.type === 'eve', services: [] };
}

function node(
    id: number,
    system: TSolarsystem,
    position: { x: number; y: number },
    status: TMapSolarsystem['status'],
    extra: Partial<TMapSolarsystem> = {},
): TMapSolarsystem {
    return {
        id,
        map_id: 1,
        solarsystem_id: system.id,
        alias: null,
        status,
        occupier_alias: null,
        notes: null,
        position,
        pinned: false,
        signatures_count: 0,
        wormhole_signatures_count: 0,
        map_connections_count: 0,
        uncategorized_signatures_count: 0,
        threat_level: null,
        signatures: null,
        solarsystem: system,
        ...extra,
    };
}

/**
 * A free-flow chain layout (positions are multiples of the grid size). HOME
 * sits on the left with its H296 static going up the chain toward the C6 and
 * two inbound K162s; the exit route runs through the C4 and C3 to Jita.
 */
const NODES = {
    home: node(1, SYSTEMS.home, { x: 80, y: 340 }, 'friendly', { signatures_count: 8, map_connections_count: 3, wormhole_signatures_count: 4 }),
    alpha: node(2, SYSTEMS.alpha, { x: 340, y: 180 }, 'empty', { signatures_count: 3, map_connections_count: 2 }),
    beta: node(3, SYSTEMS.beta, { x: 340, y: 340 }, 'active', { signatures_count: 5, map_connections_count: 2 }),
    gamma: node(4, SYSTEMS.gamma, { x: 600, y: 300 }, 'active', { signatures_count: 2, map_connections_count: 2 }),
    delta: node(5, SYSTEMS.delta, { x: 600, y: 120 }, 'hostile', { threat_level: 'critical', signatures_count: 6, map_connections_count: 1 }),
    epsilon: node(6, SYSTEMS.epsilon, { x: 340, y: 500 }, 'unscanned', { map_connections_count: 1 }),
    jita: node(7, SYSTEMS.jita, { x: 600, y: 460 }, 'unknown', { map_connections_count: 1 }),
};

export const MAP_SOLARSYSTEMS: TMapSolarsystem[] = Object.values(NODES);

function connection(
    id: number,
    source: TMapSolarsystem,
    target: TMapSolarsystem,
    mass_status: TProcessedConnection['mass_status'],
    lifetime_status: TProcessedConnection['lifetime_status'],
    ship_size: TProcessedConnection['ship_size'],
    is_on_route = false,
): TProcessedConnection {
    return {
        id,
        from_map_solarsystem_id: source.id,
        to_map_solarsystem_id: target.id,
        type: 'wormhole',
        preserve_mass: false,
        mass_status,
        lifetime_status,
        lifetime_status_updated_at: null,
        signatures: null,
        ship_size,
        jumps_mass_sum: 0,
        jumps_count: 0,
        created_at: ago(120),
        updated_at: ago(20),
        source,
        target,
        is_on_route,
    };
}

export const MAP_CONNECTIONS: TProcessedConnection[] = [
    connection(1, NODES.home, NODES.alpha, 'fresh', 'healthy', 'xlarge'),
    connection(2, NODES.beta, NODES.home, 'fresh', 'healthy', 'large', true),
    connection(3, NODES.epsilon, NODES.home, 'reduced', 'eol', 'xlarge'),
    connection(4, NODES.alpha, NODES.delta, 'critical', 'critical', 'xlarge'),
    connection(5, NODES.beta, NODES.gamma, 'fresh', 'healthy', 'large', true),
    connection(6, NODES.gamma, NODES.jita, 'reduced', 'healthy', 'large', true),
];

function character(
    id: number,
    name: string,
    ship: TType,
    system: TSolarsystem,
    online = true,
    extra: Partial<TCharacter['status']> = {},
): TCharacter {
    return {
        id,
        name,
        corporation: { id: 98000001, name: 'Wandering Phoenix', ticker: 'WPHX' },
        alliance: { id: 99000001, name: 'Hole Control', ticker: 'HOLE' },
        faction: null,
        status: {
            solarsystem_id: system.id,
            ship_type: ship,
            ship_name: `${name}'s ${ship.name}`,
            is_online: online,
            station_id: null,
            structure_id: null,
            last_online_at: ago(1),
            checked_last_online_at: ago(1),
            checked_location_at: ago(1),
            ...extra,
        },
    };
}

function route(systems: TSolarsystem[]): TResolvedSolarsystem[] {
    return systems.map((system, index) => ({
        ...system,
        connection_type: index < systems.length - 1 ? (system.type === 'wh' ? 'wormhole' : 'stargate') : null,
    }));
}

export const CHARACTERS: TCharacterViewModel[] = [
    {
        ...character(2112000001, 'Karima Solette', SHIPS.loki, SYSTEMS.home),
        route: route([SYSTEMS.home]),
        static_solarsystem: toStatic(SYSTEMS.home),
    },
    {
        ...character(2112000002, 'Tovan Khev', SHIPS.astero, SYSTEMS.alpha),
        route: route([SYSTEMS.home, SYSTEMS.alpha]),
        static_solarsystem: toStatic(SYSTEMS.alpha),
    },
    {
        ...character(2112000003, 'Aura Vex', SHIPS.stratios, SYSTEMS.gamma),
        route: route([SYSTEMS.home, SYSTEMS.beta, SYSTEMS.gamma]),
        static_solarsystem: toStatic(SYSTEMS.gamma),
    },
    {
        ...character(2112000004, 'Dagan Khar', SHIPS.buzzard, SYSTEMS.beta),
        route: route([SYSTEMS.home, SYSTEMS.beta]),
        static_solarsystem: toStatic(SYSTEMS.beta),
    },
    {
        ...character(2112000005, 'Rana Kel', SHIPS.capsule, SYSTEMS.home, false, { station_id: 60008494 }),
        route: route([SYSTEMS.home]),
        static_solarsystem: toStatic(SYSTEMS.home),
    },
];

export const MAP_PILOTS: Record<number, TCharacter[]> = {
    [NODES.home.id]: [CHARACTERS[0], CHARACTERS[4]],
    [NODES.gamma.id]: [CHARACTERS[2]],
};

function killmail(
    id: number,
    shipType: TType,
    system: TSolarsystem,
    totalValue: number,
    minutesAgo: number,
    flags: { solo?: boolean; npc?: boolean } = {},
): TKillmailViewModel {
    return {
        solarsystem: system,
        alias: null,
        killmail: {
            id,
            hash: `hash${id}`,
            solarsystem_id: system.id,
            ship_type: shipType,
            data: {
                victim: {
                    character_id: 2113000000 + id,
                    corporation_id: 98000050,
                    alliance_id: 99000050,
                    faction_id: null,
                    ship_type_id: shipType.id,
                    items: [],
                },
                attackers: Array.from({ length: flags.solo ? 1 : 3 + (id % 4) }, (_, i) => ({
                    character_id: 2114000000 + id * 10 + i,
                    corporation_id: 98000060,
                    alliance_id: 99000060,
                    faction_id: null,
                    ship_type_id: SHIPS.sabre.id,
                    damage_done: 1000,
                    final_blow: i === 0,
                    weapons_type_id: null,
                })),
            },
            zkb: {
                awox: false,
                destroyedValue: totalValue * 0.7,
                droppedValue: totalValue * 0.3,
                fittedValue: totalValue,
                hash: `hash${id}`,
                labels: [],
                locationID: 0,
                npc: flags.npc ?? false,
                points: 1,
                solo: flags.solo ?? false,
                totalValue,
            },
            victim_corporation: { id: 98000050, name: 'Lost Capsuleers', ticker: 'LOST' },
            victim_alliance: { id: 99000050, name: 'Drifter Watch', ticker: 'DRFT' },
            time: ago(minutesAgo),
        },
    };
}

/**
 * Built per-render (call it inside setup) so the relative timestamps reflect
 * the current request time on both the SSR server and the client, keeping the
 * hydrated markup in sync.
 */
export function buildKillmails(): TKillmailViewModel[] {
    return [
        killmail(101, SHIPS.astero, SYSTEMS.delta, 142_000_000, 1),
        killmail(102, SHIPS.drake, SYSTEMS.beta, 88_000_000, 6, { solo: true }),
        killmail(103, SHIPS.sabre, SYSTEMS.home, 61_000_000, 12),
        killmail(104, SHIPS.legion, SYSTEMS.gamma, 412_000_000, 23),
        killmail(105, SHIPS.venture, SYSTEMS.alpha, 12_000_000, 41),
    ];
}

/** Real signature categories, matching resources/js/data/signatures.json. */
const SIGNATURE_CATEGORIES: Record<string, { id: number; code: string }> = {
    Wormhole: { id: 1, code: 'wormhole' },
    'Data Site': { id: 2, code: 'data' },
    'Relic Site': { id: 3, code: 'relic' },
    'Combat Site': { id: 4, code: 'combat' },
    'Gas Site': { id: 5, code: 'gas' },
    'Ore Site': { id: 6, code: 'ore' },
    'Homefront Operations': { id: 7, code: 'homefront' },
};

interface SignatureInput {
    id: number;
    signature_id: string;
    categoryName: keyof typeof SIGNATURE_CATEGORIES;
    minutesAgo: number;
    lifetime?: TSignature['lifetime'];
    mass?: TSignature['mass_status'];
    map_connection_id?: number | null;
    /** Wormhole sigs: the wormhole code (e.g. K162) and where it leads. */
    code?: string;
    target_class?: string;
    extra?: string | null;
    /** Non-wormhole sigs: the resolved site name. */
    typeName?: string;
}

function signature(input: SignatureInput): TSignature {
    const category = SIGNATURE_CATEGORIES[input.categoryName];
    const isWormhole = input.categoryName === 'Wormhole';
    const lifetime = input.lifetime ?? 'healthy';

    const signature_type: TSignature['signature_type'] = isWormhole
        ? ({
              id: input.id,
              name: `${input.code} - ${input.target_class}`,
              signature: input.code!,
              signature_category_id: category.id,
              category_name: 'Wormhole',
              target_class: input.target_class ?? null,
              extra: input.extra ?? null,
              spawn_areas: null,
          } as TSignatureType)
        : input.typeName
          ? {
                id: input.id,
                name: input.typeName,
                signature: input.signature_id,
                signature_category_id: category.id,
                category_name: input.categoryName,
                target_class: null,
                extra: null,
                spawn_areas: null,
            }
          : null;

    return {
        id: input.id,
        map_solarsystem_id: 1,
        map_connection_id: input.map_connection_id ?? null,
        signature_id: input.signature_id,
        signature_type_id: signature_type?.id ?? null,
        signature_category_id: category.id,
        raw_type_name: isWormhole ? null : (input.typeName ?? null),
        signature_type,
        signature_category: { id: category.id, name: input.categoryName, code: category.code, created_at: ago(0), updated_at: ago(0) },
        mass_status: input.mass ?? null,
        ship_size: null,
        lifetime,
        lifetime_updated_at: lifetime !== 'healthy' ? ago(Math.round(input.minutesAgo / 2)) : null,
        created_at: ago(input.minutesAgo),
        updated_at: ago(input.minutesAgo),
        wormhole: null,
        map_connection: null,
    };
}

/** Built per-render (call inside setup) — see {@link buildKillmails}. */
export function buildSignatures(): TSignature[] {
    return [
        // Wormholes in HOME, wired to the mapped connections: the H296 static
        // plus the K162 sides of the holes the C4 and C5 opened into us.
        signature({
            id: 701,
            signature_id: 'AZX-114',
            categoryName: 'Wormhole',
            code: 'H296',
            target_class: '5',
            map_connection_id: 1,
            minutesAgo: 18,
        }),
        signature({
            id: 702,
            signature_id: 'JKL-204',
            categoryName: 'Wormhole',
            code: 'K162',
            target_class: '4',
            map_connection_id: 2,
            minutesAgo: 124,
        }),
        signature({
            id: 703,
            signature_id: 'QRS-557',
            categoryName: 'Wormhole',
            code: 'K162',
            target_class: '5',
            map_connection_id: 3,
            lifetime: 'eol',
            mass: 'reduced',
            minutesAgo: 6,
        }),
        // Wandering K162, not yet linked to a system on the map.
        signature({ id: 704, signature_id: 'HMF-301', categoryName: 'Wormhole', code: 'K162', target_class: '3', minutesAgo: 41 }),
        // Cosmic signatures (no connection).
        signature({ id: 705, signature_id: 'DAT-209', categoryName: 'Data Site', typeName: 'Unsecured Frontier Database', minutesAgo: 63 }),
        signature({
            id: 706,
            signature_id: 'RLC-443',
            categoryName: 'Relic Site',
            typeName: 'Forgotten Frontier Quarantine Outpost',
            minutesAgo: 182,
        }),
        signature({ id: 707, signature_id: 'GAS-118', categoryName: 'Gas Site', typeName: 'Bountiful Frontier Reservoir', minutesAgo: 96 }),
        signature({ id: 708, signature_id: 'CMB-660', categoryName: 'Combat Site', typeName: 'Fortification Frontier Stronghold', minutesAgo: 240 }),
    ];
}
