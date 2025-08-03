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

const c1_wh = [
    { name: 'K162 - Unknown', signature: 'K162', target_class: 'Unknown', extra: '' },
    { name: 'K162 - C1', signature: 'K162', target_class: 'C1', extra: '' },
    { name: 'K162 - C2/C3', signature: 'K162', target_class: 'C2/C3', extra: '' },
    { name: 'K162 - C4/C5', signature: 'K162', target_class: 'C4/C5', extra: '' },
    { name: 'K162 - C6', signature: 'K162', target_class: 'C6', extra: '' },
    { name: 'K162 - H', signature: 'K162', target_class: 'H', extra: '' },
    { name: 'K162 - L', signature: 'K162', target_class: 'L', extra: '' },
    { name: 'K162 - N', signature: 'K162', target_class: 'N', extra: '' },
    { name: 'H121 - C1', signature: 'H121', target_class: 'C1', extra: '' },
    { name: 'C125 - C2', signature: 'C125', target_class: 'C2', extra: '' },
    { name: 'O883 - C3', signature: 'O883', target_class: 'C3', extra: '' },
    { name: 'M609 - C4', signature: 'M609', target_class: 'C4', extra: '' },
    { name: 'L614 - C5', signature: 'L614', target_class: 'C5', extra: '' },
    { name: 'S804 - C6', signature: 'S804', target_class: 'C6', extra: '' },
    { name: 'N110 - H', signature: 'N110', target_class: 'H', extra: '' },
    { name: 'J244 - L', signature: 'J244', target_class: 'L', extra: '' },
    { name: 'J377 - L Turnur', signature: 'J377', target_class: 'L', extra: 'Turnur' },
    { name: 'Z060 - 0.0', signature: 'Z060', target_class: 'N', extra: '' },
    { name: 'F353 - C12 Thera', signature: 'F353', target_class: 'C12', extra: 'Thera' },
];

const c2_wh = [
    { name: 'K162 - Unknown', signature: 'K162', target_class: 'Unknown', extra: '' },
    { name: 'K162 - C1', signature: 'K162', target_class: 'C1', extra: '' },
    { name: 'K162 - C2/C3', signature: 'K162', target_class: 'C2/C3', extra: '' },
    { name: 'K162 - C4/C5', signature: 'K162', target_class: 'C4/C5', extra: '' },
    { name: 'K162 - C6', signature: 'K162', target_class: 'C6', extra: '' },
    { name: 'K162 - H', signature: 'K162', target_class: 'H', extra: '' },
    { name: 'K162 - L', signature: 'K162', target_class: 'L', extra: '' },
    { name: 'K162 - N', signature: 'K162', target_class: 'N', extra: '' },
    { name: 'Z647 - C1', signature: 'Z647', target_class: 'C1', extra: '' },
    { name: 'D382 - C2', signature: 'D382', target_class: 'C2', extra: '' },
    { name: 'O477 - C3', signature: 'O477', target_class: 'C3', extra: '' },
    { name: 'Y683 - C4', signature: 'Y683', target_class: 'C4', extra: '' },
    { name: 'N062 - C5', signature: 'N062', target_class: 'C5', extra: '' },
    { name: 'R474 - C6', signature: 'R474', target_class: 'C6', extra: '' },
    { name: 'B274 - H', signature: 'B274', target_class: 'H', extra: '' },
    { name: 'A239 - L', signature: 'A239', target_class: 'L', extra: '' },
    { name: 'J377 - L Turnur', signature: 'J377', target_class: 'L', extra: 'Turnur' },
    { name: 'E545 - 0.0', signature: 'E545', target_class: 'N', extra: '' },
    { name: 'F135 - C12 Thera', signature: 'F135', target_class: 'C12', extra: 'Thera' },
    { name: 'F216 - T Pochven', signature: 'F216', target_class: 'N', extra: 'Pochven' },
];

