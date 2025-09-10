// Combat sites ===================================================================================================

const c1_combat = ['Perimeter Ambush Point', 'Perimeter Camp', 'Phase Catalyst Node', 'The Line'];

const c2_combat = ['Perimeter Checkpoint', 'Perimeter Hangar', 'The Ruins of Enclave Cohort 27', 'Sleeper Data Sanctuary'];

const c3_combat = ['Fortification Frontier Stronghold', 'Outpost Frontier Stronghold', 'Solar Cell', 'The Oruze Construct'];

const c4_combat = ['Frontier Barracks', 'Frontier Command Post', 'Integrated Terminus', 'Sleeper Information Sanctum'];

const c5_combat = ['Core Garrison', 'Core Stronghold', 'Oruze Osobnyk', 'Quarantine Area'];

const c6_combat = ['Core Citadel', 'Core Bastion', 'Strange Energy Readings', 'The Mirror'];

// Thera WH
const c12_combat = ['Epicenter', 'Expedition Command Outpost Wreck', 'Planetary Colonization Office Wreck', 'Testing Facilities'];

// Drifter Sentinel WH
const c14_combat = ['Monolith', 'Wormhole in Rock Circle', 'Opposing Spatial Rifts', 'Sleeper Enclave Debris', 'Crystal Resource'];

// Drifter Barbican WH
const c15_combat = ['Wrecked Ships', 'Unstable Wormhole', 'Spatial Rift', 'Heavily Guarded Spatial Rift', 'Crystals'];

// Drifter Vidette WH
const c16_combat = ['Ship Graveyard', 'Sleeper Engineering Station', 'Spatial Rift', 'Sleeper Enclave in Coral Rock', 'Crystals and Stone Circle'];

// Drifter Conflux WH
const c17_combat = ['Monolith', 'Caged Wormhole', 'Rock Formation and Wormhole', 'Particle Acceleration Array', 'Guarded Asteroid Station'];

// Drifter Redoubt WH
const c18_combat = ['Ship Graveyard', 'Caged Wormhole', 'Spatial Rift Generator', 'Sleeper Enclave', 'Hollow Asteroid'];

// Relic sites ====================================================================================================

const null_relic = [
    'Ruined Angel Crystal Quarry',
    'Ruined Angel Monument Site',
    'Ruined Angel Science Outpost',
    'Ruined Angel Temple Site',
    'Ruined Blood Raider Crystal Quarry',
    'Ruined Blood Raider Monument Site',
    'Ruined Blood Raider Science Outpost',
    'Ruined Blood Raider Temple Site',
    'Ruined Guristas Crystal Quarry',
    'Ruined Guristas Monument Site',
    'Ruined Guristas Science Outpost',
    'Ruined Guristas Temple Site',
    'Ruined Sansha Crystal Quarry',
    'Ruined Sansha Monument Site',
    'Ruined Sansha Science Outpost',
    'Ruined Sansha Temple Site',
    'Ruined Serpentis Crystal Quarry',
    'Ruined Serpentis Monument Site',
    'Ruined Serpentis Science Outpost',
    'Ruined Serpentis Temple Site',
];

const c1_relic = ['Forgotten Perimeter Coronation Platform', 'Forgotten Perimeter Power Array'];

const c2_relic = ['Forgotten Perimeter Gateway', 'Forgotten Perimeter Habitation Coils'];

const c3_relic = ['Forgotten Frontier Quarantine Outpost', 'Forgotten Frontier Recursive Depot'];

const c4_relic = ['Forgotten Frontier Conversion Module', 'Forgotten Frontier Evacuation Center'];

const c5_relic = ['Forgotten Core Data Field', 'Forgotten Core Information Pen'];

const c6_relic = ['Forgotten Core Assembly Hall', 'Forgotten Core Circuitry Disassembler'];

// Data sites =====================================================================================================

const null_data = [
    'Abandoned Research Complex DA005',
    'Abandoned Research Complex DA015',
    'Abandoned Research Complex DC007',
    'Abandoned Research Complex DC021',
    'Abandoned Research Complex DC035',
    'Abandoned Research Complex DG003',
    'Central Angel Command Center',
    'Central Angel Data Mining Site',
    'Central Angel Sparking Transmitter',
    'Central Angel Survey Site',
    'Central Blood Raider Command Center',
    'Central Blood Raider Data Mining Site',
    'Central Blood Raider Sparking Transmitter',
    'Central Blood Raider Survey Site',
    'Central Guristas Command Center',
    'Central Guristas Data Mining Site',
    'Central Guristas Sparking Transmitter',
    'Central Guristas Survey Site',
    'Central Sansha Command Center',
    'Central Sansha Data Mining Site',
    'Central Sansha Sparking Transmitter',
    'Central Sansha Survey Site',
    'Central Serpentis Command Center',
    'Central Serpentis Data Mining Site',
    'Central Serpentis Sparking Transmitter',
    'Central Serpentis Survey Site',
];

const c1_data = ['Unsecured Perimeter Amplifier', 'Unsecured Perimeter Information Center'];

const c2_data = ['Unsecured Perimeter Comms Relay', 'Unsecured Perimeter Transponder Farm'];

const c3_data = ['Unsecured Frontier Database', 'Unsecured Frontier Receiver'];

const c4_data = ['Unsecured Frontier Digital Nexus', 'Unsecured Frontier Trinary Hub'];

const c5_data = ['Unsecured Frontier Enclave Relay', 'Unsecured Frontier Server Bank'];

const c6_data = ['Unsecured Core Backup Array', 'Unsecured Core Emergence'];

// Ghost sites ======================================================================================================

const hs_gh = [
    'Lesser Serpentis Covert Research Facility',
    'Lesser Sansha Covert Research Facility',
    'Lesser Guristas Covert Research Facility',
    'Lesser Blood Raiders Covert Research Facility',
];

