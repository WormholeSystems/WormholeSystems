import { alertTriggerLabel } from '@/lib/mapAlerts';
import { describe, expect, it } from 'vitest';

describe('alertTriggerLabel', () => {
    it('describes a maintainer podium alert without touching target/jump fields', () => {
        const label = alertTriggerLabel({
            type: 'maintainer_podium',
            max_jumps: null,
            ship_type: null,
            jdc_level: null,
            target_solarsystem: null,
            target_solarsystem_id: null,
            origin_solarsystem: null,
            origin_solarsystem_id: null,
        });

        expect(label).toBe('Monthly maintainer podium, posted on the 1st');
    });
});
