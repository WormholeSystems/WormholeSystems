/**
 * Single source of truth for solarsystem class metadata on the frontend.
 *
 * Data is loaded from the generated JSON file. To regenerate, run:
 *   php artisan generate:solarsystem-classes
 *
 * This mirrors the PHP `App\Enums\SolarsystemClass` enum so labels, colours and
 * groupings (standard / drifter / known-space) live in exactly one place.
 */

import classData from '@/data/solarsystem_classes.json';
import { TSolarsystemClassMeta, TStringedSolarsystemClass } from '@/types/models';

export const solarsystemClasses: TSolarsystemClassMeta[] = classData.classes as TSolarsystemClassMeta[];

const metaByValue = new Map<string, TSolarsystemClassMeta>(solarsystemClasses.map((meta) => [meta.value, meta]));

const unknownMeta = metaByValue.get('unknown')!;

/**
 * Resolve the metadata for a class value, falling back to the "unknown" entry
 * so callers never have to null-check.
 */
export function classMeta(value: TStringedSolarsystemClass | null | undefined): TSolarsystemClassMeta {
    if (value === null || value === undefined) return unknownMeta;
    return metaByValue.get(String(value)) ?? unknownMeta;
}

/** Whether the class is wormhole space (C1-C23). */
export function isWormholeClass(value: TStringedSolarsystemClass | null | undefined): boolean {
    return classMeta(value).is_wormhole_space;
}

/** Stable ordering weight: known space first, then wormholes by class number. */
export function classSortWeight(value: TStringedSolarsystemClass | null | undefined): number {
    return classMeta(value).sort_weight;
}