const c3_wh = [
    { name: 'K162 - Unknown', signature: 'K162', target_class: 'Unknown', extra: '' },
    { name: 'K162 - C1', signature: 'K162', target_class: 'C1', extra: '' },
    { name: 'K162 - C2/C3', signature: 'K162', target_class: 'C2/C3', extra: '' },
    { name: 'K162 - C4/C5', signature: 'K162', target_class: 'C4/C5', extra: '' },
    { name: 'K162 - C6', signature: 'K162', target_class: 'C6', extra: '' },
    { name: 'K162 - H', signature: 'K162', target_class: 'H', extra: '' },
    { name: 'K162 - L', signature: 'K162', target_class: 'L', extra: '' },
    { name: 'K162 - N', signature: 'K162', target_class: 'N', extra: '' },
    { name: 'V301 - C1', signature: 'V301', target_class: 'C1', extra: '' },
    { name: 'I182 - C2', signature: 'I182', target_class: 'C2', extra: '' },
    { name: 'N968 - C3', signature: 'N968', target_class: 'C3', extra: '' },
    { name: 'T405 - C4', signature: 'T405', target_class: 'C4', extra: '' },
    { name: 'N770 - C5', signature: 'N770', target_class: 'C5', extra: '' },
    { name: 'A982 - C6', signature: 'A982', target_class: 'C6', extra: '' },
    { name: 'D845 - H', signature: 'D845', target_class: 'H', extra: '' },
    { name: 'U210 - L', signature: 'U210', target_class: 'L', extra: '' },
    { name: 'J377 - L Turnur', signature: 'J377', target_class: 'L', extra: 'Turnur' },
    { name: 'K346 - 0.0', signature: 'K346', target_class: 'N', extra: '' },
    { name: 'F135 - C12 Thera', signature: 'F135', target_class: 'C12', extra: 'Thera' },
    { name: 'F216 - T Pochven', signature: 'F216', target_class: 'N', extra: 'Pochven' },
];

const c4_wh = [
    { name: 'K162 - Unknown', signature: 'K162', target_class: 'Unknown', extra: '' },
    { name: 'K162 - C1', signature: 'K162', target_class: 'C1', extra: '' },
    { name: 'K162 - C2/C3', signature: 'K162', target_class: 'C2/C3', extra: '' },
    { name: 'K162 - C4/C5', signature: 'K162', target_class: 'C4/C5', extra: '' },
    { name: 'K162 - C6', signature: 'K162', target_class: 'C6', extra: '' },
    { name: 'K162 - H', signature: 'K162', target_class: 'H', extra: '' },
    { name: 'K162 - L', signature: 'K162', target_class: 'L', extra: '' },
    { name: 'K162 - N', signature: 'K162', target_class: 'N', extra: '' },
    { name: 'P060 - C1', signature: 'P060', target_class: 'C1', extra: '' },
    { name: 'N766 - C2', signature: 'N766', target_class: 'C2', extra: '' },
    { name: 'C247 - C3', signature: 'C247', target_class: 'C3', extra: '' },
    { name: 'X877 - C4', signature: 'X877', target_class: 'C4', extra: '' },
    { name: 'H900 - C5', signature: 'H900', target_class: 'C5', extra: '' },
    { name: 'U574 - C6', signature: 'U574', target_class: 'C6', extra: '' },
    { name: 'S047 - H', signature: 'S047', target_class: 'H', extra: '' },
    { name: 'N290 - L', signature: 'N290', target_class: 'L', extra: '' },
    { name: 'J377 - L Turnur', signature: 'J377', target_class: 'L', extra: 'Turnur' },
    { name: 'K329 - 0.0', signature: 'K329', target_class: 'N', extra: '' },
    { name: 'F216 - T Pochven', signature: 'F216', target_class: 'N', extra: 'Pochven' },
];

