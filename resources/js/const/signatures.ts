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

const c1_wh = [
    'H121 - C1',
    'C125 - C2',
    'O883 - C3',
    'M609 - C4',
    'L614 - C5',
    'S804 - C6',
    'N110 - H',
    'J244 - L',
    'J377 - L Turnur',
    'Z060 - 0.0',
    'F353 - C12 Thera',
];

const c2_wh = [
    'Z647 - C1',
    'D382 - C2',
    'O477 - C3',
    'Y683 - C4',
    'N062 - C5',
    'R474 - C6',
    'B274 - H',
    'A239 - L',
    'J377 - L Turnur',
    'E545 - 0.0',
    'F135 - C12 Thera',
    'F216 - T Pochven',
];

const c3_wh = [
    'V301 - C1',
    'I182 - C2',
    'N968 - C3',
    'T405 - C4',
    'N770 - C5',
    'A982 - C6',
    'D845 - H',
    'U210 - L',
    'J377 - L Turnur',
    'K346 - 0.0',
    'F135 - C12 Thera',
    'F216 - T Pochven',
];

const c4_wh = [
    'P060 - C1',
    'N766 - C2',
    'C247 - C3',
    'X877 - C4',
    'H900 - C5',
    'U574 - C6',
    'S047 - H',
    'N290 - L',
    'J377 - L Turnur',
    'K329 - 0.0',
    'F216 - T Pochven',
];

const c5_wh = [
    'Y790 - C1',
    'D364 - C2',
    'M267 - C3',
    'E175 - C4',
    'H296 - C5',
    'V753 - C6',
    'D792 - H',
    'C140 - L',
    'J377 - L Turnur',
    'Z142 - 0.0',
    'F216 - T Pochven',
];

const c6_wh = [
    'Q317 - C1',
    'G024 - C2',
    'L477 - C3',
    'Z457 - C4',
    'V911 - C5',
    'W237 - C6',
    'B520 - H',
    'D792 - H',
    'C140 - L',
    'C391 - L',
    'J377 - L Turnur',
    'C248 - 0.0',
    'Z142 - 0.0',
    'F216 - T Pochven',
];

// Shattered WH
const c13_wh = [
    'P060 - C1',
    'Z647 - C1',
    'D382 - C2',
    'L005 - C2',
    'N766 - C2',
    'C247 - C3',
    'M267 - C3',
    'O477 - C3',
    'X877 - C4',
    'Y683 - C4',
    'H296 - C5',
    'H900 - C5',
    'H296 - C5',
    'N062 - C5',
    'V911 - C5',
    'U574 - C6',
    'V753 - C6',
    'W237 - C6',
    'B274 - H',
    'D792 - H',
    'D845 - H',
    'N110 - H',
    'A239 - L',
    'C391 - L',
    'J244 - L',
    'J377 - L Turnur',
    'U201 - L',
    'U210 - L',
    'C248 - 0.0',
    'E545 - 0.0',
    'K346 - 0.0',
    'Z060 - 0.0',
];

const hs_wh = [
    'Z971 - C1',
    'R943 - C2',
    'X702 - C3',
    'O128 - C4',
    'M555 - C5',
    'B041 - C6',
    'A641 - H',
    'R051 - L',
    'J377 - L Turnur',
    'V283 - 0.0',
    'T458 - C12 Thera',
    'C729 - T Pochven',
];

const ls_wh = [
    'Z971 - C1',
    'R943 - C2',
    'X702 - C3',
    'O128 - C4',
    'N432 - C5',
    'U319 - C6',
    'B449 - H',
    'N944 - L',
    'J377 - L Turnur',
    'S199 - 0.0',
    'M164 - C12 Thera',
    'C729 - T Pochven',
];

const null_wh = [
    'Z971 - C1',
    'R943 - C2',
    'X702 - C3',
    'O128 - C4',
    'N432 - C5',
    'U319 - C6',
    'B449 - H',
    'N944 - L',
    'J377 - L Turnur',
    'S199 - 0.0',
    'L031 - C12 Thera',
    'C729 - T Pochven',
    'U372 - T Pochven',
];

const poch_wh = ['R081 - C4', 'X450 - 0.0'];

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
            Wormhole: c1_wh,
            'Ore Site': c1_ore,
        },
        2: {
            'Combat Site': c2_combat,
            'Relic Site': [...c2_relic, ...null_relic],
            'Data Site': [...c2_data, ...null_data, ...wh_gh],
            'Gas Site': c2_gas,
            Wormhole: c2_wh,
            'Ore Site': c2_ore,
        },
        3: {
            'Combat Site': c3_combat,
            'Relic Site': [...c3_relic, ...null_relic],
            'Data Site': [...c3_data, ...null_data, ...wh_gh],
            'Gas Site': c3_gas,
            Wormhole: c3_wh,
            'Ore Site': c3_ore,
        },
        4: {
            'Combat Site': c4_combat,
            'Relic Site': c4_relic,
            'Data Site': [...c4_data, ...wh_gh],
            'Gas Site': c4_gas,
            Wormhole: c4_wh,
            'Ore Site': c4_ore,
        },
        5: {
            'Combat Site': c5_combat,
            'Relic Site': c5_relic,
            'Data Site': [...c5_data, ...wh_gh],
            'Gas Site': c5_gas,
            Wormhole: c5_wh,
            'Ore Site': c5_ore,
        },
        6: {
            'Combat Site': c6_combat,
            'Relic Site': c6_relic,
            'Data Site': [...c6_data, ...wh_gh],
            'Gas Site': c6_gas,
            Wormhole: c6_wh,
            'Ore Site': c6_ore,
        },
        12: {
            'Combat Site': c12_combat,
        },
        13: {
            wormhole: c13_wh,
            ore: c13_ore,
            ghost: wh_gh,
        },
        14: {
            'Combat Site': c14_combat,
        },
        15: {
            'Combat Site': c15_combat,
        },
        16: {
            'Combat Site': c16_combat,
        },
        17: {
            'Combat Site': c17_combat,
        },
        18: {
            'Combat Site': c18_combat,
        },
    },
    known_space: {
        hs: {
            wormhole: hs_wh,
            ghost: hs_gh,
        },
        ls: {
            wormhole: ls_wh,
            ghost: ls_gh,
        },
        ns: {
            wormhole: null_wh,
            ghost: ns_gh,
        },
        pv: {
            wormhole: poch_wh,
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
