<?php

declare(strict_types=1);

namespace App\Services\Discord;

use App\Models\MapAlert;
use App\Models\Solarsystem;

/**
 * Builds the Discord embed for a jump-range alert: a k-space exit was just added to
 * the map within the configured ship's jump range of the target system. Coloured
 * by the exit's security status.
 */
final readonly class JumpRangeAlertEmbed
{
    use ColorsBySecurity;

    /**
     * @return array<string, mixed>
     */
    public function build(MapAlert $alert, Solarsystem $exit, Solarsystem $target, float $distanceLy): array
    {
        $ship = $alert->ship_type;

        $dotlanUrl = sprintf(
            'https://evemaps.dotlan.net/range/%s,%d/%s',
            $ship->dotlanShipName(),
            $alert->jdc_level,
            rawurlencode(str_replace(' ', '_', $exit->name)),
        );

        return [
            'title' => sprintf('New exit %.2f ly from %s', $distanceLy, $target->name),
            'url' => route('maps.show', $alert->map),
            'description' => sprintf(
                '**%s** was just added to your map, **%.2f ly** from **%s** — within %s jump range.',
                $exit->name,
                $distanceLy,
                $target->name,
                $ship->label(),
            ),
            'color' => $this->colorForSecurity((float) $exit->security),
            'fields' => [
                ['name' => 'Exit', 'value' => sprintf('%s (%.1f)', $exit->name, $exit->security), 'inline' => true],
                ['name' => 'Target', 'value' => sprintf('%s (%.1f)', $target->name, $target->security), 'inline' => true],
                ['name' => 'Distance', 'value' => sprintf('%.2f ly', $distanceLy), 'inline' => true],
                ['name' => 'Ship', 'value' => sprintf('%s (JDC %d) — %.1f ly max', $ship->label(), $alert->jdc_level, $ship->maxRangeLy($alert->jdc_level)), 'inline' => true],
                ['name' => 'Region', 'value' => $exit->region->name, 'inline' => true],
                ['name' => 'Links', 'value' => sprintf('[Jump range map](%s)', $dotlanUrl), 'inline' => true],
            ],
        ];
    }
}