const c5_wh = [
    { name: 'K162 - Unknown', signature: 'K162', target_class: 'Unknown', extra: '' },
    { name: 'K162 - C1', signature: 'K162', target_class: 'C1', extra: '' },
    { name: 'K162 - C2/C3', signature: 'K162', target_class: 'C2/C3', extra: '' },
    { name: 'K162 - C4/C5', signature: 'K162', target_class: 'C4/C5', extra: '' },
    { name: 'K162 - C6', signature: 'K162', target_class: 'C6', extra: '' },
    { name: 'K162 - H', signature: 'K162', target_class: 'H', extra: '' },
    { name: 'K162 - L', signature: 'K162', target_class: 'L', extra: '' },
    { name: 'K162 - N', signature: 'K162', target_class: 'N', extra: '' },
    { name: 'Y790 - C1', signature: 'Y790', target_class: 'C1', extra: '' },
    { name: 'D364 - C2', signature: 'D364', target_class: 'C2', extra: '' },
    { name: 'M267 - C3', signature: 'M267', target_class: 'C3', extra: '' },
    { name: 'E175 - C4', signature: 'E175', target_class: 'C4', extra: '' },
    { name: 'H296 - C5', signature: 'H296', target_class: 'C5', extra: '' },
    { name: 'V753 - C6', signature: 'V753', target_class: 'C6', extra: '' },
    { name: 'D792 - H', signature: 'D792', target_class: 'H', extra: '' },
    { name: 'C140 - L', signature: 'C140', target_class: 'L', extra: '' },
    { name: 'J377 - L Turnur', signature: 'J377', target_class: 'L', extra: 'Turnur' },
    { name: 'Z142 - 0.0', signature: 'Z142', target_class: 'N', extra: '' },
    { name: 'F216 - T Pochven', signature: 'F216', target_class: 'N', extra: 'Pochven' },
];

const c6_wh = [
    { name: 'K162 - Unknown', signature: 'K162', target_class: 'Unknown', extra: '' },
    { name: 'K162 - C1', signature: 'K162', target_class: 'C1', extra: '' },
    { name: 'K162 - C2/C3', signature: 'K162', target_class: 'C2/C3', extra: '' },
    { name: 'K162 - C4/C5', signature: 'K162', target_class: 'C4/C5', extra: '' },
    { name: 'K162 - C6', signature: 'K162', target_class: 'C6', extra: '' },
    { name: 'K162 - H', signature: 'K162', target_class: 'H', extra: '' },
    { name: 'K162 - L', signature: 'K162', target_class: 'L', extra: '' },
    { name: 'K162 - N', signature: 'K162', target_class: 'N', extra: '' },
    { name: 'Q317 - C1', signature: 'Q317', target_class: 'C1', extra: '' },
    { name: 'G024 - C2', signature: 'G024', target_class: 'C2', extra: '' },
    { name: 'L477 - C3', signature: 'L477', target_class: 'C3', extra: '' },
    { name: 'Z457 - C4', signature: 'Z457', target_class: 'C4', extra: '' },
    { name: 'V911 - C5', signature: 'V911', target_class: 'C5', extra: '' },
    { name: 'W237 - C6', signature: 'W237', target_class: 'C6', extra: '' },
    { name: 'B520 - H', signature: 'B520', target_class: 'H', extra: '' },
    { name: 'D792 - H', signature: 'D792', target_class: 'H', extra: '' },
    { name: 'C140 - L', signature: 'C140', target_class: 'L', extra: '' },
    { name: 'C391 - L', signature: 'C391', target_class: 'L', extra: '' },
    { name: 'J377 - L Turnur', signature: 'J377', target_class: 'L', extra: 'Turnur' },
    { name: 'C248 - 0.0', signature: 'C248', target_class: 'N', extra: '' },
    { name: 'Z142 - 0.0', signature: 'Z142', target_class: 'N', extra: '' },
    { name: 'F216 - T Pochven', signature: 'F216', target_class: 'N', extra: 'Pochven' },
];