const ls_gh = [
    'Standard Serpentis Covert Research Facility',
    'Standard Sansha Covert Research Facility',
    'Standard Guristas Covert Research Facility',
    'Standard Blood Raiders Covert Research Facility',
];

const ns_gh = [
    'Improved Serpentis Covert Research Facility',
    'Improved Sansha Covert Research Facility',
    'Improved Guristas Covert Research Facility',
    'Improved Blood Raiders Covert Research Facility',
];

const wh_gh = [
    'Superior Serpentis Covert Research Facility',
    'Superior Sansha Covert Research Facility',
    'Superior Guristas Covert Research Facility',
    'Superior Blood Raiders Covert Research Facility',
];

// Gas sites ======================================================================================================

const c1_gas = [
    'Barren Perimeter Reservoir',
    'Token Perimeter Reservoir',
    'Minor Perimeter Reservoir',
    'Sizeable Perimeter Reservoir',
    'Ordinary Perimeter Reservoir',
];

const c2_gas = [
    'Barren Perimeter Reservoir',
    'Token Perimeter Reservoir',
    'Minor Perimeter Reservoir',
    'Sizeable Perimeter Reservoir',
    'Ordinary Perimeter Reservoir',
];

const c3_gas = [
    'Barren Perimeter Reservoir',
    'Token Perimeter Reservoir',
    'Minor Perimeter Reservoir',
    'Sizeable Perimeter Reservoir',
    'Ordinary Perimeter Reservoir',
    'Bountiful Frontier Reservoir',
    'Vast Frontier Reservoir',
];

const c4_gas = [
    'Barren Perimeter Reservoir',
    'Token Perimeter Reservoir',
    'Minor Perimeter Reservoir',
    'Sizeable Perimeter Reservoir',
    'Ordinary Perimeter Reservoir',
    'Vast Frontier Reservoir',
    'Bountiful Frontier Reservoir',
];

const c5_gas = [
    'Barren Perimeter Reservoir',
    'Minor Perimeter Reservoir',
    'Ordinary Perimeter Reservoir',
    'Sizeable Perimeter Reservoir',
    'Token Perimeter Reservoir',
    'Bountiful Frontier Reservoir',
    'Vast Frontier Reservoir',
    'Instrumental Core Reservoir',
    'Vital Core Reservoir',
];

const c6_gas = [
    'Barren Perimeter Reservoir',
    'Minor Perimeter Reservoir',
    'Ordinary Perimeter Reservoir',
    'Sizeable Perimeter Reservoir',
    'Token Perimeter Reservoir',
    'Bountiful Frontier Reservoir',
    'Vast Frontier Reservoir',
    'Instrumental Core Reservoir',
    'Vital Core Reservoir',
];

// Ore sites ======================================================================================================

const c1_ore = [
    'Ordinary Perimeter Deposit',
    'Common Perimeter Deposit',
    'Unexceptional Frontier Deposit',
    'Average Frontier Deposit',
    'Isolated Core Deposit',
    'Uncommon Core Deposit',
];

const c2_ore = [
    'Ordinary Perimeter Deposit',
    'Common Perimeter Deposit',
    'Unexceptional Frontier Deposit',
    'Average Frontier Deposit',
    'Isolated Core Deposit',
    'Uncommon Core Deposit',
];

const c3_ore = [
    'Ordinary Perimeter Deposit',
    'Common Perimeter Deposit',
    'Unexceptional Frontier Deposit',
    'Average Frontier Deposit',
    'Infrequent Core Deposit',
    'Unusual Core Deposit',
];

const c4_ore = [
    'Ordinary Perimeter Deposit',
    'Common Perimeter Deposit',
    'Unexceptional Frontier Deposit',
    'Average Frontier Deposit',
    'Unusual Core Deposit',
    'Infrequent Core Deposit',
];

const c5_ore = [
    'Average Frontier Deposit',
    'Unexceptional Frontier Deposit',
    'Uncommon Core Deposit',
    'Ordinary Perimeter Deposit',
    'Common Perimeter Deposit',
    'Exceptional Core Deposit',
    'Infrequent Core Deposit',
    'Unusual Core Deposit',
    'Rarified Core Deposit',
    'Isolated Core Deposit',
];

const c6_ore = [
    'Ordinary Perimeter Deposit',
    'Common Perimeter Deposit',
    'Unexceptional Frontier Deposit',
    'Average Frontier Deposit',
    'Rarified Core Deposit',
];

// Shattered WH
const c13_ore = ['Shattered Debris Field', 'Shattered Ice Field'];

// Wormholes ======================================================================================================

export type TWormholeDefinition = {
    name: string;
    signature: string;
    target_class: string;
    extra: string;
};

// K162 exit wormholes record (indexed by signature for sorting control)
const k162_signatures: Record<string, TWormholeDefinition> = {
    K162_Unknown: { name: 'K162 - Unknown', signature: 'K162', target_class: 'Unknown', extra: '' },
    K162_C1: { name: 'K162 - C1', signature: 'K162', target_class: 'C1', extra: '' },
    K162_C2: { name: 'K162 - C2', signature: 'K162', target_class: 'C2', extra: '' },
    K162_C3: { name: 'K162 - C3', signature: 'K162', target_class: 'C3', extra: '' },
    K162_C4: { name: 'K162 - C4', signature: 'K162', target_class: 'C4', extra: '' },
    K162_C5: { name: 'K162 - C5', signature: 'K162', target_class: 'C5', extra: '' },
    K162_C6: { name: 'K162 - C6', signature: 'K162', target_class: 'C6', extra: '' },
    K162_H: { name: 'K162 - H', signature: 'K162', target_class: 'H', extra: '' },
    K162_L: { name: 'K162 - L', signature: 'K162', target_class: 'L', extra: '' },
    K162_N: { name: 'K162 - N', signature: 'K162', target_class: 'N', extra: '' },
};

