import { maxRangeLy } from '@/const/jumpShipTypes';
import type { TMapAlert } from '@/types/models';

type TAlertTriggerFields = Pick<TMapAlert, 'type' | 'max_jumps' | 'ship_type' | 'jdc_level' | 'target_solarsystem' | 'target_solarsystem_id'>;

/**
 * Human-readable description of what fires an alert, shared by every alert list.
 * Pass a resolver when the alert payload does not include its target solarsystem.
 */
export function alertTriggerLabel(alert: TAlertTriggerFields, resolveSystemName?: (id: number | null) => string): string {
    const systemName = alert.target_solarsystem?.name ?? resolveSystemName?.(alert.target_solarsystem_id) ?? 'Unknown system';
    const jumpsPlural = alert.max_jumps === 1 ? '' : 's';

    if (alert.type === 'killmail') {
        return `Kills within ${alert.max_jumps} jump${jumpsPlural} of the chain`;
    }
    if (alert.type === 'jump_range') {
        return `Exits within ${maxRangeLy(alert.ship_type ?? 'dreadnought', alert.jdc_level ?? 5).toFixed(1)} ly of ${systemName}`;
    }
    return `${systemName} within ${alert.max_jumps} gate jump${jumpsPlural}`;
}