// Shattered WH
const c13_wh = [
    { name: 'K162 - Unknown', signature: 'K162', target_class: 'Unknown', extra: '' },
    { name: 'K162 - C1', signature: 'K162', target_class: 'C1', extra: '' },
    { name: 'K162 - C2/C3', signature: 'K162', target_class: 'C2/C3', extra: '' },
    { name: 'K162 - C4/C5', signature: 'K162', target_class: 'C4/C5', extra: '' },
    { name: 'K162 - C6', signature: 'K162', target_class: 'C6', extra: '' },
    { name: 'K162 - H', signature: 'K162', target_class: 'H', extra: '' },
    { name: 'K162 - L', signature: 'K162', target_class: 'L', extra: '' },
    { name: 'K162 - N', signature: 'K162', target_class: 'N', extra: '' },
    { name: 'P060 - C1', signature: 'P060', target_class: 'C1', extra: '' },
    { name: 'Z647 - C1', signature: 'Z647', target_class: 'C1', extra: '' },
    { name: 'D382 - C2', signature: 'D382', target_class: 'C2', extra: '' },
    { name: 'L005 - C2', signature: 'L005', target_class: 'C2', extra: '' },
    { name: 'N766 - C2', signature: 'N766', target_class: 'C2', extra: '' },
    { name: 'C247 - C3', signature: 'C247', target_class: 'C3', extra: '' },
    { name: 'M267 - C3', signature: 'M267', target_class: 'C3', extra: '' },
    { name: 'O477 - C3', signature: 'O477', target_class: 'C3', extra: '' },
    { name: 'X877 - C4', signature: 'X877', target_class: 'C4', extra: '' },
    { name: 'Y683 - C4', signature: 'Y683', target_class: 'C4', extra: '' },
    { name: 'H296 - C5', signature: 'H296', target_class: 'C5', extra: '' },
    { name: 'H900 - C5', signature: 'H900', target_class: 'C5', extra: '' },
    { name: 'H296 - C5', signature: 'H296', target_class: 'C5', extra: '' },
    { name: 'N062 - C5', signature: 'N062', target_class: 'C5', extra: '' },
    { name: 'V911 - C5', signature: 'V911', target_class: 'C5', extra: '' },
    { name: 'U574 - C6', signature: 'U574', target_class: 'C6', extra: '' },
    { name: 'V753 - C6', signature: 'V753', target_class: 'C6', extra: '' },
    { name: 'W237 - C6', signature: 'W237', target_class: 'C6', extra: '' },
    { name: 'B274 - H', signature: 'B274', target_class: 'H', extra: '' },
    { name: 'D792 - H', signature: 'D792', target_class: 'H', extra: '' },
    { name: 'D845 - H', signature: 'D845', target_class: 'H', extra: '' },
    { name: 'N110 - H', signature: 'N110', target_class: 'H', extra: '' },
    { name: 'A239 - L', signature: 'A239', target_class: 'L', extra: '' },
    { name: 'C391 - L', signature: 'C391', target_class: 'L', extra: '' },
    { name: 'J244 - L', signature: 'J244', target_class: 'L', extra: '' },
    { name: 'J377 - L Turnur', signature: 'J377', target_class: 'L', extra: 'Turnur' },
    { name: 'U201 - L', signature: 'U201', target_class: 'L', extra: '' },
    { name: 'U210 - L', signature: 'U210', target_class: 'L', extra: '' },
    { name: 'C248 - 0.0', signature: 'C248', target_class: 'N', extra: '' },
    { name: 'E545 - 0.0', signature: 'E545', target_class: 'N', extra: '' },
    { name: 'K346 - 0.0', signature: 'K346', target_class: 'N', extra: '' },
    { name: 'Z060 - 0.0', signature: 'Z060', target_class: 'N', extra: '' },
];

const hs_wh = [
    { name: 'K162 - Unknown', signature: 'K162', target_class: 'Unknown', extra: '' },
    { name: 'K162 - C1', signature: 'K162', target_class: 'C1', extra: '' },
    { name: 'K162 - C2/C3', signature: 'K162', target_class: 'C2/C3', extra: '' },
    { name: 'K162 - C4/C5', signature: 'K162', target_class: 'C4/C5', extra: '' },
    { name: 'K162 - C6', signature: 'K162', target_class: 'C6', extra: '' },
    { name: 'K162 - H', signature: 'K162', target_class: 'H', extra: '' },
    { name: 'K162 - L', signature: 'K162', target_class: 'L', extra: '' },
    { name: 'K162 - N', signature: 'K162', target_class: 'N', extra: '' },
    { name: 'Z971 - C1', signature: 'Z971', target_class: 'C1', extra: '' },
    { name: 'R943 - C2', signature: 'R943', target_class: 'C2', extra: '' },
    { name: 'X702 - C3', signature: 'X702', target_class: 'C3', extra: '' },
    { name: 'O128 - C4', signature: 'O128', target_class: 'C4', extra: '' },
    { name: 'M555 - C5', signature: 'M555', target_class: 'C5', extra: '' },
    { name: 'B041 - C6', signature: 'B041', target_class: 'C6', extra: '' },
    { name: 'A641 - H', signature: 'A641', target_class: 'H', extra: '' },
    { name: 'R051 - L', signature: 'R051', target_class: 'L', extra: '' },
    { name: 'J377 - L Turnur', signature: 'J377', target_class: 'L', extra: 'Turnur' },
    { name: 'V283 - 0.0', signature: 'V283', target_class: 'N', extra: '' },
    { name: 'T458 - C12 Thera', signature: 'T458', target_class: 'C12', extra: 'Thera' },
    { name: 'C729 - T Pochven', signature: 'C729', target_class: 'N', extra: 'Pochven' },
];

