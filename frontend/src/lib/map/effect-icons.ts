// The game client's icon for each stat a wormhole effect changes, so a modifier list reads
// at a glance the way the fitting window does. Where the client has a clear attribute icon
// (armor, shield, signature radius, velocity…) that is the one; where the stat belongs to a
// module (hardeners, repairers, neutralizers, webs…) it is that module's icon.
//
// The files in `$lib/assets/eve-icons` are named after their client resource,
// `res:/ui/texture/icons/<name>.png`, from the SDE's `icons` table, and downscaled to 32px.

const files = import.meta.glob<string>('$lib/assets/eve-icons/*.png', {
	eager: true,
	query: '?url',
	import: 'default',
});

function icon(name: string): string | null {
	return files[`/src/lib/assets/eve-icons/${name}.png`] ?? null;
}

/** Stat (as `wormhole_effects.json` names it) to client icon file. */
const STAT_ICONS: Record<string, string> = {
	'Armor HP': '22_32_6',
	'Armor Resists': '34_64_16', // Multispectrum Energized Membrane
	'Shield Capacity': '22_32_7',
	'Shield Resist': '2_64_2', // Multispectrum Shield Hardener
	'Signature Radius': '22_32_14',
	'Capacitor Capacity': '1_64_5',
	'Capacitor Recharge Time': '22_32_16',
	'Targeting Range': '22_32_15',
	'Ship Velocity': '22_32_13',
	Inertia: '22_32_24',
	'Tracking Speed': '22_32_22',
	Damage: '26_64_1',
	'Small Weapon Damage': '13_64_1', // a small blaster
	'Missile Velocity': '21_64_12', // a missile launcher
	'Missile Explosion Radius': '22_32_14',
	'Missile Explosion Velocity': '22_32_13',
	'Drone Tracking': '5_64_14', // Omnidirectional Tracking Link
	'Stasis Webifier Strength': '12_64_6',
	'Target Painter Strength': '56_64_1',
	'NOS & Neut Drain Amount': '12_64_4',
	'Remote Cap Transmitter Amount': '1_64_2',
	'Local Armor Repair Amount': '1_64_11',
	'Remote Armor Repair Amount': 'remote_armor_repair',
	'Local Shield Repair Amount': '2_64_3',
	'Shield Transfer Amount': '1_64_16',
	'Heat Damage': '22_32_10',
	'Overload Bonus': '22_32_10',
	'Smart Bomb Range': '3_64_4',
	'Smart Bomb Damage': '3_64_4',
	'Bomb Damage': '49_64_7',
};

/** The icon for a stat, or null for one the list does not know. */
export function statIcon(stat: string): string | null {
	const name = STAT_ICONS[stat];
	return name ? icon(name) : null;
}

/** Every stat with an icon, for the test that keeps the list complete. */
export const STATS_WITH_ICONS = Object.keys(STAT_ICONS);