// All normal wormhole signatures (indexed by signature for sorting control)
const normal_wh_signatures: Record<string, TWormholeDefinition> = {
    // Universal spawns (can appear from any system)
    A009: { name: 'A009 - C13', signature: 'A009', target_class: 'C13', extra: '' },
    B735: { name: 'B735 - Barbican (C15)', signature: 'B735', target_class: 'C15', extra: 'Barbican' },
    C008: { name: 'C008 - C5', signature: 'C008', target_class: 'C5', extra: '' },
    C414: { name: 'C414 - Conflux (C17)', signature: 'C414', target_class: 'C17', extra: 'Conflux' },
    E004: { name: 'E004 - C1', signature: 'E004', target_class: 'C1', extra: '' },
    G008: { name: 'G008 - C6', signature: 'G008', target_class: 'C6', extra: '' },
    J492: { name: 'J492 - L', signature: 'J492', target_class: 'L', extra: '' },
    L005: { name: 'L005 - C2', signature: 'L005', target_class: 'C2', extra: '' },
    M001: { name: 'M001 - C4', signature: 'M001', target_class: 'C4', extra: '' },
    Q003: { name: 'Q003 - 0.0', signature: 'Q003', target_class: 'N', extra: '' },
    R259: { name: 'R259 - Redoubt (C18)', signature: 'R259', target_class: 'C18', extra: 'Redoubt' },
    S877: { name: 'S877 - Sentinel (C14)', signature: 'S877', target_class: 'C14', extra: 'Sentinel' },
    V928: { name: 'V928 - Vidette (C16)', signature: 'V928', target_class: 'C16', extra: 'Vidette' },
    Z006: { name: 'Z006 - C3', signature: 'Z006', target_class: 'C3', extra: '' },

    // C1 specific
    H121: { name: 'H121 - C1', signature: 'H121', target_class: 'C1', extra: '' },
    C125: { name: 'C125 - C2', signature: 'C125', target_class: 'C2', extra: '' },
    O883: { name: 'O883 - C3', signature: 'O883', target_class: 'C3', extra: '' },
    M609: { name: 'M609 - C4', signature: 'M609', target_class: 'C4', extra: '' },
    L614: { name: 'L614 - C5', signature: 'L614', target_class: 'C5', extra: '' },
    S804: { name: 'S804 - C6', signature: 'S804', target_class: 'C6', extra: '' },
    N110: { name: 'N110 - H', signature: 'N110', target_class: 'H', extra: '' },
    J244: { name: 'J244 - L', signature: 'J244', target_class: 'L', extra: '' },
    Z060: { name: 'Z060 - 0.0', signature: 'Z060', target_class: 'N', extra: '' },
    F353: { name: 'F353 - C12 Thera', signature: 'F353', target_class: 'C12', extra: 'Thera' },

    // C2 specific
    Z647: { name: 'Z647 - C1', signature: 'Z647', target_class: 'C1', extra: '' },
    D382: { name: 'D382 - C2', signature: 'D382', target_class: 'C2', extra: '' },
    O477: { name: 'O477 - C3', signature: 'O477', target_class: 'C3', extra: '' },
    Y683: { name: 'Y683 - C4', signature: 'Y683', target_class: 'C4', extra: '' },
    N062: { name: 'N062 - C5', signature: 'N062', target_class: 'C5', extra: '' },
    R474: { name: 'R474 - C6', signature: 'R474', target_class: 'C6', extra: '' },
    B274: { name: 'B274 - H', signature: 'B274', target_class: 'H', extra: '' },
    A239: { name: 'A239 - L', signature: 'A239', target_class: 'L', extra: '' },
    E545: { name: 'E545 - 0.0', signature: 'E545', target_class: 'N', extra: '' },
    F135: { name: 'F135 - C12 Thera', signature: 'F135', target_class: 'C12', extra: 'Thera' },

    // C3 specific
    V301: { name: 'V301 - C1', signature: 'V301', target_class: 'C1', extra: '' },
    I182: { name: 'I182 - C2', signature: 'I182', target_class: 'C2', extra: '' },
    N968: { name: 'N968 - C3', signature: 'N968', target_class: 'C3', extra: '' },
    T405: { name: 'T405 - C4', signature: 'T405', target_class: 'C4', extra: '' },
    N770: { name: 'N770 - C5', signature: 'N770', target_class: 'C5', extra: '' },
    A982: { name: 'A982 - C6', signature: 'A982', target_class: 'C6', extra: '' },
    D845: { name: 'D845 - H', signature: 'D845', target_class: 'H', extra: '' },
    U210: { name: 'U210 - L', signature: 'U210', target_class: 'L', extra: '' },
    K346: { name: 'K346 - 0.0', signature: 'K346', target_class: 'N', extra: '' },

    // C4 specific
    P060: { name: 'P060 - C1', signature: 'P060', target_class: 'C1', extra: '' },
    N766: { name: 'N766 - C2', signature: 'N766', target_class: 'C2', extra: '' },
    C247: { name: 'C247 - C3', signature: 'C247', target_class: 'C3', extra: '' },
    X877: { name: 'X877 - C4', signature: 'X877', target_class: 'C4', extra: '' },
    H900: { name: 'H900 - C5', signature: 'H900', target_class: 'C5', extra: '' },
    U574: { name: 'U574 - C6', signature: 'U574', target_class: 'C6', extra: '' },
    S047: { name: 'S047 - H', signature: 'S047', target_class: 'H', extra: '' },
    N290: { name: 'N290 - L', signature: 'N290', target_class: 'L', extra: '' },
    K329: { name: 'K329 - 0.0', signature: 'K329', target_class: 'N', extra: '' },

    // C5 specific
    Y790: { name: 'Y790 - C1', signature: 'Y790', target_class: 'C1', extra: '' },
    D364: { name: 'D364 - C2', signature: 'D364', target_class: 'C2', extra: '' },
    M267: { name: 'M267 - C3', signature: 'M267', target_class: 'C3', extra: '' },
    E175: { name: 'E175 - C4', signature: 'E175', target_class: 'C4', extra: '' },
    H296: { name: 'H296 - C5', signature: 'H296', target_class: 'C5', extra: '' },
    V753: { name: 'V753 - C6', signature: 'V753', target_class: 'C6', extra: '' },
    D792: { name: 'D792 - H', signature: 'D792', target_class: 'H', extra: '' },
    C140: { name: 'C140 - L', signature: 'C140', target_class: 'L', extra: '' },
    Z142: { name: 'Z142 - 0.0', signature: 'Z142', target_class: 'N', extra: '' },

    // C6 specific
    Q317: { name: 'Q317 - C1', signature: 'Q317', target_class: 'C1', extra: '' },
    G024: { name: 'G024 - C2', signature: 'G024', target_class: 'C2', extra: '' },
    L477: { name: 'L477 - C3', signature: 'L477', target_class: 'C3', extra: '' },
    Z457: { name: 'Z457 - C4', signature: 'Z457', target_class: 'C4', extra: '' },
    V911: { name: 'V911 - C5', signature: 'V911', target_class: 'C5', extra: '' },
    W237: { name: 'W237 - C6', signature: 'W237', target_class: 'C6', extra: '' },
    B520: { name: 'B520 - H', signature: 'B520', target_class: 'H', extra: '' },
    C391: { name: 'C391 - L', signature: 'C391', target_class: 'L', extra: '' },
    C248: { name: 'C248 - 0.0', signature: 'C248', target_class: 'N', extra: '' },

    // Highsec specific
    Z971: { name: 'Z971 - C1', signature: 'Z971', target_class: 'C1', extra: '' },
    R943: { name: 'R943 - C2', signature: 'R943', target_class: 'C2', extra: '' },
    X702: { name: 'X702 - C3', signature: 'X702', target_class: 'C3', extra: '' },
    O128: { name: 'O128 - C4', signature: 'O128', target_class: 'C4', extra: '' },
    M555: { name: 'M555 - C5', signature: 'M555', target_class: 'C5', extra: '' },
    B041: { name: 'B041 - C6', signature: 'B041', target_class: 'C6', extra: '' },
    A641: { name: 'A641 - H', signature: 'A641', target_class: 'H', extra: '' },
    R051: { name: 'R051 - L', signature: 'R051', target_class: 'L', extra: '' },
    V283: { name: 'V283 - 0.0', signature: 'V283', target_class: 'N', extra: '' },
    T458: { name: 'T458 - C12 Thera', signature: 'T458', target_class: 'C12', extra: 'Thera' },

    // Lowsec/Nullsec specific
    N432: { name: 'N432 - C5', signature: 'N432', target_class: 'C5', extra: '' },
    U319: { name: 'U319 - C6', signature: 'U319', target_class: 'C6', extra: '' },
    B449: { name: 'B449 - H', signature: 'B449', target_class: 'H', extra: '' },
    N944: { name: 'N944 - L', signature: 'N944', target_class: 'L', extra: '' },
    S199: { name: 'S199 - 0.0', signature: 'S199', target_class: 'N', extra: '' },
    M164: { name: 'M164 - C12 Thera', signature: 'M164', target_class: 'C12', extra: 'Thera' },
    L031: { name: 'L031 - C12 Thera', signature: 'L031', target_class: 'C12', extra: 'Thera' },

    // Pochven specific
    R081: { name: 'R081 - C4', signature: 'R081', target_class: 'C4', extra: '' },
    X450: { name: 'X450 - 0.0', signature: 'X450', target_class: 'N', extra: '' },
    C729: { name: 'C729 - T Pochven', signature: 'C729', target_class: 'N', extra: 'Pochven' },
    U372: { name: 'U372 - T Pochven', signature: 'U372', target_class: 'N', extra: 'Pochven' },

    // Thera specific
    Q063: { name: 'Q063 - H Thera', signature: 'Q063', target_class: 'H', extra: 'Thera' },
    V898: { name: 'V898 - L Thera', signature: 'V898', target_class: 'L', extra: 'Thera' },
    E587: { name: 'E587 - 0.0 Thera', signature: 'E587', target_class: 'N', extra: 'Thera' },

    // Special entries
    J377: { name: 'J377 - L Turnur', signature: 'J377', target_class: 'L', extra: 'Turnur' },
    F216: { name: 'F216 - T Pochven', signature: 'F216', target_class: 'N', extra: 'Pochven' },
    U201: { name: 'U201 - L', signature: 'U201', target_class: 'L', extra: '' },
};