const ls_wh = [
    { name: 'K162 - Unknown', signature: 'K162', target_class: 'Unknown', extra: '' },
    { name: 'K162 - C1', signature: 'K162', target_class: 'C1', extra: '' },
    { name: 'K162 - C2/C3', signature: 'K162', target_class: 'C2/C3', extra: '' },
    { name: 'K162 - C4/C5', signature: 'K162', target_class: 'C4/C5', extra: '' },
    { name: 'K162 - C6', signature: 'K162', target_class: 'C6', extra: '' },
    { name: 'K162 - H', signature: 'K162', target_class: 'H', extra: '' },
    { name: 'K162 - L', signature: 'K162', target_class: 'L', extra: '' },
    { name: 'K162 - N', signature: 'K162', target_class: 'N', extra: '' },
    { name: 'Z971 - C1', signature: 'Z971', target_class: 'C1', extra: '' },
    { name: 'R943 - C2', signature: 'R943', target_class: 'C2', extra: '' },
    { name: 'X702 - C3', signature: 'X702', target_class: 'C3', extra: '' },
    { name: 'O128 - C4', signature: 'O128', target_class: 'C4', extra: '' },
    { name: 'N432 - C5', signature: 'N432', target_class: 'C5', extra: '' },
    { name: 'U319 - C6', signature: 'U319', target_class: 'C6', extra: '' },
    { name: 'B449 - H', signature: 'B449', target_class: 'H', extra: '' },
    { name: 'N944 - L', signature: 'N944', target_class: 'L', extra: '' },
    { name: 'J377 - L Turnur', signature: 'J377', target_class: 'L', extra: 'Turnur' },
    { name: 'S199 - 0.0', signature: 'S199', target_class: 'N', extra: '' },
    { name: 'M164 - C12 Thera', signature: 'M164', target_class: 'C12', extra: 'Thera' },
    { name: 'C729 - T Pochven', signature: 'C729', target_class: 'N', extra: 'Pochven' },
];

const null_wh = [
    { name: 'K162 - Unknown', signature: 'K162', target_class: 'Unknown', extra: '' },
    { name: 'K162 - C1', signature: 'K162', target_class: 'C1', extra: '' },
    { name: 'K162 - C2/C3', signature: 'K162', target_class: 'C2/C3', extra: '' },
    { name: 'K162 - C4/C5', signature: 'K162', target_class: 'C4/C5', extra: '' },
    { name: 'K162 - C6', signature: 'K162', target_class: 'C6', extra: '' },
    { name: 'K162 - H', signature: 'K162', target_class: 'H', extra: '' },
    { name: 'K162 - L', signature: 'K162', target_class: 'L', extra: '' },
    { name: 'K162 - N', signature: 'K162', target_class: 'N', extra: '' },
    { name: 'Z971 - C1', signature: 'Z971', target_class: 'C1', extra: '' },
    { name: 'R943 - C2', signature: 'R943', target_class: 'C2', extra: '' },
    { name: 'X702 - C3', signature: 'X702', target_class: 'C3', extra: '' },
    { name: 'O128 - C4', signature: 'O128', target_class: 'C4', extra: '' },
    { name: 'N432 - C5', signature: 'N432', target_class: 'C5', extra: '' },
    { name: 'U319 - C6', signature: 'U319', target_class: 'C6', extra: '' },
    { name: 'B449 - H', signature: 'B449', target_class: 'H', extra: '' },
    { name: 'N944 - L', signature: 'N944', target_class: 'L', extra: '' },
    { name: 'J377 - L Turnur', signature: 'J377', target_class: 'L', extra: 'Turnur' },
    { name: 'S199 - 0.0', signature: 'S199', target_class: 'N', extra: '' },
    { name: 'L031 - C12 Thera', signature: 'L031', target_class: 'C12', extra: 'Thera' },
    { name: 'C729 - T Pochven', signature: 'C729', target_class: 'N', extra: 'Pochven' },
    { name: 'U372 - T Pochven', signature: 'U372', target_class: 'N', extra: 'Pochven' },
];

