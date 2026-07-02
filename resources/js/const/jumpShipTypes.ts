/**
 * Jump-drive capable ship classes and their base jump ranges in light years.
 *
 * This mirrors the PHP `App\Enums\JumpShipType` enum so labels and ranges live
 * in exactly one place per side. Jump Drive Calibration adds 20% per level, so
 * the range is doubled at level V.
 */

import { TJumpShipType } from '@/types/models';

export type TJumpShipTypeMeta = {
    value: TJumpShipType;
    label: string;
    baseRangeLy: number;
};

export const jumpShipTypes: TJumpShipTypeMeta[] = [
    { value: 'dreadnought', label: 'Dreadnought', baseRangeLy: 3.5 },
    { value: 'carrier', label: 'Carrier', baseRangeLy: 3.5 },
    { value: 'force_auxiliary', label: 'Force Auxiliary', baseRangeLy: 3.5 },
    { value: 'supercarrier', label: 'Supercarrier', baseRangeLy: 3.0 },
    { value: 'titan', label: 'Titan', baseRangeLy: 3.0 },
    { value: 'jump_freighter', label: 'Jump Freighter', baseRangeLy: 5.0 },
    { value: 'rorqual', label: 'Rorqual', baseRangeLy: 5.0 },
    { value: 'black_ops', label: 'Black Ops', baseRangeLy: 4.0 },
];

const metaByValue = new Map<TJumpShipType, TJumpShipTypeMeta>(jumpShipTypes.map((meta) => [meta.value, meta]));

export function jumpShipLabel(value: TJumpShipType): string {
    return metaByValue.get(value)?.label ?? value;
}

/** Maximum jump range in light years for a ship class at the given JDC level. */
export function maxRangeLy(value: TJumpShipType, jdcLevel: number): number {
    return (metaByValue.get(value)?.baseRangeLy ?? 0) * (1 + 0.2 * jdcLevel);
}