// Shorter aliases for cleaner syntax
const wh = normal_wh_signatures;
const k162 = k162_signatures;

// ================================================================================================================
//  Signature Tree Structure
// ================================================================================================================

export const signature_tree = {
    shared: {
        null_relic: null_relic,
        null_data: null_data,
    },
    wormhole_space: {
        1: {
            'Combat Site': c1_combat,
            'Relic Site': [...c1_relic, ...null_relic],
            'Data Site': [...c1_data, ...null_data, ...wh_gh],
            'Gas Site': c1_gas,
            Wormhole: [
                // K162s ordered by target class (H → L → N → C1 → C2/C3 → C4/C5 → C6 → Unknown)
                k162.K162_H,
                k162.K162_L,
                k162.K162_N,
                k162.K162_C1,
                k162.K162_C2,
                k162.K162_C3,
                k162.K162_C4,
                k162.K162_C5,
                k162.K162_C6,
                k162.K162_Unknown,
                // Normal wormholes ordered by target class (H → L → N → C1 → C2 → C3 → C4 → C5 → C6 → C12 → C13)
                wh.N110, // H
                wh.J244, // L
                wh.J377, // L (Turnur)
                wh.J492, // L (universal)
                wh.Z060, // N
                wh.Q003, // N (universal)
                wh.H121, // C1
                wh.E004, // C1 (universal)
                wh.C125, // C2
                wh.L005, // C2 (universal)
                wh.O883, // C3
                wh.Z006, // C3 (universal)
                wh.M609, // C4
                wh.M001, // C4 (universal)
                wh.L614, // C5
                wh.C008, // C5 (universal)
                wh.S804, // C6
                wh.G008, // C6 (universal)
                wh.F353, // C12 (Thera)
                wh.A009, // C13 (universal)
                wh.S877, // C14 (universal)
                wh.B735, // C15 (universal)
                wh.V928, // C16 (universal)
                wh.C414, // C17 (universal)
                wh.R259, // C18 (universal)
            ],
            'Ore Site': c1_ore,
        },
        2: {
            'Combat Site': c2_combat,
            'Relic Site': [...c2_relic, ...null_relic],
            'Data Site': [...c2_data, ...null_data, ...wh_gh],
            'Gas Site': c2_gas,
            Wormhole: [
                // K162s ordered by target class (H → L → N → C1 → C2/C3 → C4/C5 → C6 → Unknown)
                k162.K162_H,
                k162.K162_L,
                k162.K162_N,
                k162.K162_C1,
                k162.K162_C2,
                k162.K162_C3,
                k162.K162_C4,
                k162.K162_C5,
                k162.K162_C6,
                k162.K162_Unknown,
                // Normal wormholes ordered by target class (H → L → N → C1 → C2 → C3 → C4 → C5 → C6 → C12 → C13)
                wh.B274, // H
                wh.A239, // L
                wh.J377, // L (Turnur)
                wh.J492, // L (universal)
                wh.E545, // N
                wh.Q003, // N (universal)
                wh.Z647, // C1
                wh.E004, // C1 (universal)
                wh.D382, // C2
                wh.L005, // C2 (universal)
                wh.O477, // C3
                wh.Z006, // C3 (universal)
                wh.Y683, // C4
                wh.M001, // C4 (universal)
                wh.N062, // C5
                wh.C008, // C5 (universal)
                wh.R474, // C6
                wh.G008, // C6 (universal)
                wh.F135, // C12 (Thera)
                wh.A009, // C13 (universal)
                wh.S877, // C14 (universal)
                wh.B735, // C15 (universal)
                wh.V928, // C16 (universal)
                wh.C414, // C17 (universal)
                wh.R259, // C18 (universal)
                wh.F216, // Pochven
            ],
            'Ore Site': c2_ore,
        },
        3: {
            'Combat Site': c3_combat,
            'Relic Site': [...c3_relic, ...null_relic],
            'Data Site': [...c3_data, ...null_data, ...wh_gh],
            'Gas Site': c3_gas,
            Wormhole: [
                // K162s ordered by target class (H → L → N → C1 → C2/C3 → C4/C5 → C6 → Unknown)
                k162.K162_H,
                k162.K162_L,
                k162.K162_N,
                k162.K162_C1,
                k162.K162_C2,
                k162.K162_C3,
                k162.K162_C4,
                k162.K162_C5,
                k162.K162_C6,
                k162.K162_Unknown,
                // Normal wormholes ordered by target class (H → L → N → C1 → C2 → C3 → C4 → C5 → C6 → C12 → C13)
                wh.D845, // H
                wh.U210, // L
                wh.J377, // L (Turnur)
                wh.J492, // L (universal)
                wh.K346, // N
                wh.Q003, // N (universal)
                wh.V301, // C1
                wh.E004, // C1 (universal)
                wh.I182, // C2
                wh.L005, // C2 (universal)
                wh.N968, // C3
                wh.Z006, // C3 (universal)
                wh.T405, // C4
                wh.M001, // C4 (universal)
                wh.N770, // C5
                wh.C008, // C5 (universal)
                wh.A982, // C6
                wh.G008, // C6 (universal)
                wh.F135, // C12 (Thera)
                wh.A009, // C13 (universal)
                wh.S877, // C14 (universal)
                wh.B735, // C15 (universal)
                wh.V928, // C16 (universal)
                wh.C414, // C17 (universal)
                wh.R259, // C18 (universal)
                wh.F216, // Pochven
            ],
            'Ore Site': c3_ore,
        },
        4: {
            'Combat Site': c4_combat,
            'Relic Site': c4_relic,
            'Data Site': [...c4_data, ...wh_gh],
            'Gas Site': c4_gas,
            Wormhole: [
                // K162s ordered by target class (H → L → N → C1 → C2/C3 → C4/C5 → C6 → Unknown)
                k162.K162_H,
                k162.K162_L,
                k162.K162_N,
                k162.K162_C1,
                k162.K162_C2,
                k162.K162_C3,
                k162.K162_C4,
                k162.K162_C5,
                k162.K162_C6,
                k162.K162_Unknown,
                // Normal wormholes ordered by target class (H → L → N → C1 → C2 → C3 → C4 → C5 → C6 → C12 → C13)
                wh.S047, // H
                wh.N290, // L
                wh.J377, // L (Turnur)
                wh.J492, // L (universal)
                wh.K329, // N
                wh.Q003, // N (universal)
                wh.P060, // C1
                wh.E004, // C1 (universal)
                wh.N766, // C2
                wh.L005, // C2 (universal)
                wh.C247, // C3
                wh.Z006, // C3 (universal)
                wh.X877, // C4
                wh.M001, // C4 (universal)
                wh.H900, // C5
                wh.C008, // C5 (universal)
                wh.U574, // C6
                wh.G008, // C6 (universal)
                wh.A009, // C13 (universal)
                wh.S877, // C14 (universal)
                wh.B735, // C15 (universal)
                wh.V928, // C16 (universal)
                wh.C414, // C17 (universal)
                wh.R259, // C18 (universal)
                wh.F216, // Pochven
            ],
            'Ore Site': c4_ore,
        },
        5: {
            'Combat Site': c5_combat,
            'Relic Site': c5_relic,
            'Data Site': [...c5_data, ...wh_gh],
            'Gas Site': c5_gas,
            Wormhole: [
                // K162s ordered by target class (H → L → N → C1 → C2/C3 → C4/C5 → C6 → Unknown)
                k162.K162_H,
                k162.K162_L,
                k162.K162_N,
                k162.K162_C1,
                k162.K162_C2,
                k162.K162_C3,
                k162.K162_C4,
                k162.K162_C5,
                k162.K162_C6,
                k162.K162_Unknown,
                // Normal wormholes ordered by target class (H → L → N → C1 → C2 → C3 → C4 → C5 → C6 → C12 → C13)
                wh.D792, // H
                wh.C140, // L
                wh.J377, // L (Turnur)
                wh.J492, // L (universal)
                wh.Z142, // N
                wh.Q003, // N (universal)
                wh.Y790, // C1
                wh.E004, // C1 (universal)
                wh.D364, // C2
                wh.L005, // C2 (universal)
                wh.M267, // C3
                wh.Z006, // C3 (universal)
                wh.E175, // C4
                wh.M001, // C4 (universal)
                wh.H296, // C5
                wh.C008, // C5 (universal)
                wh.V753, // C6
                wh.G008, // C6 (universal)
                wh.A009, // C13 (universal)
                wh.S877, // C14 (universal)
                wh.B735, // C15 (universal)
                wh.V928, // C16 (universal)
                wh.C414, // C17 (universal)
                wh.R259, // C18 (universal)
                wh.F216, // Pochven
            ],

            'Ore Site': c5_ore,
        },
        6: {
            'Combat Site': c6_combat,
            'Relic Site': c6_relic,
            'Data Site': [...c6_data, ...wh_gh],
            'Gas Site': c6_gas,
            Wormhole: [
                // K162s ordered by target class (H → L → N → C1 → C2/C3 → C4/C5 → C6 → Unknown)
                k162.K162_H,
                k162.K162_L,
                k162.K162_N,
                k162.K162_C1,
                k162.K162_C2,
                k162.K162_C3,
                k162.K162_C4,
                k162.K162_C5,
                k162.K162_C6,
                k162.K162_Unknown,
                // Normal wormholes ordered by target class (H → L → N → C1 → C2 → C3 → C4 → C5 → C6 → C12 → C13)
                wh.B520, // H
                wh.C391, // L
                wh.C140, // L (also appears in C5)
                wh.J377, // L (Turnur)
                wh.J492, // L (universal)
                wh.C248, // N
                wh.Z142, // N (also appears in C5)
                wh.Q003, // N (universal)
                wh.Q317, // C1
                wh.E004, // C1 (universal)
                wh.G024, // C2
                wh.L005, // C2 (universal)
                wh.L477, // C3
                wh.Z006, // C3 (universal)
                wh.Z457, // C4
                wh.M001, // C4 (universal)
                wh.V911, // C5
                wh.C008, // C5 (universal)
                wh.D792, // H (also appears in C5)
                wh.W237, // C6
                wh.G008, // C6 (universal)
                wh.A009, // C13 (universal)
                wh.S877, // C14 (universal)
                wh.B735, // C15 (universal)
                wh.V928, // C16 (universal)
                wh.C414, // C17 (universal)
                wh.R259, // C18 (universal)
                wh.F216, // Pochven
            ],

            'Ore Site': c6_ore,
        },
        12: {
            'Combat Site': c12_combat,
            Wormhole: [wh.Q063, wh.V898, wh.E587],
        },
        13: {
            Wormhole: [
                // K162s ordered by target class (H → L → N → C1 → C2/C3 → C4/C5 → C6 → Unknown)
                k162.K162_H,
                k162.K162_L,
                k162.K162_N,
                k162.K162_C1,
                k162.K162_C2,
                k162.K162_C3,
                k162.K162_C4,
                k162.K162_C5,
                k162.K162_C6,
                k162.K162_Unknown,
                // Normal wormholes ordered by target class (H → L → N → C1 → C2 → C3 → C4 → C5 → C6 → C12 → C13)
                wh.N110, // H (from C1)
                wh.B274, // H (from C2)
                wh.D845, // H (from C3)
                wh.J244, // L (from C1)
                wh.A239, // L (from C2)
                wh.U210, // L (from C3)
                wh.C391, // L (from C6)
                wh.J377, // L (Turnur)
                wh.U201, // L
                wh.Z060, // N (from C1)
                wh.E545, // N (from C2)
                wh.K346, // N (from C3)
                wh.C248, // N (from C6)
                wh.Q003, // N (universal)
                wh.P060, // C1 (from C4)
                wh.Z647, // C1 (from C2)
                wh.E004, // C1 (universal)
                wh.D382, // C2 (from C2)
                wh.N766, // C2 (from C4)
                wh.L005, // C2 (universal)
                wh.O477, // C3 (from C2)
                wh.C247, // C3 (from C4)
                wh.M267, // C3 (from C5)
                wh.Z006, // C3 (universal)
                wh.X877, // C4 (from C4)
                wh.Y683, // C4 (from C2)
                wh.M001, // C4 (universal)
                wh.N062, // C5 (from C2)
                wh.H296, // C5 (from C5)
                wh.H900, // C5 (from C4)
                wh.C008, // C5 (universal)
                wh.U574, // C6 (from C4)
                wh.V753, // C6 (from C5)
                wh.V911, // C6 (from C6)
                wh.W237, // C6 (from C6)
                wh.D792, // H (from C5/C6)
                wh.G008, // C6 (universal)
                wh.A009, // C13 (universal)
                wh.S877, // C14 (universal)
                wh.B735, // C15 (universal)
                wh.V928, // C16 (universal)
                wh.C414, // C17 (universal)
                wh.R259, // C18 (universal)
            ],

            'Ore Site': c13_ore,
            'Data Site': wh_gh,
        },
        14: {
            'Combat Site': c14_combat,
            Wormhole: [
                // K162s ordered by target class (H → L → N → C1 → C2/C3 → C4/C5 → C6 → Unknown)
                k162.K162_H,
                k162.K162_L,
                k162.K162_N,
                k162.K162_C1,
                k162.K162_C2,
                k162.K162_C3,
                k162.K162_C4,
                k162.K162_C5,
                k162.K162_C6,
                k162.K162_Unknown,
                // Normal wormholes ordered by target class (H → L → N → C1 → C2 → C3 → C4 → C5 → C6)
                wh.Z647, // C1
                wh.D382, // C2
                wh.O477, // C3
                wh.Y683, // C4
                wh.N062, // C5
                wh.R474, // C6
            ],
        },
        15: {
            'Combat Site': c15_combat,
            Wormhole: [
                // K162s ordered by target class (H → L → N → C1 → C2/C3 → C4/C5 → C6 → Unknown)
                k162.K162_H,
                k162.K162_L,
                k162.K162_N,
                k162.K162_C1,
                k162.K162_C2,
                k162.K162_C3,
                k162.K162_C4,
                k162.K162_C5,
                k162.K162_C6,
                k162.K162_Unknown,
                // Normal wormholes ordered by target class (H → L → N → C1 → C2 → C3 → C4 → C5 → C6)
                wh.Z647, // C1
                wh.D382, // C2
                wh.O477, // C3
                wh.Y683, // C4
                wh.N062, // C5
                wh.R474, // C6
            ],
        },
        16: {
            'Combat Site': c16_combat,
            Wormhole: [
                // K162s ordered by target class (H → L → N → C1 → C2/C3 → C4/C5 → C6 → Unknown)
                k162.K162_H,
                k162.K162_L,
                k162.K162_N,
                k162.K162_C1,
                k162.K162_C2,
                k162.K162_C3,
                k162.K162_C4,
                k162.K162_C5,
                k162.K162_C6,
                k162.K162_Unknown,
                // Normal wormholes ordered by target class (H → L → N → C1 → C2 → C3 → C4 → C5 → C6)
                wh.Z647, // C1
                wh.D382, // C2
                wh.O477, // C3
                wh.Y683, // C4
                wh.N062, // C5
                wh.R474, // C6
            ],
        },
        17: {
            'Combat Site': c17_combat,
            Wormhole: [
                // K162s ordered by target class (H → L → N → C1 → C2/C3 → C4/C5 → C6 → Unknown)
                k162.K162_H,
                k162.K162_L,
                k162.K162_N,
                k162.K162_C1,
                k162.K162_C2,
                k162.K162_C3,
                k162.K162_C4,
                k162.K162_C5,
                k162.K162_C6,
                k162.K162_Unknown,
                // Normal wormholes ordered by target class (H → L → N → C1 → C2 → C3 → C4 → C5 → C6)
                wh.Z647, // C1
                wh.D382, // C2
                wh.O477, // C3
                wh.Y683, // C4
                wh.N062, // C5
                wh.R474, // C6
            ],
        },
        18: {
            'Combat Site': c18_combat,
            Wormhole: [
                // K162s ordered by target class (H → L → N → C1 → C2/C3 → C4/C5 → C6 → Unknown)
                k162.K162_H,
                k162.K162_L,
                k162.K162_N,
                k162.K162_C1,
                k162.K162_C2,
                k162.K162_C3,
                k162.K162_C4,
                k162.K162_C5,
                k162.K162_C6,
                k162.K162_Unknown,
                // Normal wormholes ordered by target class (H → L → N → C1 → C2 → C3 → C4 → C5 → C6)
                wh.Z647, // C1
                wh.D382, // C2
                wh.O477, // C3
                wh.Y683, // C4
                wh.N062, // C5
                wh.R474, // C6
            ],
        },
    },
    known_space: {
        hs: {
            Wormhole: [
                // K162s ordered by target class (H → L → N → C1 → C2/C3 → C4/C5 → C6 → Unknown)
                k162.K162_H,
                k162.K162_L,
                k162.K162_N,
                k162.K162_C1,
                k162.K162_C2,
                k162.K162_C3,
                k162.K162_C4,
                k162.K162_C5,
                k162.K162_C6,
                k162.K162_Unknown,
                // Normal wormholes ordered by target class (H → L → N → C1 → C2 → C3 → C4 → C5 → C6 → C12 → C13)
                wh.A641, // H
                wh.R051, // L
                wh.J377, // L (Turnur)
                wh.V283, // N
                wh.Q003, // N (universal)
                wh.Z971, // C1
                wh.E004, // C1 (universal)
                wh.R943, // C2
                wh.L005, // C2 (universal)
                wh.X702, // C3
                wh.Z006, // C3 (universal)
                wh.O128, // C4
                wh.M001, // C4 (universal)
                wh.M555, // C5
                wh.C008, // C5 (universal)
                wh.B041, // C6
                wh.G008, // C6 (universal)
                wh.T458, // C12 (Thera)
                wh.A009, // C13 (universal)
                wh.S877, // C14 (universal)
                wh.B735, // C15 (universal)
                wh.V928, // C16 (universal)
                wh.C414, // C17 (universal)
                wh.R259, // C18 (universal)
                wh.C729, // Pochven
            ],

            'Data Site': hs_gh,
        },
        ls: {
            Wormhole: [
                // K162s ordered by target class (H → L → N → C1 → C2/C3 → C4/C5 → C6 → Unknown)
                k162.K162_H,
                k162.K162_L,
                k162.K162_N,
                k162.K162_C1,
                k162.K162_C2,
                k162.K162_C3,
                k162.K162_C4,
                k162.K162_C5,
                k162.K162_C6,
                k162.K162_Unknown,
                // Normal wormholes ordered by target class (H → L → N → C1 → C2 → C3 → C4 → C5 → C6 → C12 → C13)
                wh.B449, // H
                wh.N944, // L
                wh.J377, // L (Turnur)
                wh.S199, // N
                wh.Q003, // N (universal)
                wh.Z971, // C1
                wh.E004, // C1 (universal)
                wh.R943, // C2
                wh.L005, // C2 (universal)
                wh.X702, // C3
                wh.Z006, // C3 (universal)
                wh.O128, // C4
                wh.M001, // C4 (universal)
                wh.N432, // C5
                wh.C008, // C5 (universal)
                wh.U319, // C6
                wh.G008, // C6 (universal)
                wh.M164, // C12 (Thera)
                wh.A009, // C13 (universal)
                wh.S877, // C14 (universal)
                wh.B735, // C15 (universal)
                wh.V928, // C16 (universal)
                wh.C414, // C17 (universal)
                wh.R259, // C18 (universal)
                wh.C729, // Pochven
            ],

            'Data Site': ls_gh,
        },
        ns: {
            Wormhole: [
                // K162s ordered by target class (H → L → N → C1 → C2/C3 → C4/C5 → C6 → Unknown)
                k162.K162_H,
                k162.K162_L,
                k162.K162_N,
                k162.K162_C1,
                k162.K162_C2,
                k162.K162_C3,
                k162.K162_C4,
                k162.K162_C5,
                k162.K162_C6,
                k162.K162_Unknown,
                // Normal wormholes ordered by target class (H → L → N → C1 → C2 → C3 → C4 → C5 → C6 → C12 → C13)
                wh.B449, // H
                wh.N944, // L
                wh.J377, // L (Turnur)
                wh.S199, // N
                wh.Q003, // N (universal)
                wh.Z971, // C1
                wh.E004, // C1 (universal)
                wh.R943, // C2
                wh.L005, // C2 (universal)
                wh.X702, // C3
                wh.Z006, // C3 (universal)
                wh.O128, // C4
                wh.M001, // C4 (universal)
                wh.N432, // C5
                wh.C008, // C5 (universal)
                wh.U319, // C6
                wh.G008, // C6 (universal)
                wh.L031, // C12 (Thera)
                wh.A009, // C13 (universal)
                wh.S877, // C14 (universal)
                wh.B735, // C15 (universal)
                wh.V928, // C16 (universal)
                wh.C414, // C17 (universal)
                wh.R259, // C18 (universal)
                wh.C729, // Pochven
                wh.U372, // Pochven
            ],

            'Data Site': ns_gh,
        },
        pv: {
            Wormhole: [
                // K162s ordered by target class (H → L → N → C1 → C2/C3 → C4/C5 → C6 → Unknown)
                k162.K162_H,
                k162.K162_L,
                k162.K162_N,
                k162.K162_C1,
                k162.K162_C2,
                k162.K162_C3,
                k162.K162_C4,
                k162.K162_C5,
                k162.K162_C6,
                k162.K162_Unknown,
                // Normal wormholes ordered by target class (H → L → N → C1 → C2 → C3 → C4 → C5 → C6 → C12 → C13)
                wh.X450, // N
                wh.Q003, // N (universal)
                wh.E004, // C1 (universal)
                wh.L005, // C2 (universal)
                wh.Z006, // C3 (universal)
                wh.R081, // C4
                wh.M001, // C4 (universal)
                wh.C008, // C5 (universal)
                wh.G008, // C6 (universal)
                wh.A009, // C13 (universal)
                wh.S877, // C14 (universal)
                wh.B735, // C15 (universal)
                wh.V928, // C16 (universal)
                wh.C414, // C17 (universal)
                wh.R259, // C18 (universal)
            ],
        },
    },
};

export default signature_tree;

export const combat_sites = [
    ...c1_combat,
    ...c2_combat,
    ...c3_combat,
    ...c4_combat,
    ...c5_combat,
    ...c6_combat,
    ...c12_combat,
    ...c14_combat,
    ...c15_combat,
    ...c16_combat,
    ...c17_combat,
    ...c18_combat,
];

export const relic_sites = [...c1_relic, ...c2_relic, ...c3_relic, ...c4_relic, ...c5_relic, ...c6_relic, ...null_relic];

export const data_sites = [
    ...c1_data,
    ...c2_data,
    ...c3_data,
    ...c4_data,
    ...c5_data,
    ...c6_data,
    ...null_data,
    ...hs_gh,
    ...ls_gh,
    ...ns_gh,
    ...wh_gh,
];

export const gas_sites = [...c1_gas, ...c2_gas, ...c3_gas, ...c4_gas, ...c5_gas, ...c6_gas];

export const ore_sites = [...c1_ore, ...c2_ore, ...c3_ore, ...c4_ore, ...c5_ore, ...c6_ore];