const poch_wh = [
    { name: 'K162 - Unknown', signature: 'K162', target_class: 'Unknown', extra: '' },
    { name: 'K162 - C1', signature: 'K162', target_class: 'C1', extra: '' },
    { name: 'K162 - C2/C3', signature: 'K162', target_class: 'C2/C3', extra: '' },
    { name: 'K162 - C4/C5', signature: 'K162', target_class: 'C4/C5', extra: '' },
    { name: 'K162 - C6', signature: 'K162', target_class: 'C6', extra: '' },
    { name: 'K162 - H', signature: 'K162', target_class: 'H', extra: '' },
    { name: 'K162 - L', signature: 'K162', target_class: 'L', extra: '' },
    { name: 'K162 - N', signature: 'K162', target_class: 'N', extra: '' },
    { name: 'R081 - C4', signature: 'R081', target_class: 'C4', extra: '' },
    { name: 'X450 - 0.0', signature: 'X450', target_class: 'N', extra: '' },
];

const drifter_wh = [
    { name: 'K162 - Unknown', signature: 'K162', target_class: 'Unknown', extra: '' },
    { name: 'K162 - C1', signature: 'K162', target_class: 'C1', extra: '' },
    { name: 'K162 - C2/C3', signature: 'K162', target_class: 'C2/C3', extra: '' },
    { name: 'K162 - C4/C5', signature: 'K162', target_class: 'C4/C5', extra: '' },
    { name: 'K162 - C6', signature: 'K162', target_class: 'C6', extra: '' },
    { name: 'K162 - H', signature: 'K162', target_class: 'H', extra: '' },
    { name: 'K162 - L', signature: 'K162', target_class: 'L', extra: '' },
    { name: 'K162 - N', signature: 'K162', target_class: 'N', extra: '' },
    { name: 'Z647 - C1', signature: 'Z647', target_class: 'C1', extra: '' },
    { name: 'D382 - C2', signature: 'D382', target_class: 'C2', extra: '' },
    { name: 'O477 - C3', signature: 'O477', target_class: 'C3', extra: '' },
    { name: 'Y683 - C4', signature: 'Y683', target_class: 'C4', extra: '' },
    { name: 'N062 - C5', signature: 'N062', target_class: 'C5', extra: '' },
    { name: 'R474 - C6', signature: 'R474', target_class: 'C6', extra: '' },
];

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
            Wormhole: c13_wh,
            'Ore Site': c13_ore,
            'Data Site': wh_gh,
        },
        14: {
            'Combat Site': c14_combat,
            Wormhole: drifter_wh,
        },
        15: {
            'Combat Site': c15_combat,
            Wormhole: drifter_wh,
        },
        16: {
            'Combat Site': c16_combat,
            Wormhole: drifter_wh,
        },
        17: {
            'Combat Site': c17_combat,
            Wormhole: drifter_wh,
        },
        18: {
            'Combat Site': c18_combat,
            Wormhole: drifter_wh,
        },
    },
    known_space: {
        hs: {
            Wormhole: hs_wh,
            'Data Site': hs_gh,
        },
        ls: {
            Wormhole: ls_wh,
            'Data Site': ls_gh,
        },
        ns: {
            Wormhole: null_wh,
            'Data Site': ns_gh,
        },
        pv: {
            Wormhole: poch_wh,
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
